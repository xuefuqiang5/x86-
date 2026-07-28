 LOADER_START_SECTION equ 1
 LOADER_START_ADDR equ 0x900
 CODE_SELECTOR equ 0x8
 DATA_SELECTOR equ 0x10
 VIDEO_SELECTOR equ 0x18
 PAGE_DIR_TABLE_POS equ 0x10000
 PG_P  equ   1b 
 PG_RW_R  equ  00b 
 PG_RW_W equ  10b 
 PG_US_S  equ  000b 
 PG_US_U  equ  100b

KERNEL_START_SECTOR equ 0x9 
KERNEL_START_ADDR equ 0x70000
 
 
E_IDENT     equ 0x00
E_TYPE      equ 0x10
E_MACHINE   equ 0x12
E_VERSION   equ 0x14
E_ENTRY     equ 0x18
E_PHOFF     equ 0x1C
E_SHOFF     equ 0x20
E_FLAGS     equ 0x24
E_EHSIZE    equ 0x28
E_PHENTSIZE equ 0x2A
E_PHNUM     equ 0x2C
E_SHENTSIZE equ 0x2E
E_SHNUM     equ 0x30
E_SHSTRNDX  equ 0x32

P_TYPE      equ 0x00    ; p_type offset
P_OFFSET    equ 0x04    ; p_offset offset
P_VADDR     equ 0x08    ; p_vaddr offset
P_PADDR     equ 0x0C    ; p_paddr offset
P_FILESZ    equ 0x10    ; p_filesz offset
P_MEMSZ     equ 0x14    ; p_memsz offset
P_FLAGS     equ 0x18    ; p_flags offset
P_ALIGN     equ 0x1C    ; p_align offset


PT_LOAD equ 1
total_mem_addr equ 0x8000
