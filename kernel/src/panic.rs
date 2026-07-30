use core::panic::PanicInfo;

const PANIC_MESSAGE: &[u8] = b"RUST PANIC\n\0";

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    crate::interrupt::isr::intr_disable_c();
    crate::vga::put_str(PANIC_MESSAGE.as_ptr());

    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}
