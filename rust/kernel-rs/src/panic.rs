use core::panic::PanicInfo;

unsafe extern "C" {
    fn put_str(message: *const u8);
    fn intr_disable() -> i32;
}

const PANIC_MESSAGE: &[u8] = b"RUST PANIC\n\0";

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    // SAFETY: These functions use the existing kernel C ABI. PANIC_MESSAGE is
    // NUL-terminated and remains valid for the duration of the call.
    unsafe {
        intr_disable();
        put_str(PANIC_MESSAGE.as_ptr());
    }

    loop {
        core::hint::spin_loop();
    }
}
