ENTRY(_start)
PHDRS { text PT_LOAD; note PT_NOTE; }
SECTIONS {
    . = 1M;
    .multiboot : { KEEP(*(.multiboot)) } :text
    .note : { *(.note) } :note
    .text : { *(.text .text.*) } :text
    .rodata : { *(.rodata .rodata.*) } :text
    .data : { *(.data .data.*) } :text
    .bss : { *(COMMON) *(.bss .bss.*) } :text
    /DISCARD/ : { *(.eh_frame) *(.comment) }
}
