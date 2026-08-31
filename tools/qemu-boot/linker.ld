ENTRY(_start)

MEMORY
{
    RAM (rwx) : ORIGIN = 0x40100000, LENGTH = 2M
}

SECTIONS
{
    . = 0x40100000;

    .text : {
        KEEP(*(.text._start))
        *(.text .text.*)
    } > RAM

    .rodata : {
        *(.rodata .rodata.*)
    } > RAM

    .data : {
        *(.data .data.*)
    } > RAM

    .bss (NOLOAD) : {
        . = ALIGN(16);
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    } > RAM

    . = ALIGN(16);
    .stack (NOLOAD) : {
        __stack_bottom = .;
        . += 0x10000;
        __stack_top = .;
    } > RAM

    /DISCARD/ : {
        *(.comment)
        *(.eh_frame)
    }
}
