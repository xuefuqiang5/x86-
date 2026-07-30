use crate::ds::list::ListHead;

pub const MAGIC_NUM: u32 = 0x19940625;
pub const PAGE_SIZE: u32 = 4096;

#[repr(C)]
pub struct TaskStruct {
    pub self_kstack: *mut u32,
    pub status: u32,
    pub priority: u8,
    pub name: [u8; 16],
    pub ticks: u8,
    pub elapsed_ticks: u32,
    pub general_tag: ListHead,
    pub all_list_tag: ListHead,
    pub pgdir: *mut core::ffi::c_void,
    pub stack_magic: u32,
}

#[repr(C)]
pub struct IntrStack {
    pub vec_no: u32,
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub esp_dummy: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
    pub gs: u32,
    pub fs: u32,
    pub es: u32,
    pub ds: u32,
    pub err_code: u32,
    pub eip: extern "C" fn(),
    pub cs: u32,
    pub eflags: u32,
    pub esp: *mut core::ffi::c_void,
    pub ss: u32,
}

#[repr(C)]
pub struct ThreadStack {
    pub ebp: u32,
    pub ebx: u32,
    pub edi: u32,
    pub esi: u32,
    pub eip: extern "C" fn(thread_func: extern "C" fn(*mut core::ffi::c_void), func_arg: *mut core::ffi::c_void),
    pub unused_retaddr: extern "C" fn(),
    pub function: extern "C" fn(*mut core::ffi::c_void),
    pub func_arg: *mut core::ffi::c_void,
}

#[allow(non_camel_case_types)]
pub type thread_func = extern "C" fn(*mut core::ffi::c_void);
#[allow(non_camel_case_types)]
pub type task_struct = TaskStruct;

pub const TASK_RUNNING: u32 = 0;
pub const TASK_READY: u32 = 1;
pub const TASK_BLOCKED: u32 = 2;
pub const TASK_WAITING: u32 = 3;
pub const TASK_HANGING: u32 = 4;
pub const TASK_DIED: u32 = 5;

unsafe extern "C" {
    pub fn switch_to(cur: *mut TaskStruct, next: *mut TaskStruct);
    pub fn intr_enable() -> i32;
    pub fn intr_disable() -> i32;
}

fn page_align_down(x: u32) -> u32 {
    x & !(PAGE_SIZE - 1)
}

#[unsafe(no_mangle)]
pub extern "C" fn running_thread() -> *mut TaskStruct {
    let esp: u32;
    unsafe {
        core::arch::asm!("mov {0}, esp", out(reg) esp, options(nomem, nostack, preserves_flags));
    }
    page_align_down(esp) as *mut TaskStruct
}

unsafe fn streq(a: *const u8, b: *const u8) -> bool {
    let mut pa = a;
    let mut pb = b;
    loop {
        let ca = unsafe { *pa };
        if ca == 0 || ca != unsafe { *pb } {
            return ca == unsafe { *pb };
        }
        pa = unsafe { pa.add(1) };
        pb = unsafe { pb.add(1) };
    }
}

unsafe fn copy_thread_name(dst: *mut u8, src: *const u8) {
    let mut len = 0;
    while len < 15 {
        let c = unsafe { *src.add(len) };
        unsafe { *dst.add(len) = c; }
        if c == 0 {
            return;
        }
        len += 1;
    }
    unsafe { *dst.add(len) = 0; }
}

unsafe fn task_from_general_tag(tag: *mut ListHead) -> *mut TaskStruct {
    unsafe {
        tag.cast::<u8>()
            .sub(core::mem::offset_of!(TaskStruct, general_tag))
            .cast::<TaskStruct>()
    }
}

#[unsafe(no_mangle)]
extern "C" fn kernel_thread(function: extern "C" fn(*mut core::ffi::c_void), func_arg: *mut core::ffi::c_void) {
    unsafe { intr_enable(); }
    function(func_arg);
    unsafe {
        intr_disable();
        let cur = running_thread();
        (*cur).status = TASK_DIED;
        schedule();
    }
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)); }
    }
}

pub fn thread_create(
    pthread: *mut TaskStruct,
    function: extern "C" fn(*mut core::ffi::c_void),
    func_arg: *mut core::ffi::c_void,
) {
    unsafe {
        (*pthread).self_kstack = ((*pthread).self_kstack as usize - core::mem::size_of::<IntrStack>()) as *mut u32;
        (*pthread).self_kstack = ((*pthread).self_kstack as usize - core::mem::size_of::<ThreadStack>()) as *mut u32;
        let kthread_stack = &mut *((*pthread).self_kstack as *mut ThreadStack);
        kthread_stack.eip = kernel_thread;
        kthread_stack.function = function;
        kthread_stack.func_arg = func_arg;
        kthread_stack.ebp = 0;
        kthread_stack.ebx = 0;
        kthread_stack.edi = 0;
        kthread_stack.esi = 0;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn init_thread(pthread: *mut TaskStruct, name: *const u8, prio: i32) {
    unsafe {
        use crate::vga::put_str;

        let is_main = streq(name, b"main\0".as_ptr());
        if !is_main {
            core::ptr::write_bytes(pthread as *mut u8, 0, PAGE_SIZE as usize);
        }
        copy_thread_name((*pthread).name.as_mut_ptr(), name);
        if is_main {
            (*pthread).status = TASK_RUNNING;
        } else {
            (*pthread).status = TASK_READY;
        }
        (*pthread).self_kstack = (pthread as usize + PAGE_SIZE as usize) as *mut u32;
        (*pthread).elapsed_ticks = 0;
        let priority = prio.clamp(1, u8::MAX as i32) as u8;
        (*pthread).priority = priority;
        (*pthread).ticks = priority;
        (*pthread).pgdir = core::ptr::null_mut();
        (*pthread).stack_magic = MAGIC_NUM;

        put_str(b"init \0".as_ptr());
        put_str(name);
        put_str(b" done \n\0".as_ptr());
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn thread_start(
    name: *const u8,
    prio: i32,
    function: extern "C" fn(*mut core::ffi::c_void),
    func_arg: *mut core::ffi::c_void,
) -> *mut TaskStruct {
    unsafe {
        use crate::ds::list::{list_find_c, list_pushback_c};
        use crate::memory::memory::page_allocate_c;
        use crate::vga::put_str;

        let thread = page_allocate_c(1, 1) as *mut TaskStruct;
        if thread.is_null() {
            return core::ptr::null_mut();
        }
        init_thread(thread, name, prio);
        thread_create(thread, function, func_arg);
        assert!(list_find_c(&raw mut READY_LIST_HEAD, &raw mut (*thread).general_tag) == 0);
        list_pushback_c(&raw mut (*thread).general_tag, &raw mut READY_LIST_HEAD);
        assert!(list_find_c(&raw mut ALL_LIST_HEAD, &raw mut (*thread).all_list_tag) == 0);
        list_pushback_c(&raw mut (*thread).all_list_tag, &raw mut ALL_LIST_HEAD);
        put_str(b"start done \n\0".as_ptr());
        thread
    }
}

#[unsafe(no_mangle)]
static mut READY_LIST_HEAD: ListHead = ListHead { next: core::ptr::null_mut(), prev: core::ptr::null_mut() };
#[unsafe(no_mangle)]
static mut ALL_LIST_HEAD: ListHead = ListHead { next: core::ptr::null_mut(), prev: core::ptr::null_mut() };

#[unsafe(no_mangle)]
pub extern "C" fn init_list() {
    use crate::ds::list::list_init_c;
    use crate::vga::put_str;

    list_init_c(&raw mut READY_LIST_HEAD);
    list_init_c(&raw mut ALL_LIST_HEAD);
    put_str(b"list done \n\0".as_ptr());
}

#[unsafe(no_mangle)]
pub extern "C" fn init_main_thread() {
    unsafe {
        use crate::ds::list::{list_find_c, list_pushback_c};
        use crate::vga::put_str;

        let m = running_thread();
        init_thread(m, b"main\0".as_ptr(), 31);
        assert!(list_find_c(&raw mut ALL_LIST_HEAD, &raw mut (*m).all_list_tag) == 0);
        list_pushback_c(&raw mut (*m).all_list_tag, &raw mut ALL_LIST_HEAD);
        put_str(b"main done \n\0".as_ptr());
    }
}

pub fn is_enable_interrupts() -> bool {
    let flags: u32;
    unsafe {
        core::arch::asm!(
            "pushfd",
            "pop {0}",
            out(reg) flags,
            options(nomem, preserves_flags)
        );
    }
    (flags & (1 << 9)) != 0
}

#[unsafe(no_mangle)]
pub extern "C" fn is_enable_interrupts_c() -> i32 {
    is_enable_interrupts() as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn schedule() {
    unsafe {
        use crate::ds::list::{list_find_c, list_pushback_c, list_pop_c};

        assert!(!is_enable_interrupts());
        let cur = running_thread();
        if (*cur).status == TASK_RUNNING {
            assert!(list_find_c(&raw mut READY_LIST_HEAD, &raw mut (*cur).general_tag) == 0);
            list_pushback_c(&raw mut (*cur).general_tag, &raw mut READY_LIST_HEAD);
            (*cur).ticks = (*cur).priority;
            (*cur).status = TASK_READY;
        }

        let next_ptr = list_pop_c(&raw mut READY_LIST_HEAD);
        assert!(!next_ptr.is_null());
        let next = &mut *task_from_general_tag(next_ptr);
        next.status = TASK_RUNNING;
        switch_to(cur, next);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn thread_block(stat: u32) {
    unsafe {
        use crate::interrupt::isr::{intr_disable_c, set_intr_status};

        assert!(stat == TASK_BLOCKED || stat == TASK_WAITING || stat == TASK_HANGING);
        let old_status = intr_disable_c();
        let cur = running_thread();
        (*cur).status = stat;
        schedule();
        set_intr_status(old_status);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn thread_unblock(pthread: *mut TaskStruct) {
    unsafe {
        use crate::ds::list::{list_find_c, list_pushfront_c};
        use crate::interrupt::isr::{intr_disable_c, set_intr_status};

        let old_status = intr_disable_c();
        assert!(
            (*pthread).status == TASK_BLOCKED
                || (*pthread).status == TASK_WAITING
                || (*pthread).status == TASK_HANGING
        );
        assert!(list_find_c(&raw mut READY_LIST_HEAD, &raw mut (*pthread).general_tag) == 0);
        list_pushfront_c(&raw mut (*pthread).general_tag, &raw mut READY_LIST_HEAD);
        (*pthread).status = TASK_READY;
        set_intr_status(old_status);
    }
}
