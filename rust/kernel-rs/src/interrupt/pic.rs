use crate::arch::x86::port;

const MASTER_CMD: u16 = 0x20;
const MASTER_IMR: u16 = 0x21;
const MASTER_DATA: u16 = 0x21;
const SLAVE_CMD: u16 = 0xa0;
const SLAVE_IMR: u16 = 0xa1;
const SLAVE_DATA: u16 = 0xa1;
const MASK_IRQ2: u8 = 0x04;
const MASK_ALL: u8 = 0xff;

static mut MASTER_IMR_VAL: u8 = 0;
static mut SLAVE_IMR_VAL: u8 = 0;

unsafe fn port_out8(p: u16, v: u8) {
    unsafe { port::Port::<u8>::new(p).write(v); }
}

pub fn pic_init() {
    unsafe {
        port_out8(MASTER_CMD, 0x11);
        port_out8(SLAVE_CMD, 0x11);
        port_out8(MASTER_DATA, 0x20);
        port_out8(SLAVE_DATA, 0x28);
        port_out8(MASTER_DATA, 0x04);
        port_out8(SLAVE_DATA, 0x02);
        port_out8(MASTER_DATA, 0x01);
        port_out8(SLAVE_DATA, 0x01);
        MASTER_IMR_VAL = !MASK_IRQ2;
        SLAVE_IMR_VAL = MASK_ALL;
        port_out8(MASTER_IMR, MASTER_IMR_VAL);
        port_out8(SLAVE_IMR, SLAVE_IMR_VAL);
    }
}

pub fn pic_clearmask(irq: i32) {
    if !(0..16).contains(&irq) {
        return;
    }
    unsafe {
        if irq < 8 {
            MASTER_IMR_VAL &= !(1u8 << irq);
            port_out8(MASTER_IMR, MASTER_IMR_VAL);
        } else {
            SLAVE_IMR_VAL &= !(1u8 << (irq - 8));
            port_out8(SLAVE_IMR, SLAVE_IMR_VAL);
        }
    }
}

pub fn pic_setmask(irq: i32) {
    if !(0..16).contains(&irq) {
        return;
    }
    unsafe {
        if irq < 8 {
            MASTER_IMR_VAL |= 1u8 << irq;
            port_out8(MASTER_IMR, MASTER_IMR_VAL);
        } else {
            SLAVE_IMR_VAL |= 1u8 << (irq - 8);
            port_out8(SLAVE_IMR, SLAVE_IMR_VAL);
        }
    }
}

pub fn pic_sendeoi(irq: i32) {
    if !(0..16).contains(&irq) {
        return;
    }
    unsafe {
        if irq >= 8 {
            port_out8(SLAVE_CMD, 0x20);
        }
        port_out8(MASTER_CMD, 0x20);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pic_init_c() { pic_init(); }
#[unsafe(no_mangle)]
pub extern "C" fn pic_clearmask_c(irq: i32) { pic_clearmask(irq); }
#[unsafe(no_mangle)]
pub extern "C" fn pic_setmask_c(irq: i32) { pic_setmask(irq); }
#[unsafe(no_mangle)]
pub extern "C" fn pic_sendeoi_c(irq: i32) { pic_sendeoi(irq); }
