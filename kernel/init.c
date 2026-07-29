#include "init.h"
#include "rust.h"

void init_all(){
    clear();
    if (rust_kernel_probe() != RUST_PROBE_MAGIC) {
        put_str("Rust kernel probe failed\n");
        for (;;) {
            intr_disable();
        }
    }
    put_str("RUST_OK\n");
    idt_init();
    pic_init();
    timer_init();
    mem_init();
    for(int i = 0; i < IDT_DESC_CNT; i++) {idt_register(i, 0x06, intr_entry_table[i]);}
    init_idt_table();
    register_intr_handler(0x20, clock_interrupt); 
    init_list();
    init_main_thread();
    console_init();  
    init_keyboard();
}
