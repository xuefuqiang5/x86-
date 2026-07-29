[bits 32]
section .text
global ex_write
ex_write:
  push ebp 
  mov ebp, esp
  mov eax, [ebp + 8]
  mov ecx, [ebp + 0xc]
  mov edx, [ebp + 0x10]
  mov al, [gs:eax]
  mov [gs:ecx], al
  mov byte [gs:edx], 0x0f
  pop ebp
  ret 

global write_one_char
write_one_char:
  push ebp
  mov ebp, esp
  mov eax, [ebp + 8]
  mov cl, [ebp + 0xc]
  mov [gs:eax], cl
  inc eax
  mov [gs:eax], byte 0x0f
  pop ebp
  ret


  global lidt
lidt:
  push ebp
  mov ebp, esp
  mov eax, [ebp+8]
  lidt [eax]
  pop ebp
  ret
  
global intr_enable
  intr_enable:
    push ebp 
    mov ebp, esp
    pushf 
    sti
    pop eax
    and eax, 0x200
    pop ebp 
    ret  
global intr_disable
  intr_disable:
    push ebp 
    mov ebp, esp
    pushf 
    cli
    pop eax
    and eax, 0x200
    pop ebp 
    ret  
