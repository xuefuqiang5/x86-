//! x86 Global Descriptor Table support.
//!
//! The implementation is intentionally left as a skeleton.  The descriptor
//! layout and public interfaces are kept here so the table can be completed
//! incrementally without changing the rest of the kernel.

use core::mem::size_of;

use super::tss;

// Segment selectors. The index is selector >> 3; the low two bits are RPL.
pub const KERNEL_CODE_SELECTOR: u16 = 0x08;
pub const KERNEL_DATA_SELECTOR: u16 = 0x10;
pub const VIDEO_SELECTOR: u16 = 0x18;
pub const USER_CODE_SELECTOR: u16 = 0x20 | 0x3;
pub const USER_DATA_SELECTOR: u16 = 0x28 | 0x3;
pub const TSS_SELECTOR: u16 = 0x30;

const KERNEL_VIRTUAL_BASE: u32 = 0xc000_0000;

// The TSS descriptor occupies two consecutive eight-byte GDT entries.
const GDT_ENTRY_COUNT: usize = 8;

// Descriptor flags in the 32-bit descriptor encoding.
const PRESENT: u32 = 1 << 15;
const DPL_RING3: u32 = 3 << 13;
const CODE: u32 = 1 << 12; // S bit: application/code-data descriptor.
const DATA_RW: u32 = 1 << 9;
// Execute/read code segment: TYPE=0b1010.
const CODE_RX: u32 = (1 << 11) | (1 << 9);
const GRANULARITY_4K: u32 = 1 << 23;
const OPERAND_32BIT: u32 = 1 << 22;

/// 32-bit GDTR operand used by `lgdt`.
#[repr(C, packed)]
struct Gdtr {
    limit: u16,
    base: u32,
}

const _: () = assert!(size_of::<Gdtr>() == 6);

#[inline]
unsafe fn read_gdtr() -> Gdtr {
    let mut gdtr = Gdtr { limit: 0, base: 0 };
    let address = core::ptr::addr_of_mut!(gdtr);
    unsafe {
        core::arch::asm!(
            "sgdt [{}]",
            in(reg) address,
            options(nostack, preserves_flags),
        );
    }
    gdtr
}

#[inline]
unsafe fn load_gdtr(gdtr: *const Gdtr) {
    unsafe {
        core::arch::asm!(
            "lgdt [{}]",
            in(reg) gdtr,
            options(nostack, preserves_flags),
        );
    }
}

#[inline]
unsafe fn load_task_register(selector: u16) {
    unsafe {
        core::arch::asm!(
            "ltr ax",
            in("ax") selector,
            options(nostack, preserves_flags),
        );
    }
}

#[inline]
unsafe fn read_task_register() -> u16 {
    let selector: u16;
    unsafe {
        core::arch::asm!(
            "str ax",
            out("ax") selector,
            options(nostack, preserves_flags),
        );
    }
    selector
}
/// Kernel GDT storage.
///
/// Expected layout:
///
/// ```text
/// 0       null descriptor
/// 1       kernel code
/// 2       kernel data
/// 3       video memory
/// 4       user code
/// 5       user data
/// 6..7    32-bit available TSS descriptor
/// ```
static mut GDT: [u64; GDT_ENTRY_COUNT] = [0; GDT_ENTRY_COUNT];

/// Build a flat 32-bit code-segment descriptor for the requested privilege
/// level. `dpl` must be either zero or `DPL_RING3`.
fn flat_code_descriptor(dpl: u32) -> u64 {
    debug_assert!(dpl == 0 || dpl == DPL_RING3);
    descriptor(
        0,
        0xfffff,
        PRESENT | dpl | CODE | CODE_RX | GRANULARITY_4K | OPERAND_32BIT,
    )
}

/// Build a flat 32-bit read/write data-segment descriptor for the requested
/// privilege level. `dpl` must be either zero or `DPL_RING3`.
fn flat_data_descriptor(dpl: u32) -> u64 {
    debug_assert!(dpl == 0 || dpl == DPL_RING3);
    descriptor(
        0,
        0xfffff,
        PRESENT | dpl | DATA_RW | GRANULARITY_4K | OPERAND_32BIT,
    )
}

/// Encode a segment descriptor from its base, limit, and attribute fields.
///
/// `base` and `limit` are split across the descriptor rather than stored as
/// contiguous fields. The caller is responsible for supplying a limit that
/// matches the selected granularity.
fn descriptor(base: u32, limit: u32, flags: u32) -> u64 {
    let low = (limit & 0xffff) | ((base & 0xffff) << 16);
    let high =
        ((base >> 16) & 0xff) | (flags & 0x00f0ff00) | ((limit >> 16) & 0x0f) | (base & 0xff000000);

    ((high as u64) << 32) | low as u64
}

/// Build a 32-bit available TSS descriptor.
///
/// The descriptor base must be the address of `tss::TSS`, and its limit should
/// normally be `size_of::<tss::TaskStateSegment>() - 1`. A 32-bit TSS consumes
/// two consecutive GDT entries.
fn tss_descriptor(base: u32, limit: u32) -> u64 {
    // TYPE=0b1001: 32-bit available TSS.
    // S=0 identifies this as a system descriptor. DPL remains zero.
    let flags = PRESENT | (0b1001 << 8);
    descriptor(base, limit, flags)
}

/// Initialize and load the kernel GDT, reload segment registers, and load the
/// task register with `TSS_SELECTOR`.
///
/// Required work:
///
/// 1. Populate all entries in `GDT` according to the layout above.
/// 2. Build a GDTR with `limit = size_of::<GDT>() - 1` and the GDT address.
/// 3. Execute `lgdt`.
/// 4. Reload CS using a far control transfer; reload DS, ES, SS, and GS.
/// 5. Execute `ltr` with `TSS_SELECTOR`.
///
/// Segment selectors must remain consistent with the table layout because the
/// IDT and the existing assembly code currently use the fixed selector values.
pub fn gdt_init() {
    // The loader already installed the active GDT. Extend its reserved TSS
    // slot and relocate GDTR to the high-half alias before user page
    // directories remove the loader's low identity mapping.
    let loader_gdtr = unsafe { read_gdtr() };
    let loader_gdt_base = loader_gdtr.base;
    let gdt_limit = loader_gdtr.limit;
    let tss_index = (TSS_SELECTOR >> 3) as usize;
    let offset = tss_index * size_of::<u64>();

    assert!(loader_gdt_base != 0);
    assert!(offset + size_of::<u64>() <= gdt_limit as usize + 1);

    let kernel_gdt_base = if loader_gdt_base < KERNEL_VIRTUAL_BASE {
        loader_gdt_base
            .checked_add(KERNEL_VIRTUAL_BASE)
            .expect("the relocated GDT address must fit in 32 bits")
    } else {
        loader_gdt_base
    };

    let tss_base = tss::TSS.as_ptr() as u32;
    let tss_limit = size_of::<tss::TaskStateSegment>() as u32 - 1;
    let descriptor = tss_descriptor(tss_base, tss_limit);
    let kernel_gdtr = Gdtr {
        limit: gdt_limit,
        base: kernel_gdt_base,
    };

    unsafe {
        let table = kernel_gdt_base as *mut u64;
        table.add(tss_index).write_volatile(descriptor);
        load_gdtr(core::ptr::addr_of!(kernel_gdtr));
        load_task_register(TSS_SELECTOR);
    }

    let active_gdtr = unsafe { read_gdtr() };
    let active_gdt_base = active_gdtr.base;
    let task_register = unsafe { read_task_register() };

    assert!(active_gdt_base == kernel_gdt_base);
    assert!(task_register == TSS_SELECTOR);
}
