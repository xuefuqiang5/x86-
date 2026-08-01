use core::cell::UnsafeCell;
use core::mem::size_of;

pub const KERNEL_DATA_SELECTOR: u32 = 0x10;

#[repr(C)]
pub struct TaskStateSegment {
    previous_task: u32,
    pub esp0: u32,
    pub ss0: u32,
    esp1: u32,
    ss1: u32,
    esp2: u32,
    ss2: u32,
    cr3: u32,
    eip: u32,
    eflags: u32,
    eax: u32,
    ecx: u32,
    edx: u32,
    ebx: u32,
    esp: u32,
    ebp: u32,
    esi: u32,
    edi: u32,
    es: u32,
    cs: u32,
    ss: u32,
    ds: u32,
    fs: u32,
    gs: u32,
    ldt: u32,
    // Low 16 bits: trap flag; high 16 bits: I/O bitmap offset.
    trap_iomap: u32,
}

const _: () = assert!(size_of::<TaskStateSegment>() == 104);

impl TaskStateSegment {
    const fn new() -> Self {
        Self {
            previous_task: 0,
            esp0: 0,
            ss0: KERNEL_DATA_SELECTOR,
            esp1: 0,
            ss1: 0,
            esp2: 0,
            ss2: 0,
            cr3: 0,
            eip: 0,
            eflags: 0,
            eax: 0,
            ecx: 0,
            edx: 0,
            ebx: 0,
            esp: 0,
            ebp: 0,
            esi: 0,
            edi: 0,
            es: 0,
            cs: 0,
            ss: 0,
            ds: 0,
            fs: 0,
            gs: 0,
            ldt: 0,
            // No I/O bitmap is present; the offset points immediately after the TSS.
            trap_iomap: (size_of::<TaskStateSegment>() as u32) << 16,
        }
    }
}

pub struct GlobalTss {
    inner: UnsafeCell<TaskStateSegment>,
}
impl GlobalTss {
    pub fn as_ptr(&self) -> *mut TaskStateSegment {
        self.inner.get()
    }
}

unsafe impl Sync for GlobalTss {}

#[unsafe(no_mangle)]
pub static TSS: GlobalTss = GlobalTss {
    inner: UnsafeCell::new(TaskStateSegment::new()),
};

pub fn set_esp0(esp0: u32) {
    unsafe {
        (*TSS.inner.get()).esp0 = esp0;
    }
}
