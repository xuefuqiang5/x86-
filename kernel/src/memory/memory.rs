use crate::ds::bitmap::Bitmap;
use core::ptr::NonNull;
use core::sync::atomic::AtomicU32;

pub const PAGE_SIZE: u32 = 4096;
pub const PG_P_1: u32 = 1;
pub const PG_P_0: u32 = 0;
pub const PG_RW_R: u32 = 0;
pub const PG_RW_W: u32 = 2;
pub const PG_US_S: u32 = 0;
pub const PG_US_U: u32 = 4;

const RESERVED_PHYSICAL_END: u32 = 0x0020_0000;
const BITMAP_STORAGE_START: u32 = 0xc009_a000;
const KERNEL_DYNAMIC_START: u32 = 0xc010_0000;
const PAGE_ENTRY_ADDRESS_MASK: u32 = 0xffff_f000;
const RECURSIVE_PAGE_TABLE_BASE: u32 = 0xffc0_0000;
const RECURSIVE_PAGE_DIRECTORY_BASE: u32 = 0xffff_f000;
pub const PAGE_DIRECTORY_ENTRY_COUNT: usize = 1024;
pub const KERNEL_PDE_START: usize = 768;
pub const RECURSIVE_PDE_INDEX: usize = 1023;

/// Physical base address of the original kernel page directory.
///
/// A value of zero means that `mem_init` has not saved the active kernel CR3
/// value yet.
static KERNEL_PAGE_DIRECTORY_PADDR: AtomicU32 = AtomicU32::new(0);

/// A 32-bit x86 page directory.
///
/// A page directory contains 1024 32-bit entries and must occupy one
/// page-aligned 4-KiB physical frame.
#[repr(C, align(4096))]
pub struct PageDirectory {
    entries: [u32; PAGE_DIRECTORY_ENTRY_COUNT],
}

const _: () = assert!(core::mem::size_of::<PageDirectory>() == PAGE_SIZE as usize);

/// Errors that can occur while creating a user page directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageDirectoryCreateError {
    /// A kernel page could not be allocated for the new page directory.
    KernelPageAllocationFailed,
}

fn get_pde(vaddr: u32) -> u32 {
    (vaddr >> 22) & 0x3ff
}

fn get_pte(vaddr: u32) -> u32 {
    (vaddr >> 12) & 0x3ff
}

/// Reads the physical base address of the currently active page directory.
#[inline]
pub fn read_cr3() -> u32 {
    let cr3: u32;

    unsafe {
        core::arch::asm!(
            "mov {}, cr3",
            out(reg) cr3,
            options(nomem, nostack, preserves_flags),
        );
    }

    cr3
}

/// Loads a physical page-directory address into CR3.
///
/// Loading CR3 activates a different virtual address space and flushes all
/// non-global TLB entries.
///
/// # Safety
///
/// `page_directory_paddr` must be 4-KiB aligned and must point to a valid
/// 32-bit x86 page directory. The directory must map the kernel code, data,
/// stack, and paging structures required immediately after the switch.
#[inline]
unsafe fn write_cr3(page_directory_paddr: u32) {
    unsafe {
        core::arch::asm!(
            "mov cr3, {0:e}",
            in(reg) page_directory_paddr,
            options(nostack, preserves_flags),
        );
    }
}

/// Activates a user page directory or restores the kernel page directory.
///
/// `Some(page_directory)` selects the supplied user address space. `None`
/// selects the original kernel address space saved during memory
/// initialization.
///
/// This function only changes the active page directory. It does not create
/// mappings, attach the directory to a task, or update the TSS kernel stack.
pub fn activate_page_directory(page_directory: Option<NonNull<PageDirectory>>) {
    let page_directory_paddr = match page_directory {
        Some(page_directory) => {
            active_virtual_to_physical(page_directory.as_ptr() as usize as u32)
                .expect("the page directory must be mapped")
                & PAGE_ENTRY_ADDRESS_MASK
        }
        None => {
            let kernel_page_directory =
                KERNEL_PAGE_DIRECTORY_PADDR.load(core::sync::atomic::Ordering::Relaxed);
            assert_ne!(
                kernel_page_directory, 0,
                "the kernel page directory is not initialized"
            );

            kernel_page_directory
        }
    };
    unsafe {
        write_cr3(page_directory_paddr);
    }
}

/// Translates a virtual address through the currently active page tables.
///
/// The returned physical address includes the original offset within the page.
/// Returns `None` if either the PDE or PTE is not present.
///
/// This function only reads the current paging structures. It never creates,
/// removes, or modifies a mapping.
fn active_virtual_to_physical(vaddr: u32) -> Option<u32> {
    let pde_index = get_pde(vaddr);
    let pte_index = get_pte(vaddr);

    unsafe {
        let page_directory = RECURSIVE_PAGE_DIRECTORY_BASE as *const u32;
        let pde_ptr = page_directory.add(pde_index as usize);
        let pde = core::ptr::read_volatile(pde_ptr);
        if (pde & PG_P_1) == 0 {
            return None;
        }

        let page_table = (RECURSIVE_PAGE_TABLE_BASE + pde_index * PAGE_SIZE) as *const u32;
        let pte_ptr = page_table.add(pte_index as usize);
        let pte = core::ptr::read_volatile(pte_ptr);

        if (pte & PG_P_1) == 0 {
            return None;
        }
        let physical_page = pte & PAGE_ENTRY_ADDRESS_MASK;
        let page_offset = vaddr & (PAGE_SIZE - 1);

        Some(physical_page | page_offset)
    }
}

/// Allocates and initializes a page directory for a new user address space.
///
/// The user portion of the directory is left empty. Kernel PDEs are copied
/// from the currently active page directory so kernel code and data remain
/// accessible while the new directory is active.
///
/// The final PDE is rebuilt as a supervisor-only recursive mapping that points
/// to the physical frame containing the new page directory.
///
/// This function only constructs the page directory. It does not load CR3,
/// create user mappings, initialize a user virtual-address pool, or attach the
/// directory to a task.
///
/// # Returns
///
/// Returns a kernel virtual pointer to the new page directory.
///
/// # Errors
///
/// Returns `PageDirectoryCreateError::KernelPageAllocationFailed` when a page
/// cannot be allocated for the directory.
///
/// # Invariants
///
/// - Entries `[0, 768)` are not present.
/// - Entries `[768, 1023)` share the current kernel mappings.
/// - Entry `1023` recursively maps the new page directory.
/// - All kernel and recursive mappings remain supervisor-only.
/// - A failed operation must not leave partially allocated resources behind.
pub fn create_user_page_directory() -> Result<NonNull<PageDirectory>, PageDirectoryCreateError> {
    let raw_page = page_allocate(1, PF_KERNEL);
    let user_page_directory = NonNull::new(raw_page.cast::<PageDirectory>())
        .ok_or(PageDirectoryCreateError::KernelPageAllocationFailed)?;

    let page_directory_vaddr = user_page_directory.as_ptr() as usize as u32;
    let page_directory_paddr = active_virtual_to_physical(page_directory_vaddr)
        .expect("a page returned by page_allocate must be mapped");

    unsafe {
        core::ptr::write_bytes(
            user_page_directory.cast::<u8>().as_ptr(),
            0,
            PAGE_SIZE as usize,
        );

        let current_directory = &*(RECURSIVE_PAGE_DIRECTORY_BASE as *const PageDirectory);
        let new_directory = &mut *user_page_directory.as_ptr();

        new_directory.entries[KERNEL_PDE_START..RECURSIVE_PDE_INDEX]
            .copy_from_slice(&current_directory.entries[KERNEL_PDE_START..RECURSIVE_PDE_INDEX]);

        new_directory.entries[RECURSIVE_PDE_INDEX] =
            (page_directory_paddr & PAGE_ENTRY_ADDRESS_MASK) | PAGE_KERNEL_RW;
    }

    Ok(user_page_directory)
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

pub const PAGE_KERNEL_RW: u32 = PG_US_S | PG_RW_W | PG_P_1;
pub const PAGE_USER_RW: u32 = PG_US_U | PG_RW_W | PG_P_1;

struct MemoryLayout {
    kernel_physical_start: u32,
    kernel_physical_size: u32,
    user_physical_start: u32,
    user_physical_size: u32,
    kernel_physical_bitmap: u32,
    user_physical_bitmap: u32,
    kernel_virtual_bitmap: u32,
    kernel_bitmap_bytes: u32,
    user_bitmap_bytes: u32,
}

impl MemoryLayout {
    fn from_total_memory(total_mem: u32) -> Self {
        assert!(total_mem > RESERVED_PHYSICAL_END);

        let free_mem = total_mem - RESERVED_PHYSICAL_END;
        let kernel_physical_size = free_mem / 2;
        let user_physical_size = free_mem - kernel_physical_size;
        let kernel_bitmap_bytes = kernel_physical_size / PAGE_SIZE / 8;
        let user_bitmap_bytes = user_physical_size / PAGE_SIZE / 8;

        let kernel_physical_bitmap = BITMAP_STORAGE_START;
        let user_physical_bitmap = kernel_physical_bitmap + kernel_bitmap_bytes;
        let kernel_virtual_bitmap = user_physical_bitmap + user_bitmap_bytes;
        let bitmap_storage_end = kernel_virtual_bitmap + kernel_bitmap_bytes;

        // The first MiB of the high-half mapping contains the kernel image,
        // boot-time metadata, bitmaps, and the main thread stack. Dynamic
        // mappings must start after that complete reserved region.
        assert!(bitmap_storage_end <= KERNEL_DYNAMIC_START);

        Self {
            kernel_physical_start: RESERVED_PHYSICAL_END,
            kernel_physical_size,
            user_physical_start: RESERVED_PHYSICAL_END + kernel_physical_size,
            user_physical_size,
            kernel_physical_bitmap,
            user_physical_bitmap,
            kernel_virtual_bitmap,
            kernel_bitmap_bytes,
            user_bitmap_bytes,
        }
    }
}

static mut TOTAL_MEM: u32 = 0;
static mut VIR_KER_POOL: VirMemoryPool = VirMemoryPool {
    addr_start: 0,
    vir_bitmap: Bitmap {
        byte_size: 0,
        bits: core::ptr::null_mut(),
    },
};
static mut VIR_USR_POOL: VirMemoryPool = VirMemoryPool {
    addr_start: 0,
    vir_bitmap: Bitmap {
        byte_size: 0,
        bits: core::ptr::null_mut(),
    },
};
static mut PHY_KER_POOL: PhyMemoryPool = PhyMemoryPool {
    addr_start: 0,
    phy_bitmap: Bitmap {
        byte_size: 0,
        bits: core::ptr::null_mut(),
    },
    pool_size: 0,
};
static mut PHY_USR_POOL: PhyMemoryPool = PhyMemoryPool {
    addr_start: 0,
    phy_bitmap: Bitmap {
        byte_size: 0,
        bits: core::ptr::null_mut(),
    },
    pool_size: 0,
};

pub fn mem_pool_init(total_mem: u32) {
    let layout = MemoryLayout::from_total_memory(total_mem);

    unsafe {
        PHY_KER_POOL.addr_start = layout.kernel_physical_start;
        PHY_KER_POOL.pool_size = layout.kernel_physical_size;
        PHY_KER_POOL.phy_bitmap.bits = layout.kernel_physical_bitmap as *mut u8;
        PHY_KER_POOL.phy_bitmap.byte_size = layout.kernel_bitmap_bytes;
        crate::ds::bitmap::init_bitmap(core::ptr::addr_of!(PHY_KER_POOL.phy_bitmap));

        PHY_USR_POOL.addr_start = layout.user_physical_start;
        PHY_USR_POOL.pool_size = layout.user_physical_size;
        PHY_USR_POOL.phy_bitmap.bits = layout.user_physical_bitmap as *mut u8;
        PHY_USR_POOL.phy_bitmap.byte_size = layout.user_bitmap_bytes;
        crate::ds::bitmap::init_bitmap(core::ptr::addr_of!(PHY_USR_POOL.phy_bitmap));

        VIR_KER_POOL.addr_start = KERNEL_DYNAMIC_START;
        VIR_KER_POOL.vir_bitmap.byte_size = layout.kernel_bitmap_bytes;
        VIR_KER_POOL.vir_bitmap.bits = layout.kernel_virtual_bitmap as *mut u8;
        crate::ds::bitmap::init_bitmap(core::ptr::addr_of!(VIR_KER_POOL.vir_bitmap));

        // User virtual memory is not implemented yet. Keep this pool empty
        // instead of aliasing and clearing the kernel virtual bitmap.
        VIR_USR_POOL.addr_start = 0;
        VIR_USR_POOL.vir_bitmap.byte_size = 0;
        VIR_USR_POOL.vir_bitmap.bits = core::ptr::null_mut();
    }
}

pub fn mem_init() {
    let kernel_page_directory_paddr = read_cr3() & PAGE_ENTRY_ADDRESS_MASK;
    assert_ne!(
        kernel_page_directory_paddr, 0,
        "the kernel page directory address is invalid"
    );
    KERNEL_PAGE_DIRECTORY_PADDR.store(
        kernel_page_directory_paddr,
        core::sync::atomic::Ordering::Relaxed,
    );

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
            let start_idx = (*core::ptr::addr_of!(VIR_KER_POOL))
                .vir_bitmap
                .set_bits(cnt);
            if start_idx < 0 {
                return core::ptr::null_mut();
            }
            ((*core::ptr::addr_of!(VIR_KER_POOL)).addr_start as usize
                + start_idx as usize * PAGE_SIZE as usize) as *mut core::ffi::c_void
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
    (unsafe { (*pool).addr_start as usize } + start_idx as usize * PAGE_SIZE as usize)
        as *mut core::ffi::c_void
}

pub fn page_register(
    vaddr: *mut core::ffi::c_void,
    paddr: *mut core::ffi::c_void,
    flags: u32,
) -> bool {
    if vaddr.is_null()
        || paddr.is_null()
        || (vaddr as usize & (PAGE_SIZE as usize - 1)) != 0
        || (paddr as usize & (PAGE_SIZE as usize - 1)) != 0
        || (flags & PG_P_1) == 0
    {
        return false;
    }
    let vaddr_val = vaddr as usize as u32;
    let pde_idx = get_pde(vaddr_val);
    let pte_idx = get_pte(vaddr_val);

    let pde_entry = (RECURSIVE_PAGE_DIRECTORY_BASE | (pde_idx << 2)) as *mut u32;
    let pte_table = (RECURSIVE_PAGE_TABLE_BASE | (pde_idx << 12)) as *mut u32;

    unsafe {
        if (*pde_entry & PG_P_1) == 0 {
            let pt_phy = phy_allocate(&raw const PHY_KER_POOL);
            if pt_phy.is_null() {
                return false;
            }
            *pde_entry = ((pt_phy as u32) & PAGE_ENTRY_ADDRESS_MASK) | flags;
            core::ptr::write_bytes(pte_table, 0, PAGE_SIZE as usize);
        }

        let pte = pte_table.add(pte_idx as usize);
        if (*pte & PG_P_1) != 0 {
            return false;
        }

        *pte = ((paddr as u32) & PAGE_ENTRY_ADDRESS_MASK) | flags;
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
            let pte_table = (RECURSIVE_PAGE_TABLE_BASE | (pde_idx << 12)) as *mut u32;
            let pte = pte_table.add(pte_idx as usize);
            let paddr = (*pte & PAGE_ENTRY_ADDRESS_MASK) as *mut core::ffi::c_void;
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
        if !page_register(var_vaddr as *mut core::ffi::c_void, paddr, PAGE_KERNEL_RW) {
            phy_release(&raw const PHY_KER_POOL, paddr);
            rollback_kernel_pages(vaddr as u32, mapped_count, cnt);
            return core::ptr::null_mut();
        }
        var_vaddr += PAGE_SIZE;
    }
    vaddr
}

#[unsafe(no_mangle)]
pub extern "C" fn mem_init_c() {
    mem_init();
}
#[unsafe(no_mangle)]
pub extern "C" fn mem_pool_init_c(total_mem: u32) {
    mem_pool_init(total_mem);
}
#[unsafe(no_mangle)]
pub extern "C" fn vir_allocate_c(cnt: u32, pool_flag: u32) -> *mut core::ffi::c_void {
    vir_allocate(cnt, pool_flag)
}
#[unsafe(no_mangle)]
pub extern "C" fn phy_allocate_c(pool: *mut PhyMemoryPool) -> *mut core::ffi::c_void {
    phy_allocate(pool)
}
#[unsafe(no_mangle)]
pub extern "C" fn page_register_c(
    vaddr: *mut core::ffi::c_void,
    paddr: *mut core::ffi::c_void,
    flags: u32,
) {
    let _ = page_register(vaddr, paddr, flags);
}
#[unsafe(no_mangle)]
pub extern "C" fn page_allocate_c(cnt: u32, pool_flag: u32) -> *mut core::ffi::c_void {
    page_allocate(cnt, pool_flag)
}
