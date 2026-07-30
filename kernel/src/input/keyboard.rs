use crate::task::ioqueue::IoQueue;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Scanf2ascii {
    pub make_code: u8,
    pub break_code: u8,
    pub ascii: u8,
}

#[repr(C)]
pub struct ModifierFlags {
    pub caps_lock: u8,
    pub num_lock: u8,
    pub left_shift: u8,
    pub right_shift: u8,
    pub left_ctrl: u8,
    pub right_ctrl: u8,
    pub left_alt: u8,
    pub right_alt: u8,
}

#[unsafe(no_mangle)]
pub static mut keyboard_buf: IoQueue = IoQueue {
    producer: core::ptr::null_mut(),
    consumer: core::ptr::null_mut(),
    mutex: unsafe { core::mem::zeroed() },
    buf: [0; 64],
    head: 0,
    tail: 0,
};

pub static mut MODIFY_KEY_STATUS: ModifierFlags = ModifierFlags {
    caps_lock: 0,
    num_lock: 0,
    left_shift: 0,
    right_shift: 0,
    left_ctrl: 0,
    right_ctrl: 0,
    left_alt: 0,
    right_alt: 0,
};

pub static mut KEY_MAPPING_TABLE: [Scanf2ascii; 256] = {
    let mut table = [Scanf2ascii {
        make_code: 0,
        break_code: 0,
        ascii: 0,
    }; 256];

    // Letters
    table[0x1E] = Scanf2ascii {
        make_code: 0x1E,
        break_code: 0x9E,
        ascii: b'a',
    };
    table[0x30] = Scanf2ascii {
        make_code: 0x30,
        break_code: 0xB0,
        ascii: b'b',
    };
    table[0x2E] = Scanf2ascii {
        make_code: 0x2E,
        break_code: 0xAE,
        ascii: b'c',
    };
    table[0x20] = Scanf2ascii {
        make_code: 0x20,
        break_code: 0xA0,
        ascii: b'd',
    };
    table[0x12] = Scanf2ascii {
        make_code: 0x12,
        break_code: 0x92,
        ascii: b'e',
    };
    table[0x21] = Scanf2ascii {
        make_code: 0x21,
        break_code: 0xA1,
        ascii: b'f',
    };
    table[0x22] = Scanf2ascii {
        make_code: 0x22,
        break_code: 0xA2,
        ascii: b'g',
    };
    table[0x23] = Scanf2ascii {
        make_code: 0x23,
        break_code: 0xA3,
        ascii: b'h',
    };
    table[0x17] = Scanf2ascii {
        make_code: 0x17,
        break_code: 0x97,
        ascii: b'i',
    };
    table[0x24] = Scanf2ascii {
        make_code: 0x24,
        break_code: 0xA4,
        ascii: b'j',
    };
    table[0x25] = Scanf2ascii {
        make_code: 0x25,
        break_code: 0xA5,
        ascii: b'k',
    };
    table[0x26] = Scanf2ascii {
        make_code: 0x26,
        break_code: 0xA6,
        ascii: b'l',
    };
    table[0x32] = Scanf2ascii {
        make_code: 0x32,
        break_code: 0xB2,
        ascii: b'm',
    };
    table[0x31] = Scanf2ascii {
        make_code: 0x31,
        break_code: 0xB1,
        ascii: b'n',
    };
    table[0x18] = Scanf2ascii {
        make_code: 0x18,
        break_code: 0x98,
        ascii: b'o',
    };
    table[0x19] = Scanf2ascii {
        make_code: 0x19,
        break_code: 0x99,
        ascii: b'p',
    };
    table[0x10] = Scanf2ascii {
        make_code: 0x10,
        break_code: 0x90,
        ascii: b'q',
    };
    table[0x13] = Scanf2ascii {
        make_code: 0x13,
        break_code: 0x93,
        ascii: b'r',
    };
    table[0x1F] = Scanf2ascii {
        make_code: 0x1F,
        break_code: 0x9F,
        ascii: b's',
    };
    table[0x14] = Scanf2ascii {
        make_code: 0x14,
        break_code: 0x94,
        ascii: b't',
    };
    table[0x16] = Scanf2ascii {
        make_code: 0x16,
        break_code: 0x96,
        ascii: b'u',
    };
    table[0x2F] = Scanf2ascii {
        make_code: 0x2F,
        break_code: 0xAF,
        ascii: b'v',
    };
    table[0x11] = Scanf2ascii {
        make_code: 0x11,
        break_code: 0x91,
        ascii: b'w',
    };
    table[0x2D] = Scanf2ascii {
        make_code: 0x2D,
        break_code: 0xAD,
        ascii: b'x',
    };
    table[0x15] = Scanf2ascii {
        make_code: 0x15,
        break_code: 0x95,
        ascii: b'y',
    };
    table[0x2C] = Scanf2ascii {
        make_code: 0x2C,
        break_code: 0xAC,
        ascii: b'z',
    };

    // Numbers
    table[0x02] = Scanf2ascii {
        make_code: 0x02,
        break_code: 0x82,
        ascii: b'1',
    };
    table[0x03] = Scanf2ascii {
        make_code: 0x03,
        break_code: 0x83,
        ascii: b'2',
    };
    table[0x04] = Scanf2ascii {
        make_code: 0x04,
        break_code: 0x84,
        ascii: b'3',
    };
    table[0x05] = Scanf2ascii {
        make_code: 0x05,
        break_code: 0x85,
        ascii: b'4',
    };
    table[0x06] = Scanf2ascii {
        make_code: 0x06,
        break_code: 0x86,
        ascii: b'5',
    };
    table[0x07] = Scanf2ascii {
        make_code: 0x07,
        break_code: 0x87,
        ascii: b'6',
    };
    table[0x08] = Scanf2ascii {
        make_code: 0x08,
        break_code: 0x88,
        ascii: b'7',
    };
    table[0x09] = Scanf2ascii {
        make_code: 0x09,
        break_code: 0x89,
        ascii: b'8',
    };
    table[0x0A] = Scanf2ascii {
        make_code: 0x0A,
        break_code: 0x8A,
        ascii: b'9',
    };
    table[0x0B] = Scanf2ascii {
        make_code: 0x0B,
        break_code: 0x8B,
        ascii: b'0',
    };

    // Symbols
    table[0x29] = Scanf2ascii {
        make_code: 0x29,
        break_code: 0xA9,
        ascii: b'`',
    };
    table[0x0C] = Scanf2ascii {
        make_code: 0x0C,
        break_code: 0x8C,
        ascii: b'-',
    };
    table[0x0D] = Scanf2ascii {
        make_code: 0x0D,
        break_code: 0x8D,
        ascii: b'=',
    };
    table[0x1A] = Scanf2ascii {
        make_code: 0x1A,
        break_code: 0x9A,
        ascii: b'[',
    };
    table[0x1B] = Scanf2ascii {
        make_code: 0x1B,
        break_code: 0x9B,
        ascii: b']',
    };
    table[0x2B] = Scanf2ascii {
        make_code: 0x2B,
        break_code: 0xAB,
        ascii: b'\\',
    };
    table[0x27] = Scanf2ascii {
        make_code: 0x27,
        break_code: 0xA7,
        ascii: b';',
    };
    table[0x28] = Scanf2ascii {
        make_code: 0x28,
        break_code: 0xA8,
        ascii: b'\'',
    };
    table[0x33] = Scanf2ascii {
        make_code: 0x33,
        break_code: 0xB3,
        ascii: b',',
    };
    table[0x34] = Scanf2ascii {
        make_code: 0x34,
        break_code: 0xB4,
        ascii: b'.',
    };
    table[0x35] = Scanf2ascii {
        make_code: 0x35,
        break_code: 0xB5,
        ascii: b'/',
    };

    // Control keys
    table[0x1C] = Scanf2ascii {
        make_code: 0x1C,
        break_code: 0x9C,
        ascii: b'\n',
    };
    table[0x0E] = Scanf2ascii {
        make_code: 0x0E,
        break_code: 0x8E,
        ascii: 0x08,
    };
    table[0x0F] = Scanf2ascii {
        make_code: 0x0F,
        break_code: 0x8F,
        ascii: b'\t',
    };
    table[0x39] = Scanf2ascii {
        make_code: 0x39,
        break_code: 0xB9,
        ascii: b' ',
    };
    table[0x3A] = Scanf2ascii {
        make_code: 0x3A,
        break_code: 0xBA,
        ascii: 0,
    };
    table[0x2A] = Scanf2ascii {
        make_code: 0x2A,
        break_code: 0xAA,
        ascii: 0,
    };
    table[0x36] = Scanf2ascii {
        make_code: 0x36,
        break_code: 0xB6,
        ascii: 0,
    };
    table[0x1D] = Scanf2ascii {
        make_code: 0x1D,
        break_code: 0x9D,
        ascii: 0,
    };
    table[0x38] = Scanf2ascii {
        make_code: 0x38,
        break_code: 0xB8,
        ascii: 0,
    };
    table[0x45] = Scanf2ascii {
        make_code: 0x45,
        break_code: 0xC5,
        ascii: 0,
    };

    // Function keys
    table[0x01] = Scanf2ascii {
        make_code: 0x01,
        break_code: 0x81,
        ascii: 0x1B,
    };
    table[0x3B] = Scanf2ascii {
        make_code: 0x3B,
        break_code: 0xBB,
        ascii: 0,
    };
    table[0x3C] = Scanf2ascii {
        make_code: 0x3C,
        break_code: 0xBC,
        ascii: 0,
    };
    table[0x3D] = Scanf2ascii {
        make_code: 0x3D,
        break_code: 0xBD,
        ascii: 0,
    };
    table[0x3E] = Scanf2ascii {
        make_code: 0x3E,
        break_code: 0xBE,
        ascii: 0,
    };
    table[0x3F] = Scanf2ascii {
        make_code: 0x3F,
        break_code: 0xBF,
        ascii: 0,
    };
    table[0x40] = Scanf2ascii {
        make_code: 0x40,
        break_code: 0xC0,
        ascii: 0,
    };
    table[0x41] = Scanf2ascii {
        make_code: 0x41,
        break_code: 0xC1,
        ascii: 0,
    };
    table[0x42] = Scanf2ascii {
        make_code: 0x42,
        break_code: 0xC2,
        ascii: 0,
    };
    table[0x43] = Scanf2ascii {
        make_code: 0x43,
        break_code: 0xC3,
        ascii: 0,
    };
    table[0x44] = Scanf2ascii {
        make_code: 0x44,
        break_code: 0xC4,
        ascii: 0,
    };
    table[0x57] = Scanf2ascii {
        make_code: 0x57,
        break_code: 0xD7,
        ascii: 0,
    };
    table[0x58] = Scanf2ascii {
        make_code: 0x58,
        break_code: 0xD8,
        ascii: 0,
    };

    // Numpad
    table[0x52] = Scanf2ascii {
        make_code: 0x52,
        break_code: 0xD2,
        ascii: b'0',
    };
    table[0x4F] = Scanf2ascii {
        make_code: 0x4F,
        break_code: 0xCF,
        ascii: b'1',
    };
    table[0x50] = Scanf2ascii {
        make_code: 0x50,
        break_code: 0xD0,
        ascii: b'2',
    };
    table[0x51] = Scanf2ascii {
        make_code: 0x51,
        break_code: 0xD1,
        ascii: b'3',
    };
    table[0x4B] = Scanf2ascii {
        make_code: 0x4B,
        break_code: 0xCB,
        ascii: b'4',
    };
    table[0x4C] = Scanf2ascii {
        make_code: 0x4C,
        break_code: 0xCC,
        ascii: b'5',
    };
    table[0x4D] = Scanf2ascii {
        make_code: 0x4D,
        break_code: 0xCD,
        ascii: b'6',
    };
    table[0x47] = Scanf2ascii {
        make_code: 0x47,
        break_code: 0xC7,
        ascii: b'7',
    };
    table[0x48] = Scanf2ascii {
        make_code: 0x48,
        break_code: 0xC8,
        ascii: b'8',
    };
    table[0x49] = Scanf2ascii {
        make_code: 0x49,
        break_code: 0xC9,
        ascii: b'9',
    };
    table[0x37] = Scanf2ascii {
        make_code: 0x37,
        break_code: 0xB7,
        ascii: b'*',
    };
    table[0x4E] = Scanf2ascii {
        make_code: 0x4E,
        break_code: 0xCE,
        ascii: b'+',
    };
    table[0x4A] = Scanf2ascii {
        make_code: 0x4A,
        break_code: 0xCA,
        ascii: b'-',
    };
    table[0x53] = Scanf2ascii {
        make_code: 0x53,
        break_code: 0xD3,
        ascii: b'.',
    };

    table
};

pub fn shift_char(c: u8) -> u8 {
    match c {
        b'1' => b'!',
        b'2' => b'@',
        b'3' => b'#',
        b'4' => b'$',
        b'5' => b'%',
        b'6' => b'^',
        b'7' => b'&',
        b'8' => b'*',
        b'9' => b'(',
        b'0' => b')',
        b'-' => b'_',
        b'=' => b'+',
        b'`' => b'~',
        b',' => b'<',
        b'.' => b'>',
        b'/' => b'?',
        b'[' => b'{',
        b']' => b'}',
        b'\\' => b'|',
        b';' => b':',
        b'\'' => b'"',
        _ => 0,
    }
}

const CAPS_LOCK_BREAK: u8 = 0x3A | 0x80;
const NUM_LOCK_BREAK: u8 = 0x45 | 0x80;
const LEFT_SHIFT_MAKE: u8 = 0x2A;
const LEFT_SHIFT_BREAK: u8 = 0x2A | 0x80;
const RIGHT_SHIFT_MAKE: u8 = 0x36;
const RIGHT_SHIFT_BREAK: u8 = 0x36 | 0x80;
const LEFT_CTRL_MAKE: u8 = 0x1D;
const LEFT_CTRL_BREAK: u8 = 0x1D | 0x80;
const LEFT_ALT_MAKE: u8 = 0x38;
const LEFT_ALT_BREAK: u8 = 0x38 | 0x80;

pub fn change_key_status(code: u8) {
    unsafe {
        match code {
            CAPS_LOCK_BREAK => MODIFY_KEY_STATUS.caps_lock ^= 1,
            NUM_LOCK_BREAK => MODIFY_KEY_STATUS.num_lock ^= 1,
            LEFT_SHIFT_MAKE => MODIFY_KEY_STATUS.left_shift = 1,
            LEFT_SHIFT_BREAK => MODIFY_KEY_STATUS.left_shift = 0,
            LEFT_ALT_MAKE => MODIFY_KEY_STATUS.left_alt = 1,
            LEFT_ALT_BREAK => MODIFY_KEY_STATUS.left_alt = 0,
            LEFT_CTRL_MAKE => MODIFY_KEY_STATUS.left_ctrl = 1,
            LEFT_CTRL_BREAK => MODIFY_KEY_STATUS.left_ctrl = 0,
            RIGHT_SHIFT_MAKE => MODIFY_KEY_STATUS.right_shift = 1,
            RIGHT_SHIFT_BREAK => MODIFY_KEY_STATUS.right_shift = 0,
            _ => {}
        }
    }
}

const KEY_PORT: u16 = 0x60;

fn in8(port: u16) -> u8 {
    let val: u8;
    unsafe {
        core::arch::asm!("in al, dx", in("dx") port, out("al") val, options(nomem, nostack, preserves_flags));
    }
    val
}

#[unsafe(no_mangle)]
pub extern "C" fn keyboard_intr_handler() {
    unsafe {
        use crate::task::ioqueue::{ioq_is_full, ioq_putchar};
        use crate::vga::to_upper;

        let s = in8(KEY_PORT) as usize;
        if KEY_MAPPING_TABLE[s].ascii == 0 {
            change_key_status(s as u8);
            return;
        }
        let mut c = KEY_MAPPING_TABLE[s].ascii;
        if c.is_ascii_lowercase() {
            let shift = MODIFY_KEY_STATUS.left_shift == 1 || MODIFY_KEY_STATUS.right_shift == 1;
            if shift ^ (MODIFY_KEY_STATUS.caps_lock == 1) {
                c = to_upper(c);
            }
        } else if MODIFY_KEY_STATUS.left_shift == 1 || MODIFY_KEY_STATUS.right_shift == 1 {
            c = shift_char(c);
        }
        if !ioq_is_full(&raw mut keyboard_buf) {
            ioq_putchar(c, &raw mut keyboard_buf);
        }
    }
}

pub fn init_keyboard() {
    use crate::interrupt::isr::register_intr_handler;
    use crate::task::ioqueue::ioq_init;
    use crate::vga::put_str;

    register_intr_handler(0x21, keyboard_intr_handler);
    ioq_init(&raw mut keyboard_buf);
    put_str(b"keyboard has done\n\0".as_ptr());
}

#[unsafe(no_mangle)]
pub extern "C" fn shift_char_c(c: u8) -> u8 {
    shift_char(c)
}
#[unsafe(no_mangle)]
pub extern "C" fn change_key_status_c(code: u8) {
    change_key_status(code);
}
#[unsafe(no_mangle)]
pub extern "C" fn init_keyboard_c() {
    init_keyboard();
}
