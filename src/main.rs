#![no_std]
#![no_main]
// Required by `#[handler]` macro.
#![feature(naked_functions)]

extern crate alloc;

use alloc::sync::Arc;
use hopter::{
    config,
    interrupt::declare::{handler, irq},
    sync::{Mailbox, Mutex, SpinIrqSafe},
    task::{self, main},
    time::IntervalBarrier,
};
use stm32f4xx_hal::{
    self,
    gpio::{Output, Pin},
    pac::TIM2,
    prelude::*,
    rcc::RccExt,
    timer::{CounterUs, Event},
};

type GreenLed = Pin<'D', 12, Output>;
type OrangeLed = Pin<'D', 13, Output>;
type RedLed = Pin<'D', 14, Output>;
type BlueLed = Pin<'D', 15, Output>;

// #################################
// # Part 0: Project Configuration #
// #################################
//
// Hopter needs to be built with a customized compiler toolchain with specific
// option flags turned on. It also has a specific pattern for the application
// to configure kernel parameters. Below lists the required items.
//
// 1. Put a file with name `rust-toolchain` at the repository root directory.
//    The file should contain a single line: "segstk-rust". This file instructs
//    `cargo` to use the customized compiler toolchain. Do not forget to
//    install it by following the instructions here:
//    <https://github.com/ZhiyaoMa98/hopter-compiler-toolchain>
//
// 2. Build the `core` and `alloc` crate locally instead of using the prebuilt
//    binary version. Also, enable unwinding upon panic. See the configuration
//    file located at `.cargo/config.toml`. Related lines are "-C panic=unwind"
//    and "-Z build-std=core,alloc" equivalent.
//
// 3. Put a file with name `memory.x` at the repository root directory. It
//    should contain the range description of the RAM and flash region. This is
//    the standard practice when building embedded programs with Rust. See
//    additional information aboud `memory.x` here:
//    <https://docs.rust-embedded.org/book/start/hardware.html#configuring>
//
// 4. To change the kernel configuration parameters, clone the crate
//    `hopter_conf_params` to a local directory, and override the dependency.
//    See `Cargo.toml` for the override. For the override to be successful, the
//    local version must have a large yet compatible semantic version number.
//    See `hopter-conf-params/Cargo.toml` for an example. The compatibility
//    rules are listed here:
//    <https://doc.rust-lang.org/cargo/reference/resolver.html#semver-compatibility>

// #################################
// # Part 1: System Initialization #
// #################################
//
// Hopter starts the main task after bootstrap. The entry function of the main
// task is marked by the `#[main]` attribute.
//
// The main task is usually for system initialization. By default, the main
// task has the highest task priority, so any task spawned by the main task
// will not cause an immediate context switch.
//
// The function serving as the entry of the main task should take an argument
// of type `cortex_m::Peripherals`.
//
// When the entry function returns, the system resource acquired by the main
// task will be automatically released. The same is also true for other tasks.
#[main]
fn main(mut cp: cortex_m::Peripherals) {
    // Acquire the board peripherals. Must not use `take()` because it
    // internally masks interrupts using `cpsid i` instruction. Hopter may
    // extend a function call stack via SVC, which leads to a hard fault when
    // `cpsid i` is in effect. Use `steal()` here to circumvent the problem.
    // Hopter uses other mechanisms to mask interrupts.
    let dp = unsafe { stm32f4xx_hal::pac::Peripherals::steal() };

    // Bring the system clock to the maximum speed on STM32F407.
    let clocks = dp
        .RCC
        .constrain()
        .cfgr
        .use_hse(8.MHz())
        .sysclk(168.MHz())
        .freeze();

    // Initialize the four LED lights.
    let gpiod = dp.GPIOD.split();
    let green_led = gpiod.pd12.into_push_pull_output();
    let orange_led = gpiod.pd13.into_push_pull_output();
    let red_led = gpiod.pd14.into_push_pull_output();
    let blue_led = gpiod.pd15.into_push_pull_output();

    // ########################
    // # Part 2: Spawn a Task #
    // ########################
    //
    // A task is spawned by the task builder pattern as below. `set_entry`
    // is the only required method which configures the entry point of the new
    // task. Any closure that is `FnOnce + Send + 'static` can be the entry
    // point.
    //
    // Panicking inside a task will not hang the whole system. Instead, if the
    // task is started by `spwan()`, the panic will be caught and the task
    // gracefully terminated with resources reclaimed. Moreover, the panicked
    // task can also be automatically restarted, as we will demostrate in the
    // next part of the tutorial.

    task::build()
        .set_entry(move || blink_green(green_led))
        .spawn()
        .unwrap();

    fn blink_green(mut green_led: GreenLed) {
        // Define a barrier that allows a task to pass through every 500 ms.
        let mut barrier = IntervalBarrier::new(500).unwrap();

        loop {
            // The sleeping API causes the task to yield the CPU to other ready
            // tasks. If no application task is ready, the idle task will be
            // scheduled and put the CPU into low power mode with `wfe`
            // instruction.
            //
            // The effect of using `IntervalBarrier` is similar to simply calling
            // `hopter::time::sleep_ms()`. However, `sleep_ms` can slowly drift
            // away the interval when the system is under load.
            barrier.wait();

            green_led.toggle();
        }
    }

    // ####################################
    // # Part 3: Spawn a Restartable Task #
    // ####################################
    //
    // When the task's entry closure is also `Clone`, the `spawn_restartable()`
    // method will be available to spawn the task as a restartable one. In this
    // case, if the task panics, a new instance will be spawned automatically
    // and will start executing from the same entry closure. The old panicked
    // instance will be gracefully terminated with resources reclaimed.
    //
    // Hopter will attempt to spawn a new restarted instance before cleaning up
    // the old panicked instance. This is called concurrent restart, i.e., the
    // new instance and the cleaning up procedure of the old instance run
    // concurrently. The old instance will be reduced to a low priority, so
    // that the clean up procedure uses otherwise idle CPU time.
    //
    // Hopter performs concurrent restart under the following constraints:
    //
    // 1. The number of existing tasks is within the limit of the configuration.
    //    See `hopter::config::MAX_TASK_NUMBER`. If the number of tasks is
    //    already at the maximum, Hopter will not concurrently spawn a new
    //    instance. Instead, the new instance will start to run after cleaning
    //    up the old instance.
    // 2. At most two instances of a task can exist. One is the old panicked
    //    instance undergoing clean up. The other is the concurrently restarted
    //    instance. If the restarted instance panics again before the old
    //    instance is cleaned up, Hopter will not attempt to further
    //    concurrently spawn yet another new instance. The restart will happen
    //    after the second instance is cleaned up.

    // Move the LED behind an `Arc`, so that the entry closure becomes `Clone`.
    let orange_led = Arc::new(Mutex::new(orange_led));

    // Spawn the task as a restartable one.
    task::build()
        .set_entry(move || blink_orange(&mut *orange_led.lock()))
        .spawn_restartable()
        .unwrap();

    fn blink_orange(orange_led: &mut OrangeLed) {
        let mut barrier = IntervalBarrier::new(500).unwrap();
        let mut cnt = 0;

        loop {
            barrier.wait();
            orange_led.toggle();

            // Panic every 10 loop cycles. Since the task is restartable, the
            // LED appears to blink just as normal.
            cnt += 1;
            if cnt >= 10 {
                panic!();
            }
        }
    }

    // ##################################
    // # Part 4: Spawn a Breathing Task #
    // ##################################
    //
    // By default, tasks on Hopter run with segmented stacks, which is not a
    // contiguous memory chunk but rather small chunks allocated and freed on
    // demand. Segmented stacks allow the opportunity to time-multiplex the
    // stack memory usage among tasks. The breathing task is a sugar API that
    // simplify the memory time-multiplexing.
    //
    // A breathing task requires three closures upon definition: `init`, `wait`,
    // and `work`. The task will be constructed to look roughly like the
    // following:
    //
    // ```
    //     let mut state = init();
    //     loop {
    //         let item = wait(&mut state);
    //         work(&mut state, item);
    //     }
    // ```
    //
    // But more precisely, to smooth out stack memory usage among tasks and
    // avoid high peaks, the concurrency among breathing tasks is constrained.
    // Only a number of breathing tasks can run in the `work` function,
    // controlled by the `hopter::config::BREATHING_CONCURRENCY` parameter.
    //
    // Also, some inlining heuristics are applied to the functions of breathing
    // tasks to keep the stack usage low when the task is blocked.

    let red_led = Arc::new(Mutex::new(red_led));

    // Define a type of the `state`.
    struct BlinkRedCtxt {
        red_led: Arc<Mutex<RedLed>>,
        barrier: IntervalBarrier,
    }

    // A breathing task can also be restartable if all three closures are
    // `Clone`.
    task::build_breathing()
        .set_init(move || BlinkRedCtxt {
            red_led,
            barrier: IntervalBarrier::new(500).unwrap(),
        })
        .set_wait(|ctxt| ctxt.barrier.wait())
        .set_work(|ctxt, _| ctxt.red_led.lock().toggle())
        .spawn_restartable()
        .unwrap();

    // ################################################
    // # Part 5A: IRQs and Synchronization Primitives #
    // ################################################
    //
    // See Part 5B for more descriptions.

    // Initialize the TIM2 timer to trigger an IRQ every 500 ms.
    let mut timer = dp.TIM2.counter(&clocks);
    timer.listen(Event::Update);
    timer.start(500.millis()).unwrap();

    // Put the timer to the global variable so the IRQ handler can access it.
    *TIMER.lock() = Some(timer);

    // Set a priority TIM2 IRQ and unmask it.
    unsafe {
        cp.NVIC.set_priority(stm32f4xx_hal::pac::interrupt::TIM2, 0);
        cortex_m::peripheral::NVIC::unmask(stm32f4xx_hal::pac::interrupt::TIM2);
    }

    // Spawn a task that wait for the signal from the IRQ to blink the LED.
    task::build()
        .set_entry(|| blink_blue(blue_led))
        .spawn()
        .unwrap();

    fn blink_blue(mut blue_led: BlueLed) {
        loop {
            MAILBOX.wait();
            blue_led.toggle();
        }
    }

    // ##########################################
    // # Part 6: Protect Against Stack Overflow #
    // ##########################################
    //
    // Function call stacks can optionally have a size limit on them. If the
    // memory usage exceeds the limit, the task will be forcefully killed with
    // its resources reclaimed. The task will be restarted if it was spawned as
    // a restartable task.
    //
    // Technically, killing a task is accomplished by unwinding its function
    // call stack, regardless of the reason of killing, e.g., panic or stack
    // overflow. In case of a stack overflow, the function call causing the
    // overflow will be diverted to a `panic!()` call.
    //
    // Hopter further addresses an important corner case of the panic diversion.
    // If the task is running inside a drop handler when the stack hits the
    // size limit, the panic diversion is deferred until the drop handler
    // finishes. This is because an unwinding must not be initiated inside a
    // drop handler.

    task::build()
        // Set a stack size limit for the task.
        .set_stack_limit(4096)
        // Make the task higher priority than other tasks. Smaller numerical
        // value represents higher priority. If the task hangs up, it will
        // prevent other LED blinking tasks from running. But Hopter will
        // gracefully terminate this task so it will not have visible
        // effect on LED blinking.
        .set_priority(config::DEFAULT_TASK_PRIORITY - 1)
        // Attempt to overflow the stack by deep function recursion.
        .set_entry(|| {
            fibonacci(usize::MAX);
        })
        .spawn()
        .unwrap();

    fn fibonacci(x: usize) -> usize {
        if x >= 2 {
            fibonacci(x - 1).wrapping_add(fibonacci(x - 2))
        } else {
            x
        }
    }
}

// ################################################
// # Part 5B: IRQs and Synchronization Primitives #
// ################################################
//
// Hopter provides synchronization primitives under `hopter::sync`, which can
// be used for synchronization among tasks. A subset of them can be used also
// for synchronization between IRQs and tasks, including `Mailbox`, `Semaphore`,
// and `Channel`. The methods that allow calling from IRQ handler context are
// those with name ending in `allow_isr`.
//
// Some lock types with their names ending in `IrqSafe` are designed to use
// with IRQ handlers. When these locks are acquired, the corresponding IRQ will
// also be masked to avoid deadlock. The `irq!` macro generates types to
// represent IRQs, which can be passed to the `IrqSafe` locks.
//
// The handler of a specific IRQ is marked with `#[handler(IRQ_NAME)]`.
// Panicking inside a handler will not cause the system to hang, either. The
// handler will be forced to return with resources reclaimed.

// Generate the `Tim2Irq` type that represents the TIM2 IRQ.
irq!(Tim2Irq, stm32f4xx_hal::pac::interrupt::TIM2);

// The global `TIMER` variable is protected by the spin lock. TIM2 IRQ will be
// masked when the lock is acquired.
static TIMER: SpinIrqSafe<Option<CounterUs<TIM2>>, Tim2Irq> = SpinIrqSafe::new(None);

// Provide synchronization between the IRQ handler and the task.
static MAILBOX: Mailbox = Mailbox::new();

#[handler(TIM2)]
fn tim2_handler() {
    // Notify the `blink_blue` task.
    MAILBOX.notify_allow_isr();

    // Acknowledge the IRQ.
    TIMER.lock().as_mut().unwrap().wait().unwrap();
}
