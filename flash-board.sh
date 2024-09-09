#!/bin/bash
set -e

# Generate a binary image from the compiled ELF file.
arm-none-eabi-objcopy -O binary --pad-to 0 --remove-section=.bss $1 hopter-quick-start.bin

# Flash the binary to the board.
st-flash write hopter-quick-start.bin 0x8000000
