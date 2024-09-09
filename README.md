# Hopter Quick Start Guide

The tutorial demonstrates the key features of the [Hopter](https://github.com/ZhiyaoMa98/hopter) embedded operating system by blinking the four LEDs on [STM32F407-Discovery](https://www.st.com/en/evaluation-tools/stm32f4discovery.html) board. It covers essential topics, including:

- Project setup
- System initialization and the `main` task
- Normal, restartable, and breathing tasks
- Interrupt handling (IRQ)
- Synchronization primitives
- Panic and stack overflow protection

The source code `src/main.rs` includes detailed explanations for each topic.

This guide also serves as a good starting point for building your own projects.

## Prerequisite Installation

### Rust Compiler

Hopter requires a customized Rust compiler toolchain. The easiest way to get it is by downloading a prebuilt version for your system. Follow the instructions [here](https://github.com/ZhiyaoMa98/hopter-compiler-toolchain).

### Arm GNU Toolchain

The tool `arm-none-eabi-objcopy` is needed to prepare the binary for flashing to the board.

- **MacOS**: Install using the following command:
  ```
  brew install --cask gcc-arm-embedded
  ```
- **Ubuntu**: Download the toolchain from [here](https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads). Ensure the executable is included in your `PATH`.

### ST-Link

`st-link` is used to flash the binary onto the board.

- **MacOS**: Install using the following command:
  ```
  brew install stlink
  ```
- **Ubuntu**: Install using the following command:
  ```
  sudo apt install stlink-tools
  ```

## Flash the Board

Run `cargo build --release` to compile the code. Run `cargo run --release` to flash the board.
