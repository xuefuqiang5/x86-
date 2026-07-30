use crate::interrupt::idt::IDT_DESC_CNT;

unsafe extern "C" {
    static intr_entry_table: [usize; IDT_DESC_CNT as usize];
}

pub fn init_all() {
    unsafe {
        crate::vga::clear();
        crate::interrupt::idt::idt_init();
        crate::interrupt::pic::pic_init();
        crate::timer::timer_init();
        crate::memory::memory::mem_init();

        for i in 0..(IDT_DESC_CNT as usize) {
            let entry_fn: extern "C" fn() = core::mem::transmute(intr_entry_table[i]);
            crate::interrupt::idt::idt_register_c(
                i as u8,
                crate::interrupt::idt::IDT_INTGATE,
                entry_fn,
            );
        }
        crate::interrupt::isr::init_idt_table_c();
        crate::interrupt::isr::register_intr_handler_c(
            0x20,
            crate::interrupt::isr::clock_interrupt_c,
        );
        crate::task::thread::init_list();
        crate::task::thread::init_main_thread();
        crate::console::console_init_c();
        crate::input::keyboard::init_keyboard_c();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn init_all_c() {
    init_all();
}
