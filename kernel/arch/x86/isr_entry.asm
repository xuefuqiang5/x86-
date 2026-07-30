
[bits 32]
%define ERROR_CODE nop
%define ZERO push 0

extern put_int_hex
extern idt_table

section .data
intr_str db "interrupt occur!", 0xa, 0
global intr_entry_table
intr_entry_table:

%macro VECTOR 2
section .text
intr%1entry:
    %2
    push ds
    push es
    push fs
    push gs
    pushad
%if %1 >= 0x20
    mov al, 0x20
%if %1 >= 0x28
    out 0xa0, al
%endif
    out 0x20, al
%endif
    push %1
    call [idt_table + %1*4]
    jmp intr_exit

section .data
    dd intr%1entry
%endmacro

VECTOR 0x00, ZERO
VECTOR 0x01, ZERO
VECTOR 0x02, ZERO
VECTOR 0x03, ZERO
VECTOR 0x04, ZERO
VECTOR 0x05, ZERO
VECTOR 0x06, ZERO
VECTOR 0x07, ZERO
VECTOR 0x08, ERROR_CODE
VECTOR 0x09, ZERO
VECTOR 0x0A, ERROR_CODE
VECTOR 0x0B, ERROR_CODE
VECTOR 0x0C, ERROR_CODE
VECTOR 0x0D, ERROR_CODE
VECTOR 0x0E, ERROR_CODE
VECTOR 0x0F, ZERO
VECTOR 0x10, ZERO
VECTOR 0x11, ERROR_CODE
VECTOR 0x12, ZERO
VECTOR 0x13, ZERO
VECTOR 0x14, ZERO
VECTOR 0x15, ZERO
VECTOR 0x16, ZERO
VECTOR 0x17, ZERO
VECTOR 0x18, ZERO
VECTOR 0x19, ZERO
VECTOR 0x1A, ZERO
VECTOR 0x1B, ZERO
VECTOR 0x1C, ZERO
VECTOR 0x1D, ZERO
VECTOR 0x1E, ZERO
VECTOR 0x1F, ZERO
VECTOR 0x20, ZERO
VECTOR 0x21,ZERO        ; 键盘中断（IRQ1）
VECTOR 0x22,ZERO        ; 级联中断（IRQ2）
VECTOR 0x23,ZERO        ; 串口2（IRQ3）
VECTOR 0x24,ZERO        ; 串口1（IRQ4）
VECTOR 0x25,ZERO        ; 并口2（IRQ5）
VECTOR 0x26,ZERO        ; 软盘控制器（IRQ6）
VECTOR 0x27,ZERO        ; 并口1（IRQ7）
VECTOR 0x28,ZERO        ; 实时时钟（IRQ8）
VECTOR 0x29,ZERO        ; 重定向（IRQ9）
VECTOR 0x2a,ZERO        ; 保留（IRQ10）
VECTOR 0x2b,ZERO        ; 保留（IRQ11）
VECTOR 0x2c,ZERO        ; PS/2鼠标（IRQ12）
VECTOR 0x2d,ZERO        ; FPU异常（IRQ13）
VECTOR 0x2e,ZERO        ; 硬盘（IRQ14）
VECTOR 0x2f,ZERO        ; 保留（IRQ15）

section .text
global intr_exit
intr_exit:
    add esp, 4
    popad
    pop gs
    pop fs
    pop es
    pop ds
    add esp, 4
    iretd

section .note.GNU-stack noalloc noexec nowrite progbits
