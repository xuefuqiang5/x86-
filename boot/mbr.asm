%include "boot.inc"
section mbr vstart=0x7c00 
jmp code_start   
code_start:
   cli
   xor ax, ax
   mov ss, ax 
   mov sp, 0x7c00
   
    mov ax, 0x600
    mov bx, 0x700
    mov cx, 0
    mov dx, 0x184f
    
    int 0x10
 print_char:
    mov ax, 0xb800
    mov es, ax
    mov si, message
    mov di, 0 
    xor ax, ax
    mov ds, ax
  print_loop:
    ds lodsb
    cmp al, 0
    je print_end
    mov byte [es:di], al
    inc di
    mov byte [es:di], 0xa4
    inc di 
    jmp print_loop
 print_end:

read_sector_to_memory:
    xor eax, eax
    mov esi, LOADER_START_SECTION
    mov bp, 3
    xor ax, ax
    mov es, ax
    mov bx, LOADER_START_ADDR

.next_sector:
    mov al, 1
    mov dx, 0x1f2
    out dx, al

    mov eax, esi
    mov dx, 0x1f3
    out dx, al
    shr eax, 8
    mov dx, 0x1f4
    out dx, al
    shr eax, 8
    mov dx, 0x1f5
    out dx, al
    shr eax, 8
    mov dx, 0x1f6
    and al, 0x0f
    or al, 0xe0
    out dx, al

    mov dx, 0x3f6
    in al, dx
    in al, dx
    in al, dx
    in al, dx

    mov dx, 0x1f7
    mov al, 0x20
    out dx, al

.not_ready:
    in al, dx
    test al, 0x80
    jnz .not_ready
    test al, 0x01
    jnz disk_error
    test al, 0x08
    jz .not_ready

    mov cx, 256
    mov dx, 0x1f0
.read_word:
    in ax, dx
    mov [es:bx], ax
    add bx, 2
    loop .read_word

    mov dx, 0x1f7
.finish_sector:
    in al, dx
    test al, 0x80
    jnz .finish_sector
    test al, 0x01
    jnz disk_error
    test al, 0x08
    jnz .finish_sector

    inc esi
    dec bp
    jnz .next_sector

   push 0x0000
   push LOADER_START_ADDR
   retf 

disk_error:
   mov dx, 0x3f8
   mov al, '!'
   out dx, al
   cli
   hlt
   jmp disk_error

message db "Welcome to my OS!", 0

times 510-($-$$) db 0
db 0x55, 0xaa
