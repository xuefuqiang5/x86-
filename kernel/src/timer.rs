use crate::arch::x86::port;

const IRQ0_FREQUENCY: u32 = 100;
const INPUT_FREQUENCY: u32 = 1193180;
const COUNTER0_VALUE: u16 = (INPUT_FREQUENCY / IRQ0_FREQUENCY) as u16;
const CONTRER0_PORT: u16 = 0x40;
const COUNTER0_NO: u8 = 0;
const COUNTER_MODE: u8 = 2;
const READ_WRITE_LATCH: u8 = 3;
const PIT_CONTROL_PORT: u16 = 0x43;

unsafe fn port_out8(p: u16, v: u8) {
    unsafe {
        port::Port::<u8>::new(p).write(v);
    }
}

pub fn frequency_set(
    counter_port: u16,
    counter_no: u8,
    rwl: u8,
    counter_mode: u8,
    counter_value: u16,
) {
    unsafe {
        port_out8(
            PIT_CONTROL_PORT,
            counter_no << 6 | rwl << 4 | counter_mode << 1,
        );
        port_out8(counter_port, counter_value as u8);
        port_out8(counter_port, (counter_value >> 8) as u8);
    }
}

pub fn timer_init() {
    use crate::vga::put_str;
    put_str(b"timer_init start\n\0".as_ptr());
    frequency_set(
        CONTRER0_PORT,
        COUNTER0_NO,
        READ_WRITE_LATCH,
        COUNTER_MODE,
        COUNTER0_VALUE,
    );
    put_str(b"timer_init done\n\0".as_ptr());
}

#[unsafe(no_mangle)]
pub extern "C" fn timer_init_c() {
    timer_init();
}
