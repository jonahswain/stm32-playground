/* Linker script for the STM32F767ZI */
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 2048K
    RAM : ORIGIN = 0x20000000, LENGTH = 512K
}


/* Full memory layout

MEMORY
{
    FLASH (rx)        :   ORIGIN = 0x08000000, LENGTH = 2048K
    ITCMRAM (rw)      :   ORIGIN = 0x00000000, LENGTH = 16K
    DTCMRAM (rw)      :   ORIGIN = 0x20000000, LENGTH = 128K
    SRAM1 (rw)        :   ORIGIN = 0x20020000, LENGTH = 368K
    SRAM2 (rw)        :   ORIGIN = 0x2007C000, LENGTH = 16K
    BKPSRAM (rw)      :   ORIGIN = 0x40024000, LENGTH = 4K
}
*/