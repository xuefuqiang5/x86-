#include "memory.h"
extern char _BSS_END;
uint32_t total_mem = 0x00;


Vpl vir_ker_pool, vir_usr_pool;
Ppl phy_ker_pool, phy_usr_pool;
/**
 * phy 的开始地址是 0x100000 + 页目录表大小（1） + 内核页表的大小（255） 
 * 之后的空闲空间 ker 与 usr 各分一半
 * phy kernel addr start 的位置是kernel free 的开始位置
 * phy use addr start 的位置是 use free 的开始位置
 * phy kernel bits 的位置是mem_bits标识的地方
 * phy usr bits 的位置是紧跟在phy kernel bits 位置的后面
*/
void mem_pool_init(uint32_t total_mem){
    uint32_t used_mem = 0 ;
    uint32_t free_mem = 0 ;
    uint32_t phy_ker_pool_base = 0x00;
    uint32_t phy_usr_pool_base = 0x00;
    uint32_t phy_ker_bmp_start = 0;
    uint32_t phy_usr_bmp_start = 0;
    uint32_t phy_ker_size = 0;
    uint32_t phy_usr_size = 0;
    uint32_t vir_ker_bmp_start = 0;
    uint32_t heap_base = 0;
    used_mem = 0x100000 + 256 * PAGE_SIZE;
    free_mem = total_mem - used_mem;
    phy_ker_size = free_mem / 2;
    phy_usr_size = free_mem - phy_ker_size;
    phy_ker_pool_base = used_mem;
    phy_usr_pool_base = phy_ker_pool_base + phy_ker_size;
    uint32_t ker_bitmap_byte_size = phy_ker_size / PAGE_SIZE / 8;
    uint32_t usr_bitmap_byte_size = phy_usr_size / PAGE_SIZE / 8; 
    phy_ker_bmp_start = bitmap_base;
    phy_usr_bmp_start = phy_ker_bmp_start + ker_bitmap_byte_size;  
    phy_ker_pool.addr_start = phy_ker_pool_base;
    phy_ker_pool.pool_size = phy_ker_size;
    phy_ker_pool.phy_bitmap.bits = (uint8_t *)(uintptr_t)phy_ker_bmp_start;
    phy_ker_pool.phy_bitmap.byte_size = ker_bitmap_byte_size;
    init_bitmap(&phy_ker_pool.phy_bitmap);
    phy_usr_pool.addr_start = phy_usr_pool_base;
    phy_usr_pool.pool_size = phy_usr_size;
    phy_usr_pool.phy_bitmap.bits = (uint8_t *)(uintptr_t)phy_usr_bmp_start;
    phy_usr_pool.phy_bitmap.byte_size = usr_bitmap_byte_size;
    init_bitmap(&phy_usr_pool.phy_bitmap);
    vir_ker_bmp_start = phy_usr_bmp_start + usr_bitmap_byte_size;
    heap_base = PAGE_ALIGN_UP(vir_ker_bmp_start + ker_bitmap_byte_size);
    vir_ker_pool.addr_start = heap_base;
    vir_ker_pool.vir_bitmap.byte_size = ker_bitmap_byte_size;
    vir_ker_pool.vir_bitmap.bits = (uint8_t *)(uintptr_t)vir_ker_bmp_start;
    init_bitmap(&vir_ker_pool.vir_bitmap);
    
}
void print_vpl(Vpl *vpl) {
    put_str("Vpl.addr_start: 0x\0");
    put_int_hex(vpl->addr_start);  // 打印 addr_start

    put_str("\nVpl.vir_bitmap.byte_size: 0x\0");
    put_int_hex(vpl->vir_bitmap.byte_size);  // 打印 Bitmap 的字节大小

    put_str("\nVpl.vir_bitmap.bits: 0x\0");
    put_int_hex((uint32_t)vpl->vir_bitmap.bits);  // 打印 bits 指针的地址
}
void print_ppl(Ppl *ppl) {
    put_str("\nPpl.addr_start: 0x\0");
    put_int_hex(ppl->addr_start);  // 打印 addr_start

    put_str("\nPpl.phy_bitmap.byte_size: 0x\0");
    put_int_hex(ppl->phy_bitmap.byte_size);  // 打印 Bitmap 的字节大小

    put_str("\nPpl.phy_bitmap.bits: 0x\0");
    put_int_hex((uint32_t)ppl->phy_bitmap.bits);  // 打印 bits 指针的地址

    put_str("\nPpl.pool_size: 0x\0");
    put_int_hex(ppl->pool_size);  // 打印内存池大小
}
void mem_init() { 
    total_mem = *(uint32_t *)(uintptr_t)0x8000;
    put_str("the total memory is: \n");
    put_int_hex(total_mem);
    put_char('\n');
    put_str("mem_init start\n\0"); 
    mem_pool_init(total_mem);        // 初始化内存池 
    put_str("mem_init done\n\0"); 
}
void *vir_allocate(uint32_t cnt, enum pool_flags pool_flag){
    if(pool_flag == PF_KERNEL){
        Bitmap *bmp = &vir_ker_pool.vir_bitmap;
        uint32_t start_idx = set_bits(bmp, cnt);
        return (void *)(vir_ker_pool.addr_start + start_idx * PAGE_SIZE);
    }
    if(pool_flag == PF_USER){
        return (void *)(uintptr_t)vir_usr_pool.addr_start;
    }
    return NULL;
}
void *phy_allocate(Ppl *pool){
    Bitmap *bmp = &pool->phy_bitmap;
    uint32_t start_idx = set_bits(bmp, 1);
    return (void *)(pool->addr_start + start_idx * PAGE_SIZE);
}
void page_register(void *vaddr, void *paddr){
    uint32_t *pde = (uint32_t *)(uintptr_t)((GET_PDE((uint32_t)(uintptr_t)vaddr) << 12) | 0xffc00000);
    uint32_t item = pde[0];
    if(!(item & PG_P_1)){
        item |= PG_P_1;
        item |= PG_RW_W;
        item |= PG_US_U;
        pde[0] = item;    
    }
    uint32_t *pte = (uint32_t *)(uintptr_t)((uint32_t)(uintptr_t)pde + (GET_PTE((uint32_t)(uintptr_t)vaddr) << 2));
    item = pte[0];
    if(!(item & PG_P_1)){
        item |= PG_P_1;
        item |= PG_RW_W;
        item |= PG_US_U;    
        pte[0] = item;
    }
    pte[0] |= (uint32_t)paddr & 0xfffff000;
}
//建立映射关系
void* page_allocate(uint32_t cnt, enum pool_flags pool_flag){
    void *vaddr =  vir_allocate(cnt, pool_flag);
    for(uint32_t i = 0,  var_vaddr = (uint32_t)vaddr; i < cnt; i++,  var_vaddr += PAGE_SIZE){
        void *paddr = phy_allocate(&phy_ker_pool);
        page_register((void *)var_vaddr, paddr);
    }
    return (void *)vaddr;
}
