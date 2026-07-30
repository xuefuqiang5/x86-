#![no_std]

mod arch;
mod panic;
pub mod vga;
pub mod ds;
pub mod sync;
pub mod task;
pub mod interrupt;
pub mod timer;
pub mod input;
pub mod memory;
pub mod console;
pub mod init;

pub const RUST_PROBE_MAGIC: u32 = 0x5255_5354;

#[unsafe(no_mangle)]
pub extern "C" fn rust_kernel_probe() -> u32 {
    RUST_PROBE_MAGIC
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut p = dest;
    let byte = c as u8;
    for _ in 0..n {
        unsafe { *p = byte; }
        p = unsafe { p.add(1) };
    }
    dest
}
