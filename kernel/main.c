#include "init.h"
void k_thread_a(void *arg);
struct lock mutex;
int main(){
    init_all();
    intr_enable();
    pic_clearmask(0); // open clock intr
    thread_start("print_keybuf", 7, k_thread_a, "print_keybuf");
    pic_clearmask(1);// open keybroad intr 
    while(1){
    }
    return 0; 
}
void k_thread_a(void *arg __attribute__((unused))) {
    while(1) {
        uint32_t old_status = intr_disable();
        if(!ioq_is_empty(&keyboard_buf)){
            char c = ioq_getchar(&keyboard_buf);
            console_put_char(c);
        } 
        set_intr_status(old_status);
    }
}
