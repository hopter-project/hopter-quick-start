#!/bin/bash
set -e

# Flash the binary to the board
openocd -f interface/stlink.cfg -f target/stm32f4x.cfg -c "program $1 verify reset exit"
