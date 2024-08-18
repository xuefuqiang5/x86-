
SECTION text 
    org 0x7C00
    align 16

start:
    ; 初始化堆栈段
    cli                ; 禁用中断
    xor ax, ax         ; ax = 0
    mov ds, ax         ; 将DS和SS都设置为数据段基地址（假设都从0x0000开始）
    mov ss, ax
    mov sp, ax ; 初始化堆栈指针
    sti                ; 启用中断

    ; BIOS: 设置文本模式
    mov ah, 0x00
    mov al, 0x03
    int 0x10

    ; 输出Loading_info字符串
    mov si, Loading_info
    call put_char
    jmp load_disk
put_char:
    push ax 
.put_loop:
    lodsb               ; 加载下一个字节 (SI -> AL)
    cmp al, 0           ; 检查是否到达字符串结尾
    je .done
    mov ah, 0x0E        ; BIOS 功能号: 输出字符
    int 0x10            ; 调用 BIOS 中断
    jmp .put_loop       ; 继续循环输出
.done: 
    pop ax
    ret
load_disk:
    mov si, DAPA
    mov bx, si
    mov ah, 0x42
    mov dl, 0x80
    int 0x13

    jc error 
    jmp success                         
    
error:
    mov ah, 0x0E      ; BIOS teletype output function
    mov al, 0x0D      ; Carriage Return (CR)
    int 0x10          ; Call BIOS interrupt
    mov al, 0x0A      ; Line Feed (LF)
    int 0x10          ; Call BIOS interrupt
    
    
    mov si, error_process
    call put_char
    jmp _end 
success:
    mov ah, 0x0E      ; BIOS teletype output function
    mov al, 0x0D      ; Carriage Return (CR)
    int 0x10          ; Call BIOS interrupt
    mov al, 0x0A      ; Line Feed (LF)
    int 0x10          ; Call BIOS interrupt
    
    
    mov si, success_process
    call put_char
    jmp _end
_end jmp _end
Loading_info db 'Welcome to this loading procedure', 0
First_confirm db 'the preceding address loads successfully', 0
error_process db 'loading disk has certain mistake , please try again!', 0
success_process db 'Congratulations!',0
DAPA:
    db 16        ; Size of DAPA (16 bytes)
    db 0         ; Reserved, must be 0
    dw 1          ; Number of sectors to read
    dw 0x0000    ; Offset address of the buffer to store data
    dw 0x1000   ; Segment address of the buffer to store data
    dq 0x64       ; Logical Block Address (LBA) to start reading from


times 510 -($-$$) DB 0x00
db 0x55,0xaa
