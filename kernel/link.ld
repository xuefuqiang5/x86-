ENTRY(main)

SECTIONS
{
    . = 0xc0001500;

    .text : {
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data : {
        *(.data .data.*)
    }

    .bss : {
        *(.bss .bss.*)
        PROVIDE(_BSS_END = .);  
    }
}
