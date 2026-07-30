#![no_std]

use core::ffi::c_void;

mod arch;
pub mod console;
pub mod ds;
pub mod init;
pub mod input;
pub mod interrupt;
pub mod memory;
mod panic;
pub mod sync;
pub mod task;
pub mod timer;
pub mod vga;

extern "C" fn keyboard_consumer(_arg: *mut c_void) {
    loop {
        let old_status = interrupt::isr::intr_disable_c();
        let queue = &raw mut input::keyboard::keyboard_buf;
        if !task::ioqueue::ioq_is_empty(queue) {
            let c = task::ioqueue::ioq_getchar(queue);
            console::console_put_char(c);
        }
        interrupt::isr::set_intr_status(old_status);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    init::init_all();
    interrupt::isr::intr_enable_c();
    interrupt::pic::pic_clearmask(0);

    let thread = task::thread::thread_start(
        b"print_keybuf\0".as_ptr(),
        7,
        keyboard_consumer,
        core::ptr::null_mut(),
    );
    assert!(!thread.is_null());

    interrupt::pic::pic_clearmask(1);
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}

/// # Safety
///
/// `dest` must be valid for writes of `n` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut p = dest;
    let byte = c as u8;
    for _ in 0..n {
        unsafe {
            *p = byte;
        }
        p = unsafe { p.add(1) };
    }
    dest
}
