#include "rust.h"

void k_thread_a(void *arg);

int main() {
    init_all_c();
    intr_enable_c();
    pic_clearmask_c(0);
    thread_start("print_keybuf", 7, k_thread_a, (void*)"print_keybuf");
    pic_clearmask_c(1);
    while (1) {}
    return 0;
}

void k_thread_a(void *arg __attribute__((unused))) {
    while (1) {
        uint32_t old_status = intr_disable_c();
        if (!ioq_is_empty_c(&keyboard_buf)) {
            char c = ioq_getchar_c(&keyboard_buf);
            console_put_char_c(c);
        }
        set_intr_status(old_status);
    }
}
