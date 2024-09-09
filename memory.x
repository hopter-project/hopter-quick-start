/* This is the memory layout for STM32F4-Discovery board. */

MEMORY
{
  /* SRAM 0x20000000 to 0x20001000 is reserved for the contiguous stack. */
  /* The rest of SRAM goes from 0x20001000 to 0x20020000. */
  RAM (xrw) : ORIGIN = 0x20001000, LENGTH = 126976
  CCMRAM (xrw) : ORIGIN = 0x10000000, LENGTH = 64K
  FLASH (rx) : ORIGIN = 0x8000000, LENGTH = 1024K
  /* NOTE 1 K = 1 KiB = 1024 bytes */
}

/* This is where the contiguous call stack will be allocated. */
/* The stack grows downward. */
__contiguous_stack_start = ORIGIN(RAM);
