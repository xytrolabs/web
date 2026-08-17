# Zor OS Development Guide

Build bare-metal kernels and embedded systems with Zor → Rust.

## Quick Start

```bash
ZOR_NO_STD=1 zor build kernel.zor
# → produces kernel.o (ELF64 object, no libc, no std)
```

## Minimal Kernel

```zor
// kernel.zor
fun kernel_main int {
    var vga int = 753664  // 0xB8000 VGA text buffer

    // Write "ZOR" to top-left
    var p int = vga
    unsafe { p[0] = 90 | 1792 }
    p = p + 2
    unsafe { p[0] = 79 | 1792 }
    p = p + 2
    unsafe { p[0] = 82 | 1792 }

    loop {}
    return 0
}
```

## Building

### 1. Transpile
```bash
ZOR_NO_STD=1 zor build kernel.zor
# → kernel (ELF64 object file)
```

### 2. Link with bootloader
```bash
# Assemble boots trap (GAS syntax)
cc -c boot.S -o boot.o

# Link
ld.lld -T linker.ld -o kernel.bin boot.o kernel.o
```

### 3. Boot in QEMU
```bash
qemu-system-x86_64 -kernel kernel.bin
```

## Boot Assembly (GAS)

```asm
# boot.S — Multiboot2 header + 64-bit entry
.section .multiboot, "a"
.align 8
    .long 0xE85250D6       # magic
    .long 0                # arch
    .long mb2_end - mb2_start
    .long -(0xE85250D6 + 0 + (mb2_end - mb2_start))
    .short 0; .short 0; .long 8
mb2_end:

# PVH ELF Note (for QEMU direct boot)
.pushsection .note, "a"
.align 4
    .long 2f - 1f; .long 4f - 3f; .long 18
1:  .asciz "Xen"
2:  .align 4
3:  .long _start
4:  .align 4
.popsection

.text
.code64
.global _start
_start:
    mov $stack_top, %rsp
    call kernel_main
.halt: cli; hlt; jmp .halt

.bss
.align 16
stack_bottom: .space 65536
stack_top:
```

## Linker Script

```ld
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
```

## Port I/O

```zor
// Add these helpers to your boot.S:
// outb:  mov %edi, %edx; mov %esi, %eax; outb %al, %dx; ret
// inb:   mov %edi, %edx; xor %eax, %eax; inb %dx, %al; ret

extern "C" {
    fun outb(port int, val int)
    fun inb(port int) -> int
}

fun pic_remap() {
    outb(0x20, 0x11)
    outb(0xA0, 0x11)
    outb(0x21, 0x20)
    outb(0xA1, 0x28)
    outb(0x21, 0x04)
    outb(0xA1, 0x02)
    outb(0x21, 0x01)
    outb(0xA1, 0x01)
}
```

## Memory Management

```zor
var heap_start int = 0x200000   // 2MB
var heap_current int = 0x200000

fun kmalloc(size int) int {
    var ptr int = heap_current
    heap_current = heap_current + size
    return ptr
}
```

## What Works in no_std

| Feature | Status |
|---|---|
| Functions, variables | ✅ |
| Control flow | ✅ |
| Structs, enums | ✅ |
| `unsafe` raw pointers | ✅ |
| `extern "C"` FFI | ✅ |
| `&T`, `&mut T` refs | ✅ |
| `match` + patterns | ✅ |
| String literals (`&str`) | ✅ |
| `loop`, `while` | ✅ |
| `return`, `stop`, `next` | ✅ |

## What's Different in no_std

| Feature | std mode | no_std mode |
|---|---|---|
| `string` type | `String` | `&str` |
| `say` output | `println!` | your `print()` function |
| `main` function | auto-wrapped | you define entry |
| Runtime helpers | included | skipped |
