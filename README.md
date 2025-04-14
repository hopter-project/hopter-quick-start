<img src="./.github/assets/hopter-logo.png" alt="Hopter's Logo" width="400"/>

# Hopter Quick Start Guide

This tutorial demonstrates the key features of the [Hopter](https://github.com/hopter-project/hopter) embedded operating system by blinking the four LEDs on a development board.
It covers essential topics, including:

- Project setup
- System initialization and the `main` task
- Normal, restartable, and breathing tasks
- Interrupt handling (IRQ)
- Synchronization primitives
- Panic and stack overflow protection

The source code `src/main.rs` includes detailed explanations for each topic.

This guide also serves as a good starting point for building your own projects.

## Choose a Board

The tutorial currently supports four boards:
- [STM32F407G-Discovery (Cortex-M4)](https://www.st.com/en/evaluation-tools/stm32f4discovery.html)
- [STM32F411E-Discovery (Cortex-M4)](https://www.st.com/en/evaluation-tools/32f411ediscovery.html)
- [STM32F412G-Discovery (Cortex-M4)](https://www.st.com/en/evaluation-tools/32f412gdiscovery.html)
- [STM32F072B-Discovery (Cortex-M0)](https://www.st.com/en/evaluation-tools/32f072bdiscovery.html)

There is nothing to do if the code runs with an STM32F407G-Discovery board.

For other boards, apply the corresponding patch to the source code.
The patch rewrites a few configuration parameters and slightly changes the API call to the HAL library to match the chosen board.

For example with F411, run the following command.

```
patch -p1 < stm32f411-discovery.patch
```

## Install Prerequisite

### Rust Compiler

Hopter requires a customized Rust compiler toolchain. The easiest way to get it is by downloading a prebuilt version for your system. Follow the instructions [here](https://github.com/hopter-project/hopter-compiler-toolchain).

### OpenOCD

The tutorial code uses OpenOCD to flash the board.

For MacOS, run `brew install open-ocd`.

For Ubuntu, run `apt install openocd`.

## Flash the Board

Run `cargo build --release` to compile the code. Run `cargo run --release` to flash the board.
