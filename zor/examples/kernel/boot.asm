; Zor Kernel — x86-64 Multiboot2 boot entry
; Assembles with NASM: nasm -f elf64 boot.asm -o boot.o

section .multiboot
align 8
mb2_header_start:
    dd 0xE85250D6                ; magic
    dd 0                         ; architecture (i386 protected mode)
    dd mb2_header_end - mb2_header_start  ; header length
    dd 0x100000000 - (0xE85250D6 + 0 + (mb2_header_end - mb2_header_start))  ; checksum
    ; Framebuffer tag
    dw 5                         ; type: framebuffer
    dw 0                         ; flags
    dd 20                        ; size
    dd 1024                      ; width
    dd 768                       ; height
    dd 32                        ; depth
mb2_header_end:

section .text
global _start
extern kernel_main

_start:
    ; Stack setup
    mov rsp, stack_top

    ; Reset EFLAGS
    push 0
    popf

    ; Save multiboot info pointer (rdi = multiboot magic, rsi = multiboot info)
    ; We pass them transparently — kernel_main can use them later
    mov rdi, rax    ; multiboot magic
    mov rsi, rbx    ; multiboot info pointer

    ; Enable SSE
    mov rax, cr0
    and ax, 0xFFFB
    or ax, 0x2
    mov cr0, rax
    mov rax, cr4
    or ax, 3 << 9
    mov cr4, rax

    ; Call kernel
    call kernel_main

    ; Halt if kernel returns
.halt:
    cli
    hlt
    jmp .halt

section .bss
align 16
stack_bottom:
    resb 65536   ; 64 KiB stack
stack_top:
