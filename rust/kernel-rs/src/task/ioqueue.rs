use crate::sync::semaphore::Lock;
use crate::task::thread::{running_thread, task_struct, thread_block, thread_unblock, TASK_BLOCKED};

pub const BUF_SIZE: usize = 64;

#[repr(C)]
pub struct IoQueue {
    pub producer: *mut task_struct,
    pub consumer: *mut task_struct,
    pub mutex: Lock,
    pub buf: [u8; BUF_SIZE],
    pub head: i32,
    pub tail: i32,
}

unsafe fn memset(ptr: *mut u8, val: u8, size: usize) {
    unsafe { core::ptr::write_bytes(ptr, val, size); }
}

pub fn ioq_init(i: *mut IoQueue) {
    unsafe {
        use crate::sync::semaphore::lock_init;
        memset((*i).buf.as_mut_ptr(), 0, BUF_SIZE);
        (*i).consumer = core::ptr::null_mut();
        (*i).producer = core::ptr::null_mut();
        lock_init(&raw mut (*i).mutex);
        (*i).head = 0;
        (*i).tail = 0;
    }
}

fn next_pos(pos: i32) -> i32 {
    (pos + 1) % BUF_SIZE as i32
}

pub fn ioq_is_empty(i: *const IoQueue) -> bool {
    assert!(!crate::task::thread::is_enable_interrupts());
    unsafe { (*i).head == (*i).tail }
}

pub fn ioq_is_full(i: *const IoQueue) -> bool {
    assert!(!crate::task::thread::is_enable_interrupts());
    unsafe { next_pos((*i).head) == (*i).tail }
}

unsafe fn ioq_wait(waiter: *mut *mut task_struct) {
    unsafe {
        assert!(!waiter.is_null() && (*waiter).is_null());
        *waiter = running_thread();
    }
    thread_block(TASK_BLOCKED);
}

unsafe fn ioq_wakeup(waiter: *mut *mut task_struct) {
    unsafe {
        assert!(!(*waiter).is_null());
        thread_unblock(*waiter);
        *waiter = core::ptr::null_mut();
    }
}

pub fn ioq_getchar(i: *mut IoQueue) -> u8 {
    assert!(!crate::task::thread::is_enable_interrupts());
    unsafe {
        use crate::sync::semaphore::{lock_acquire, lock_release};

        while ioq_is_empty(i) {
            lock_acquire(&raw mut (*i).mutex);
            ioq_wait(&raw mut (*i).consumer);
            lock_release(&raw mut (*i).mutex);
        }
        let c = (*i).buf[(*i).tail as usize];
        (*i).tail = next_pos((*i).tail);
        if !(*i).producer.is_null() {
            ioq_wakeup(&raw mut (*i).producer);
        }
        c
    }
}

pub fn ioq_putchar(c: u8, i: *mut IoQueue) {
    assert!(!crate::task::thread::is_enable_interrupts());
    unsafe {
        use crate::sync::semaphore::{lock_acquire, lock_release};

        while ioq_is_full(i) {
            lock_acquire(&raw mut (*i).mutex);
            ioq_wait(&raw mut (*i).producer);
            lock_release(&raw mut (*i).mutex);
        }
        (*i).buf[(*i).head as usize] = c;
        (*i).head = next_pos((*i).head);
        if !(*i).consumer.is_null() {
            ioq_wakeup(&raw mut (*i).consumer);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ioq_init_c(i: *mut IoQueue) { ioq_init(i); }
#[unsafe(no_mangle)]
pub extern "C" fn ioq_is_empty_c(i: *mut IoQueue) -> i32 { ioq_is_empty(i) as i32 }
#[unsafe(no_mangle)]
pub extern "C" fn ioq_is_full_c(i: *mut IoQueue) -> i32 { ioq_is_full(i) as i32 }
#[unsafe(no_mangle)]
pub extern "C" fn ioq_getchar_c(i: *mut IoQueue) -> u8 { ioq_getchar(i) }
#[unsafe(no_mangle)]
pub extern "C" fn ioq_putchar_c(c: u8, i: *mut IoQueue) { ioq_putchar(c, i) }
