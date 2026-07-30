use crate::sync::semaphore::Lock;

static mut CONSOLE_LOCK: Lock = Lock {
    holder: core::ptr::null_mut(),
    semaphore: crate::sync::semaphore::Semaphore {
        value: 0,
        waiter_head: crate::ds::list::ListHead {
            next: core::ptr::null_mut(),
            prev: core::ptr::null_mut(),
        },
    },
    lock_rpt_nr: 0,
};

pub fn console_init() {
    use crate::sync::semaphore::lock_init;
    lock_init(&raw mut CONSOLE_LOCK);
}

pub fn console_acquire() {
    crate::sync::semaphore::lock_acquire(&raw mut CONSOLE_LOCK);
}

pub fn console_release() {
    crate::sync::semaphore::lock_release(&raw mut CONSOLE_LOCK);
}

pub fn console_put_str(s: *const u8) {
    console_acquire();
    crate::vga::put_str(s);
    console_release();
}

pub fn console_put_char(c: u8) {
    console_acquire();
    crate::vga::put_char(c);
    console_release();
}

pub fn console_put_int(num: u32) {
    console_acquire();
    crate::vga::put_int_hex(num);
    console_release();
}

#[unsafe(no_mangle)]
pub extern "C" fn console_init_c() {
    console_init();
}
#[unsafe(no_mangle)]
pub extern "C" fn console_acquire_c() {
    console_acquire();
}
#[unsafe(no_mangle)]
pub extern "C" fn console_release_c() {
    console_release();
}
#[unsafe(no_mangle)]
pub extern "C" fn console_put_str_c(s: *const u8) {
    console_put_str(s);
}
#[unsafe(no_mangle)]
pub extern "C" fn console_put_char_c(c: u8) {
    console_put_char(c);
}
#[unsafe(no_mangle)]
pub extern "C" fn console_put_int_c(num: u32) {
    console_put_int(num);
}
