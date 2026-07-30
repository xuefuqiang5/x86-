#[repr(C)]
pub struct Bitmap {
    pub byte_size: u32,
    pub bits: *mut u8,
}

impl Bitmap {
    pub fn test(&self, idx: u32) -> bool {
        if idx >= self.byte_size * 8 {
            return false;
        }
        let byte_idx = (idx / 8) as usize;
        let bit = (idx % 8) as u8;
        unsafe { (*self.bits.add(byte_idx) & (1 << bit)) != 0 }
    }

    pub fn set_one(&self, idx: u32) -> i32 {
        if idx >= self.byte_size * 8 {
            return -1;
        }
        let byte_idx = (idx / 8) as usize;
        let bit = (idx % 8) as u8;
        unsafe { *self.bits.add(byte_idx) |= 1 << bit };
        0
    }

    pub fn clear_one(&self, idx: u32) -> i32 {
        if idx >= self.byte_size * 8 {
            return -1;
        }
        let byte_idx = (idx / 8) as usize;
        let bit = (idx % 8) as u8;
        unsafe { *self.bits.add(byte_idx) &= !(1 << bit) };
        0
    }

    pub fn set_bits(&self, cnt: u32) -> i32 {
        let total = self.byte_size.saturating_mul(8);
        if cnt == 0 || cnt > total {
            return -1;
        }

        let mut run_start = 0;
        let mut run_len = 0;
        for idx in 0..total {
            if self.test(idx) {
                run_len = 0;
            } else {
                if run_len == 0 {
                    run_start = idx;
                }
                run_len += 1;
                if run_len == cnt {
                    for bit in run_start..run_start + cnt {
                        let _ = self.set_one(bit);
                    }
                    return run_start as i32;
                }
            }
        }
        -1
    }
}

pub fn init_bitmap(bmp: *const Bitmap) {
    unsafe {
        core::ptr::write_bytes((*bmp).bits, 0, (*bmp).byte_size as usize);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn set_one_bit(bmp: *mut Bitmap, idx: u32) -> i32 {
    unsafe { (*bmp).set_one(idx) }
}

#[unsafe(no_mangle)]
pub extern "C" fn clear_bitmap_fn(bmp: *mut Bitmap, idx: u32) -> i32 {
    unsafe { (*bmp).clear_one(idx) }
}

#[unsafe(no_mangle)]
pub extern "C" fn init_bitmap_c(bmp: *mut Bitmap) {
    init_bitmap(bmp)
}

#[unsafe(no_mangle)]
pub extern "C" fn bitmap_test(bmp: *mut Bitmap, idx: u32) -> i32 {
    if unsafe { (*bmp).test(idx) } { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn set_bits(bmp: *mut Bitmap, cnt: u32) -> i32 {
    unsafe { (*bmp).set_bits(cnt) }
}
