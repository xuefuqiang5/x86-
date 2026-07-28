#include "ioqueue.h"
void ioq_init(struct ioqueue *i){
    memset(i->buf, 0, BUF_SIZE);
    i->consumer = NULL;
    i->producer = NULL;
    lock_init(&i->mutex);
    i->head = 0; 
    i->tail = 0; 
}
static int32_t next_pos(int32_t pos){
    return (pos + 1) % BUF_SIZE;
}
bool ioq_is_empty(struct ioqueue *i){
    assert(!is_enable_interrupts());
    return i->head == i->tail;
}
bool ioq_is_full(struct ioqueue *i){
    assert(!is_enable_interrupts());
    return next_pos(i->head) == i->tail;
}
static void ioq_wait(struct task_struct **waiter){
    assert(waiter != NULL && *waiter == NULL);
    *waiter = running_thread();
    thread_block(TASK_BLOCKED);
}
static void ioq_wakeup(struct task_struct **waiter){
    assert(*waiter != NULL);
    thread_unblock(*waiter);
    *waiter = NULL;
}
char ioq_getchar(struct ioqueue *i){
    assert(!is_enable_interrupts());
    while(ioq_is_empty(i)){
        lock_acquire(&i->mutex);
        ioq_wait(&i->consumer);
        lock_release(&i->mutex);
    }
    char c = i->buf[i->tail];
    i->tail = next_pos(i->tail);
    if(i->producer != NULL) ioq_wakeup(&i->producer);
    return c;
}
void ioq_putchar(char c, struct ioqueue *i){
    assert(!is_enable_interrupts());
    while(ioq_is_full(i)){
        lock_acquire(&i->mutex);
        ioq_wait(&i->producer);
        lock_release(&i->mutex);
    }
    i->buf[i->head] = c;
    i->head = next_pos(i->head);
    if(i->consumer != NULL) ioq_wakeup(&i->consumer);
}
