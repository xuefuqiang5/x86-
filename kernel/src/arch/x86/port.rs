use core::arch::asm;
use core::marker::PhantomData;

/// Typed x86 I/O port.
///
/// Constructing a port is safe because it does not access hardware. Reads and
/// writes are unsafe: the caller must ensure that the port exists, the access
/// width is correct, and the operation is valid for the current device state.
pub struct Port<T> {
    number: u16,
    value_type: PhantomData<T>,
}

impl<T> Port<T> {
    pub const fn new(number: u16) -> Self {
        Self {
            number,
            value_type: PhantomData,
        }
    }
}

impl Port<u8> {
    pub unsafe fn read(&self) -> u8 {
        let value: u8;
        // SAFETY: The caller upholds the I/O-port contract documented above.
        unsafe {
            asm!(
                "in al, dx",
                in("dx") self.number,
                out("al") value,
                options(nomem, nostack, preserves_flags)
            );
        }
        value
    }

    pub unsafe fn write(&self, value: u8) {
        // SAFETY: The caller upholds the I/O-port contract documented above.
        unsafe {
            asm!(
                "out dx, al",
                in("dx") self.number,
                in("al") value,
                options(nomem, nostack, preserves_flags)
            );
        }
    }
}

impl Port<u16> {
    pub unsafe fn read(&self) -> u16 {
        let value: u16;
        // SAFETY: The caller upholds the I/O-port contract documented above.
        unsafe {
            asm!(
                "in ax, dx",
                in("dx") self.number,
                out("ax") value,
                options(nomem, nostack, preserves_flags)
            );
        }
        value
    }

    pub unsafe fn write(&self, value: u16) {
        // SAFETY: The caller upholds the I/O-port contract documented above.
        unsafe {
            asm!(
                "out dx, ax",
                in("dx") self.number,
                in("ax") value,
                options(nomem, nostack, preserves_flags)
            );
        }
    }
}

impl Port<u32> {
    pub unsafe fn read(&self) -> u32 {
        let value: u32;
        // SAFETY: The caller upholds the I/O-port contract documented above.
        unsafe {
            asm!(
                "in eax, dx",
                in("dx") self.number,
                out("eax") value,
                options(nomem, nostack, preserves_flags)
            );
        }
        value
    }

    pub unsafe fn write(&self, value: u32) {
        // SAFETY: The caller upholds the I/O-port contract documented above.
        unsafe {
            asm!(
                "out dx, eax",
                in("dx") self.number,
                in("eax") value,
                options(nomem, nostack, preserves_flags)
            );
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn in8(port: u16) -> u8 {
    // SAFETY: This compatibility ABI preserves the contract of the original
    // assembly routine. New Rust drivers should expose device-specific APIs.
    unsafe { Port::<u8>::new(port).read() }
}

#[unsafe(no_mangle)]
pub extern "C" fn in16(port: u16) -> u16 {
    // SAFETY: See in8.
    unsafe { Port::<u16>::new(port).read() }
}

#[unsafe(no_mangle)]
pub extern "C" fn in32(port: u16) -> u32 {
    // SAFETY: See in8.
    unsafe { Port::<u32>::new(port).read() }
}

#[unsafe(no_mangle)]
pub extern "C" fn out8(port: u16, value: u8) {
    // SAFETY: See in8.
    unsafe { Port::<u8>::new(port).write(value) }
}

#[unsafe(no_mangle)]
pub extern "C" fn out16(port: u16, value: u16) {
    // SAFETY: See in8.
    unsafe { Port::<u16>::new(port).write(value) }
}

#[unsafe(no_mangle)]
pub extern "C" fn out32(port: u16, value: u32) {
    // SAFETY: See in8.
    unsafe { Port::<u32>::new(port).write(value) }
}
