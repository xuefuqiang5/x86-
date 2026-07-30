use crate::ds::bitmap::Bitmap;

pub const PAGE_SIZE: u32 = 4096;
pub const PG_P_1: u32 = 1;
pub const PG_P_0: u32 = 0;
pub const PG_RW_R: u32 = 0;
pub const PG_RW_W: u32 = 2;
pub const PG_US_S: u32 = 0;
pub const PG_US_U: u32 = 4;

const BITMAP_BASE: u32 = 0xc009a000;

const fn page_align_up(x: u32) -> u32 {
    (x + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

fn get_pde(vaddr: u32) -> u32 {
    (vaddr >> 22) & 0x3ff
}

fn get_pte(vaddr: u32) -> u32 {
    (vaddr >> 12) & 0x3ff
}

#[repr(C)]
pub struct VirMemoryPool {
    pub addr_start: u32,
    pub vir_bitmap: Bitmap,
}

#[repr(C)]
pub struct PhyMemoryPool {
    pub addr_start: u32,
    pub phy_bitmap: Bitmap,
    pub pool_size: u32,
}

pub type Vpl = VirMemoryPool;
pub type Ppl = PhyMemoryPool;

pub const PF_KERNEL: u32 = 1;
pub const PF_USER: u32 = 2;

static mut TOTAL_MEM: u32 = 0;
static mut VIR_KER_POOL: VirMemoryPool = VirMemoryPool {
    addr_start: 0,
    vir_bitmap: Bitmap { byte_size: 0, bits: core::ptr::null_mut() },
};
static mut VIR_USR_POOL: VirMemoryPool = VirMemoryPool {
    addr_start: 0,
    vir_bitmap: Bitmap { byte_size: 0, bits: core::ptr::null_mut() },
};
static mut PHY_KER_POOL: PhyMemoryPool = PhyMemoryPool {
    addr_start: 0,
    phy_bitmap: Bitmap { byte_size: 0, bits: core::ptr::null_mut() },
    pool_size: 0,
};
static mut PHY_USR_POOL: PhyMemoryPool = PhyMemoryPool {
    addr_start: 0,
    phy_bitmap: Bitmap { byte_size: 0, bits: core::ptr::null_mut() },
    pool_size: 0,
};

pub fn mem_pool_init(total_mem: u32) {
    let used_mem: u32 = 0x100000 + 256 * PAGE_SIZE;
    let free_mem: u32 = total_mem.saturating_sub(used_mem);
    let phy_ker_size = free_mem / 2;
    let phy_usr_size = free_mem - phy_ker_size;
    let phy_ker_pool_base = used_mem;
    let phy_usr_pool_base = phy_ker_pool_base + phy_ker_size;
    let ker_bitmap_byte_size = phy_ker_size / PAGE_SIZE / 8;
    let usr_bitmap_byte_size = phy_usr_size / PAGE_SIZE / 8;
    let phy_ker_bmp_start = BITMAP_BASE;
    let phy_usr_bmp_start = phy_ker_bmp_start + ker_bitmap_byte_size;
    let vir_ker_bmp_start = phy_usr_bmp_start + usr_bitmap_byte_size;
    let heap_base = page_align_up(vir_ker_bmp_start + ker_bitmap_byte_size);

    unsafe {
        PHY_KER_POOL.addr_start = phy_ker_pool_base;
        PHY_KER_POOL.pool_size = phy_ker_size;
        PHY_KER_POOL.phy_bitmap.bits = phy_ker_bmp_start as *mut u8;
        PHY_KER_POOL.phy_bitmap.byte_size = ker_bitmap_byte_size;
        crate::ds::bitmap::init_bitmap(core::ptr::addr_of!(PHY_KER_POOL.phy_bitmap));

        PHY_USR_POOL.addr_start = phy_usr_pool_base;
        PHY_USR_POOL.pool_size = phy_usr_size;
        PHY_USR_POOL.phy_bitmap.bits = phy_usr_bmp_start as *mut u8;
        PHY_USR_POOL.phy_bitmap.byte_size = usr_bitmap_byte_size;
        crate::ds::bitmap::init_bitmap(core::ptr::addr_of!(PHY_USR_POOL.phy_bitmap));

        VIR_KER_POOL.addr_start = heap_base;
        VIR_KER_POOL.vir_bitmap.byte_size = ker_bitmap_byte_size;
        VIR_KER_POOL.vir_bitmap.bits = vir_ker_bmp_start as *mut u8;
        crate::ds::bitmap::init_bitmap(core::ptr::addr_of!(VIR_KER_POOL.vir_bitmap));

        // User virtual memory is not implemented yet. Keep this pool empty
        // instead of aliasing and clearing the kernel virtual bitmap.
        VIR_USR_POOL.addr_start = 0;
        VIR_USR_POOL.vir_bitmap.byte_size = 0;
        VIR_USR_POOL.vir_bitmap.bits = core::ptr::null_mut();
    }
}

pub fn mem_init() {
    unsafe {
        use crate::vga::{put_char, put_int_hex, put_str};

        TOTAL_MEM = *(0x8000 as *const u32);
        put_str(b"the total memory is: \n\0".as_ptr());
        put_int_hex(TOTAL_MEM);
        put_char(b'\n');
        put_str(b"mem_init start\n\0".as_ptr());
        mem_pool_init(TOTAL_MEM);
        put_str(b"mem_init done\n\0".as_ptr());
    }
}

pub fn vir_allocate(cnt: u32, pool_flag: u32) -> *mut core::ffi::c_void {
    if cnt == 0 {
        return core::ptr::null_mut();
    }
    unsafe {
        if pool_flag == PF_KERNEL {
            let start_idx = (*core::ptr::addr_of!(VIR_KER_POOL)).vir_bitmap.set_bits(cnt);
            if start_idx < 0 {
                return core::ptr::null_mut();
            }
            ((*core::ptr::addr_of!(VIR_KER_POOL)).addr_start as usize + start_idx as usize * PAGE_SIZE as usize) as *mut core::ffi::c_void
        } else {
            core::ptr::null_mut()
        }
    }
}

pub fn phy_allocate(pool: *const PhyMemoryPool) -> *mut core::ffi::c_void {
    let start_idx = unsafe { (*pool).phy_bitmap.set_bits(1) };
    if start_idx < 0 {
        return core::ptr::null_mut();
    }
    (unsafe { (*pool).addr_start as usize } + start_idx as usize * PAGE_SIZE as usize) as *mut core::ffi::c_void
}

pub fn page_register(vaddr: *mut core::ffi::c_void, paddr: *mut core::ffi::c_void) -> bool {
    if vaddr.is_null()
        || paddr.is_null()
        || (vaddr as usize & (PAGE_SIZE as usize - 1)) != 0
        || (paddr as usize & (PAGE_SIZE as usize - 1)) != 0
    {
        return false;
    }
    let vaddr_val = vaddr as usize as u32;
    let pde_idx = get_pde(vaddr_val);
    let pte_idx = get_pte(vaddr_val);

    let pde_entry = (0xFFFFF000 | (pde_idx << 2)) as *mut u32;
    let pte_table = ((pde_idx << 12) | 0xFFC00000) as *mut u32;

    unsafe {
        if (*pde_entry & PG_P_1) == 0 {
            let pt_phy = phy_allocate(&raw const PHY_KER_POOL);
            if pt_phy.is_null() {
                return false;
            }
            *pde_entry = ((pt_phy as u32) & 0xFFFFF000) | PG_US_U | PG_RW_W | PG_P_1;
            core::ptr::write_bytes(pte_table, 0, PAGE_SIZE as usize);
        }

        *pte_table.add(pte_idx as usize) = ((paddr as u32) & 0xFFFFF000) | PG_US_U | PG_RW_W | PG_P_1;
    }
    true
}

fn phy_release(pool: *const PhyMemoryPool, paddr: *mut core::ffi::c_void) {
    if pool.is_null() || paddr.is_null() {
        return;
    }
    unsafe {
        let addr = paddr as u32;
        let start = (*pool).addr_start;
        if addr < start || addr >= start.saturating_add((*pool).pool_size) {
            return;
        }
        let offset = addr - start;
        if offset % PAGE_SIZE == 0 {
            let _ = (*pool).phy_bitmap.clear_one(offset / PAGE_SIZE);
        }
    }
}

fn rollback_kernel_pages(vaddr: u32, mapped_count: u32, reserved_count: u32) {
    unsafe {
        for page in 0..mapped_count {
            let current = vaddr + page * PAGE_SIZE;
            let pde_idx = get_pde(current);
            let pte_idx = get_pte(current);
            let pte_table = ((pde_idx << 12) | 0xFFC00000) as *mut u32;
            let pte = pte_table.add(pte_idx as usize);
            let paddr = (*pte & 0xFFFFF000) as *mut core::ffi::c_void;
            *pte = 0;
            core::arch::asm!("invlpg [{0}]", in(reg) current, options(nostack, preserves_flags));
            phy_release(&raw const PHY_KER_POOL, paddr);
        }

        let pool = &*core::ptr::addr_of!(VIR_KER_POOL);
        let first_bit = (vaddr - pool.addr_start) / PAGE_SIZE;
        for bit in first_bit..first_bit + reserved_count {
            let _ = pool.vir_bitmap.clear_one(bit);
        }
    }
}

pub fn page_allocate(cnt: u32, pool_flag: u32) -> *mut core::ffi::c_void {
    if cnt == 0 || pool_flag != PF_KERNEL {
        return core::ptr::null_mut();
    }
    let vaddr = vir_allocate(cnt, pool_flag);
    if vaddr.is_null() {
        return core::ptr::null_mut();
    }
    let mut var_vaddr = vaddr as u32;
    for mapped_count in 0..cnt {
        let paddr = phy_allocate(&raw const PHY_KER_POOL);
        if paddr.is_null() {
            rollback_kernel_pages(vaddr as u32, mapped_count, cnt);
            return core::ptr::null_mut();
        }
        if !page_register(var_vaddr as *mut core::ffi::c_void, paddr) {
            phy_release(&raw const PHY_KER_POOL, paddr);
            rollback_kernel_pages(vaddr as u32, mapped_count, cnt);
            return core::ptr::null_mut();
        }
        var_vaddr += PAGE_SIZE;
    }
    vaddr
}

#[unsafe(no_mangle)]
pub extern "C" fn mem_init_c() { mem_init(); }
#[unsafe(no_mangle)]
pub extern "C" fn mem_pool_init_c(total_mem: u32) { mem_pool_init(total_mem); }
#[unsafe(no_mangle)]
pub extern "C" fn vir_allocate_c(cnt: u32, pool_flag: u32) -> *mut core::ffi::c_void {
    vir_allocate(cnt, pool_flag)
}
#[unsafe(no_mangle)]
pub extern "C" fn phy_allocate_c(pool: *mut PhyMemoryPool) -> *mut core::ffi::c_void {
    phy_allocate(pool)
}
#[unsafe(no_mangle)]
pub extern "C" fn page_register_c(vaddr: *mut core::ffi::c_void, paddr: *mut core::ffi::c_void) {
    let _ = page_register(vaddr, paddr);
}
#[unsafe(no_mangle)]
pub extern "C" fn page_allocate_c(cnt: u32, pool_flag: u32) -> *mut core::ffi::c_void {
    page_allocate(cnt, pool_flag)
}
