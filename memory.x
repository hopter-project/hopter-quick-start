/* This is the memory layout for STM32F407-Discovery board. */

MEMORY
{
  /* NOTE 1 K = 1 KiB = 1024 bytes */
  RAM (xrw) : ORIGIN = 0x20000000, LENGTH = 128K
  FLASH (rx) : ORIGIN = 0x8000000, LENGTH = 1024K
}

/* Length of the contiguous stack placed at the beginning of the RAM region.
   The value must match the one in Hopter configuration parameters. */
_contiguous_stack_length = 0x1000;
