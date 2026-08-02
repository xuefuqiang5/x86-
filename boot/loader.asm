%include "boot.inc"

section load vstart=0x900
jmp loader_start
db 0x00
GDT_ADDR:
    GDT_BASE:
        dd 0x00000000
        dd 0x00000000
    CODE_GDT:
        dd 0x0000ffff
        dd 0x00cf9800
    DATA_GDT:
        dd 0x0000ffff
        dd 0x00cf9200
    VIDEO_GDT:
        dd 0x8000ffff
        dd 0x0040920b
    RING3_CODE_DESC:
        dd 0x0000ffff
        dd 0x00cffa00
    RING3_DATA_DESC:
        dd 0x0000ffff
        dd 0x00cff200
    TSS_DESC:
        dq 0
    gdt_ptr:
        dw $ - GDT_ADDR - 1
        dd GDT_ADDR
    ards_buf times 254 db 0 
    ards_count dd 0 
detected_memory:
    xor ebx, ebx 
    xor eax, eax
    mov es, ax 
    mov edi, ards_buf
    mov edx, 0x534d4150 
.probe_loop:
        mov eax, 0xe820
        mov ecx, 20
        int 0x15
        jc .probe_end
        cmp eax, 0x534d4150
        jne .probe_end
        add di, cx
        inc dword [es:ards_count]
        test ebx, ebx
        jnz .probe_loop
.probe_end:
.culculate_memory:
        mov word cx, [es:ards_count]
        mov edi, ards_buf
        xor eax, eax
        test cx, cx
        jz .calculate_done
        push eax
.cmp_loop:
            mov eax, [es:edi]
            mov edx, [es:edi + 8]
            add eax, edx
            pop edx
            push edx
            cmp eax, edx
            jg .push_eax
            jle .next
            .push_eax:
                pop ebx
                push eax
            .next:
                add edi, 20 
                loop .cmp_loop
            pop eax 
.calculate_done:
            ret
    
loader_start:
    call detected_memory
    mov [es:total_mem_addr], eax
    mov dx, 0x3f8
    mov al, 'L'
    out dx, al
    mov eax, 0x00
    mov ds, eax
    in al, 0x92
    or al, 2
    out 0x92, al
    lgdt [gdt_ptr]
    mov eax, cr0 
    or eax, 0x00000001 
    mov cr0, eax 
    jmp dword CODE_SELECTOR:p_mode_start    

[bits 32]
p_mode_start:
    mov dx, 0x3f8
    mov al, 'P'
    out dx, al
    mov ax, DATA_SELECTOR 
    mov ds, ax 
    mov es, ax 
    mov ss, ax 
    mov esp, 0x7c00
    mov ebp, esp
    mov ax, VIDEO_SELECTOR 
    mov gs, ax 
    mov esi, msg
    call print_string
    mov dx, 0x3f8
    mov al, 'D'
    out dx, al

    mov eax, KERNEL_START_SECTOR
    mov ebx, KERNEL_START_ADDR
    mov ecx, KERNEL_SECTOR_COUNT
    call read_disk_m_32
    mov dx, 0x3f8
    mov al, 'E'
    out dx, al
    call setup_page_table
    sgdt [gdt_ptr]
    mov ebx, [gdt_ptr + 2]
    or dword [ebx + 0x18 + 4], 0xc0000000
    add esp, 0xc0000000
    mov eax, PAGE_DIR_TABLE_POS
    mov cr3, eax
    mov eax, cr0
    or eax, 0x80000000
    mov cr0, eax
    lgdt [gdt_ptr]
    jmp CODE_SELECTOR:enter_kernel
enter_kernel:
    call kernel_init
    mov dx, 0x3f8
    mov al, 'X'
    out dx, al
    mov eax, VIDEO_SELECTOR
    mov gs, eax
    mov esi, msg1
    call print_string
    mov esp, 0xc009f000
    mov dx, 0x3f8
    mov al, 'Y'
    out dx, al
    jmp [vstart]

setup_page_table:
    pushad

    ; Clear all 1024 page-directory entries.
    mov edi, PAGE_DIR_TABLE_POS
    xor eax, eax
    mov ecx, 1024
    rep stosd

    ; Clear the complete first page table. Only its first 256 entries are
    ; populated below; the remaining entries are reserved for mappings that
    ; the Rust memory manager creates on demand.
    mov edi, PAGE_DIR_TABLE_POS + 0x1000
    xor eax, eax
    mov ecx, 1024
    rep stosd

    ; Identity-map the first MiB and mirror it at 0xc0000000.
    mov eax, PAGE_DIR_TABLE_POS + 0x1000
    or eax, PG_US_S | PG_RW_W | PG_P
    mov [PAGE_DIR_TABLE_POS], eax
    mov [PAGE_DIR_TABLE_POS + 0xc00], eax

    ; Make the final PDE recursively map the page directory itself.
    mov eax, PAGE_DIR_TABLE_POS
    or eax, PG_US_S | PG_RW_W | PG_P
    mov [PAGE_DIR_TABLE_POS + 4092], eax

    mov edi, PAGE_DIR_TABLE_POS + 0x1000
    mov eax, PG_US_S | PG_RW_W | PG_P
    mov ecx, 256
.create_identity_pte:
    stosd
    add eax, 4096
    loop .create_identity_pte

    popad
    ret
read_disk_m_32:
    pushad
    mov esi, eax
    mov edi, ebx
    mov ebp, ecx

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
    jnz .disk_error
    test al, 0x08
    jz .not_ready

    mov ecx, 256
    mov dx, 0x1f0
.read_word:
    in ax, dx
    mov [edi], ax
    add edi, 2
    loop .read_word

    mov dx, 0x1f7
.finish_sector:
    in al, dx
    test al, 0x80
    jnz .finish_sector
    test al, 0x01
    jnz .disk_error
    test al, 0x08
    jnz .finish_sector

    inc esi
    dec ebp
    jnz .next_sector
    popad
    ret

.disk_error:
    mov dx, 0x3f8
    mov al, '!'
    out dx, al
    cli
    hlt
    jmp .disk_error

print_string:
    push eax
    push edi
    mov edi, 160
    .print_loop:
        lodsb
        cmp al, 0
        je print_end
        mov byte [gs:di], al
        inc di
        mov byte [gs:di], 0xa4
        inc di 
    jmp .print_loop
    print_end:
    pop edi
    pop eax
    ret
   
kernel_init:
    xor eax, eax
    xor ebx, ebx
    xor ecx, ecx
    xor edx, edx
    xor esi, esi
    xor edi, edi
    mov ebx, KERNEL_START_ADDR
    mov edx, KERNEL_START_ADDR
    mov cx, [ebx + E_PHNUM]
    mov eax, [ebx + E_PHOFF]
    add ebx, eax
    mov esi, 0
    .load_segment:
        mov eax, [ebx + esi + P_TYPE]
        cmp eax, PT_LOAD
        jne .next
        push ecx 
        mov ecx, [ebx + esi + P_FILESZ]
        mov eax, [ebx + esi + P_OFFSET]
        add eax, edx
        mov edi, [ebx + esi + P_VADDR]
        call memcpy
        mov edi, [ebx + esi + P_VADDR]
        add edi, [ebx + esi + P_FILESZ]
        mov ecx, [ebx + esi + P_MEMSZ]
        sub ecx, [ebx + esi + P_FILESZ]
        xor eax, eax
        rep stosb
        pop ecx
        .next:
            add esi, 32
            loop .load_segment
        mov eax, [edx + E_ENTRY]
        mov [vstart], eax
        ret
memcpy:
    push esi
    push edi
    mov esi, eax
    rep movsb
    pop edi
    pop esi
    ret



msg db "protect mode", 0
msg1 db "kernel has been loaded!", 0
vstart dd 0x00000000
