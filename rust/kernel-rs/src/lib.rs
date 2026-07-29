#![no_std]

mod arch;
mod panic;

/// Value returned across the C ABI to prove that the Rust object code is
/// present in the final kernel image and can be called safely from C.
pub const RUST_PROBE_MAGIC: u32 = 0x5255_5354;

/// Temporary C-to-Rust boundary used while the kernel is migrated module by
/// module.
///
/// Only fixed-width values and raw pointers may cross this boundary.
#[unsafe(no_mangle)]
pub extern "C" fn rust_kernel_probe() -> u32 {
    RUST_PROBE_MAGIC
}
