use crate::task::thread::{MAGIC_NUM, running_thread};

pub const IDT_DESC_CNT: usize = 0x30;

unsafe extern "C" {
    fn intr_enable() -> i32;
    fn intr_disable() -> i32;
}

#[unsafe(no_mangle)]
static mut idt_table: [usize; IDT_DESC_CNT] = [0; IDT_DESC_CNT];

#[unsafe(no_mangle)]
pub extern "C" fn general_program(vecnum: u32) {
    if vecnum == 0x20 {
        return;
    }
    unsafe {
        use crate::vga::{clear, put_char, put_int_hex, put_str};
        clear();
        crate::vga::set_cursor_pos_c(80 * 4);
        put_str(b"the thread name is  \0".as_ptr());
        let cur = running_thread();
        put_str((*cur).name.as_ptr());
        put_char(b'\n');
        put_str(b"the vecnum  = \0".as_ptr());
        put_int_hex(vecnum);
        put_char(b'\n');
        loop {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}

pub fn init_idt_table() {
    let addr = general_program as *const () as usize;
    for i in 0..IDT_DESC_CNT {
        unsafe {
            (*core::ptr::addr_of_mut!(idt_table))[i] = addr;
        }
    }
}

pub fn register_intr_handler(vecnum: u32, func: extern "C" fn()) {
    assert!((vecnum as usize) < IDT_DESC_CNT);
    unsafe {
        (*core::ptr::addr_of_mut!(idt_table))[vecnum as usize] = func as usize;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn clock_interrupt() {
    unsafe {
        let cur = running_thread();
        assert!((*cur).stack_magic == MAGIC_NUM);
        (*cur).elapsed_ticks += 1;
        if (*cur).ticks <= 1 {
            crate::task::thread::schedule();
        } else {
            (*cur).ticks -= 1;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn intr_disable_c() -> u32 {
    let flags: u32;
    unsafe {
        core::arch::asm!(
            "pushfd",
            "cli",
            "pop {0}",
            "and {0}, 0x200",
            out(reg) flags,
            options(nomem)
        );
    }
    flags
}

#[unsafe(no_mangle)]
pub extern "C" fn intr_enable_c() -> u32 {
    let flags: u32;
    unsafe {
        core::arch::asm!(
            "pushfd",
            "sti",
            "pop {0}",
            "and {0}, 0x200",
            out(reg) flags,
            options(nomem)
        );
    }
    flags
}

#[unsafe(no_mangle)]
pub extern "C" fn set_intr_status(status: u32) {
    assert!(status == 0x200 || status == 0x00);
    if status == 0x200 {
        unsafe {
            intr_enable();
        }
    } else {
        unsafe {
            intr_disable();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn init_idt_table_c() {
    init_idt_table();
}
#[unsafe(no_mangle)]
pub extern "C" fn register_intr_handler_c(vecnum: u32, func: extern "C" fn()) {
    register_intr_handler(vecnum, func);
}
#[unsafe(no_mangle)]
pub extern "C" fn clock_interrupt_c() {
    clock_interrupt();
}
