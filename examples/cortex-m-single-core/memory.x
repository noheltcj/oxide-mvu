/* Memory layout for Cortex-M single-core example (nRF52840) */
MEMORY
{
  /* Flash memory - 1MB (nRF52840) */
  FLASH : ORIGIN = 0x00000000, LENGTH = 1M

  /* RAM - 256KB (nRF52840) */
  RAM : ORIGIN = 0x20000000, LENGTH = 256K
}

/* Define the entry point */
ENTRY(Reset);

/* Define the stack size */
_stack_size = 16K;
