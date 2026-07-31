.section .multiboot, "a"
.align 8
mb2_start:
    .long 0xE85250D6; .long 0
    .long mb2_end - mb2_start
    .long -(0xE85250D6 + 0 + (mb2_end - mb2_start))
    .short 0; .short 0; .long 8
mb2_end:

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
.global _start, outb, inb
_start:
    mov $stack_top, %rsp
    call kernel_main
.halt: cli; hlt; jmp .halt

# void outb(int port, int val) — serial output helper
outb:
    mov %edi, %edx
    mov %esi, %eax
    outb %al, %dx
    ret

# int inb(int port)
inb:
    mov %edi, %edx
    xor %eax, %eax
    inb %dx, %al
    ret

.bss
.align 16
stack_bottom: .space 65536
stack_top:
