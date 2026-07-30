use core::arch::asm;

unsafe extern "C" {
    fn ex_write(old_pos: u16, new_pos: u16, background: u16);
    fn write_one_char(pos: u16, c: u8);
}

const VIDEO_SELECTOR: u16 = 0x18;
const VIDEO_COLS: u16 = 80;
const VIDEO_ROWS: u16 = 25;

fn out8(port: u16, val: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
    }
}

fn in8(port: u16) -> u8 {
    let val: u8;
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") val, options(nomem, nostack, preserves_flags));
    }
    val
}

#[unsafe(no_mangle)]
pub extern "C" fn set_cursor_pos_c(cursor: u16) { set_cursor_pos(cursor); }

fn set_cursor_pos(cursor: u16) {
    let cursor_low = (cursor & 0xff) as u8;
    let cursor_high = ((cursor >> 8) & 0xff) as u8;
    out8(0x3d4, 0x0e);
    out8(0x3d5, cursor_high);
    out8(0x3d4, 0x0f);
    out8(0x3d5, cursor_low);
}

fn setup_gs() {
    unsafe {
        asm!("mov gs, ax", in("ax") VIDEO_SELECTOR, options(nomem, nostack, preserves_flags));
    }
}

fn write_into(cursor: u16, c: u8) {
    let pos = cursor * 2;
    setup_gs();
    unsafe { write_one_char(pos, c) };
}

fn roll_screen() {
    setup_gs();
    for i in 1..VIDEO_ROWS {
        for j in 0..VIDEO_COLS {
            let old_pos = (i * VIDEO_COLS + j) * 2;
            let new_pos = ((i - 1) * VIDEO_COLS + j) * 2;
            let background = new_pos + 1;
            unsafe { ex_write(old_pos, new_pos, background) };
        }
    }
    let mut cursor = 24 * VIDEO_COLS;
    for _ in 0..VIDEO_COLS {
        write_into(cursor, b' ');
        cursor += 1;
    }
}

fn read_cursor() -> u16 {
    out8(0x3d4, 0x0e);
    let high = in8(0x3d5) as u16;
    out8(0x3d4, 0x0f);
    let low = in8(0x3d5) as u16;
    (high << 8) | low
}

#[unsafe(no_mangle)]
pub extern "C" fn put_char(c: u8) {
    setup_gs();

    let mut cursor = read_cursor();

    match c {
        b'\n' => {
            if cursor / VIDEO_COLS == 24 {
                roll_screen();
                cursor = 24 * VIDEO_COLS;
            } else {
                cursor = (cursor / VIDEO_COLS + 1) * VIDEO_COLS;
            }
        }
        b'\r' => {
            cursor = cursor / VIDEO_COLS * VIDEO_COLS;
        }
        b'\x08' => {
            if cursor > 0 {
                cursor -= 1;
                write_into(cursor, b' ');
            }
        }
        b'\t' => {
            let new_col = cursor % VIDEO_COLS + 8 - (cursor % VIDEO_COLS % 8);
            if new_col >= VIDEO_COLS {
                let new_row = cursor / VIDEO_COLS + 1;
                if new_row >= VIDEO_ROWS {
                    roll_screen();
                    cursor = 24 * VIDEO_COLS;
                } else {
                    cursor = new_row * VIDEO_COLS;
                }
            } else {
                cursor = cursor / VIDEO_COLS * VIDEO_COLS + new_col;
            }
        }
        _ => {
            write_into(cursor, c);
            cursor += 1;
            if cursor >= VIDEO_COLS * VIDEO_ROWS {
                roll_screen();
                cursor = 24 * VIDEO_COLS;
            }
        }
    }

    set_cursor_pos(cursor);
}

#[unsafe(no_mangle)]
pub extern "C" fn put_str(s: *const u8) {
    let mut p = s;
    loop {
        let ch = unsafe { *p };
        if ch == 0 {
            break;
        }
        put_char(ch);
        p = unsafe { p.add(1) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn put_int_hex(num: u32) {
    if num == 0 {
        put_char(b'0');
        return;
    }
    let mut buf = [0u8; 32];
    let mut n = num;
    let mut i = 0usize;
    loop {
        if n == 0 {
            break;
        }
        let t = (n % 16) as u8;
        buf[i] = if t < 10 { b'0' + t } else { b'A' + t - 10 };
        n /= 16;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        put_char(buf[i]);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn put_int_dec(num: u32) {
    if num == 0 {
        put_char(b'0');
        return;
    }
    let mut buf = [0u8; 32];
    let mut n = num;
    let mut i = 0usize;
    loop {
        if n == 0 {
            break;
        }
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        put_char(buf[i]);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn clear() {
    setup_gs();
    for cursor in 0..(VIDEO_COLS * VIDEO_ROWS) {
        write_into(cursor, b' ');
    }
    set_cursor_pos(0);
}

#[unsafe(no_mangle)]
pub extern "C" fn to_upper(c: u8) -> u8 {
    if c.is_ascii_lowercase() {
        c - 32
    } else {
        c
    }
}
