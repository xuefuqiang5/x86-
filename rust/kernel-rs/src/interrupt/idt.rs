pub const IDT_INTGATE: u8 = 0x6;
pub const IDT_TRAPGATE: u8 = 0x7;
pub const IDT_DESC_CNT: u32 = 0x30;

const IDT_PRESENT: u8 = 0x80;
const IDT_GATE32: u8 = 0x8;
const IDT_SIZE: usize = 256;
const CODE_SELECTOR: u16 = 0x08;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct GateDesc {
    baselo: u16,
    selector: u16,
    reserved: u8,
    flags: u8,
    basehi: u16,
}

#[repr(C, packed)]
struct Idtr {
    limit: u16,
    base: *mut GateDesc,
}

static mut IDT: [GateDesc; IDT_SIZE] = [GateDesc {
    baselo: 0,
    selector: 0,
    reserved: 0,
    flags: 0,
    basehi: 0,
}; IDT_SIZE];

unsafe extern "C" {
    fn lidt(addr: *const Idtr);
}

pub fn idt_register(vecnum: u8, gatetype: u8, base: extern "C" fn()) {
    unsafe {
        let idt_ptr = core::ptr::addr_of_mut!(IDT);
        let desc = &mut (*idt_ptr)[vecnum as usize];
        let addr = base as usize as u32;
        desc.selector = CODE_SELECTOR;
        desc.baselo = (addr & 0xffff) as u16;
        desc.basehi = (addr >> 16) as u16;
        desc.reserved = 0;
        desc.flags = gatetype | IDT_PRESENT | IDT_GATE32;
    }
}

pub fn idt_unregister(vecnum: u8) {
    unsafe {
        (*core::ptr::addr_of_mut!(IDT))[vecnum as usize].flags = 0;
    }
}

pub fn change_func_addr(vecnum: u32, base: extern "C" fn()) {
    unsafe {
        let idt_ptr = core::ptr::addr_of_mut!(IDT);
        let addr = base as usize as u32;
        (*idt_ptr)[vecnum as usize].baselo = (addr & 0xffff) as u16;
        (*idt_ptr)[vecnum as usize].basehi = (addr >> 16) as u16;
    }
}

pub fn idt_init() {
    unsafe {
        for i in 0..IDT_SIZE {
            idt_unregister(i as u8);
        }
        let idtr = Idtr {
            limit: (IDT_SIZE * core::mem::size_of::<GateDesc>() - 1) as u16,
            base: core::ptr::addr_of_mut!(IDT) as *mut GateDesc,
        };
        lidt(&raw const idtr);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn idt_register_c(vecnum: u8, gatetype: u8, base: extern "C" fn()) {
    idt_register(vecnum, gatetype, base);
}
#[unsafe(no_mangle)]
pub extern "C" fn idt_unregister_c(vecnum: u8) { idt_unregister(vecnum); }
#[unsafe(no_mangle)]
pub extern "C" fn idt_init_c() { idt_init(); }
#[unsafe(no_mangle)]
pub extern "C" fn change_func_addr_c(vecnum: u32, base: extern "C" fn()) {
    change_func_addr(vecnum, base);
}
