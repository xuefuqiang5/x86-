#pragma once
#include <stdint.h>

#define RUST_PROBE_MAGIC 0x52555354u

uint32_t rust_kernel_probe(void);

struct list_head {
    struct list_head *next;
    struct list_head *prev;
};

struct semaphore {
    uint8_t value;
    struct list_head waiter_head;
};

struct task_struct;

struct lock {
    struct task_struct *holder;
    struct semaphore semaphore;
    uint32_t lock_rpt_nr;
};

enum task_status {
    TASK_RUNNING,
    TASK_READY,
    TASK_BLOCKED,
    TASK_WAITING,
    TASK_HANGING,
    TASK_DIED
};

typedef void thread_func(void*);

struct task_struct {
    uint32_t *self_kstack;
    enum task_status status;
    uint8_t priority;
    char name[16];
    uint8_t ticks;
    uint32_t elapsed_ticks;
    struct list_head general_tag;
    struct list_head all_list_tag;
    void *pgdir;
    uint32_t stack_magic;
};

struct ioqueue {
    struct task_struct *producer;
    struct task_struct *consumer;
    struct lock mutex;
    char buf[64];
    int32_t head;
    int32_t tail;
};

extern struct ioqueue keyboard_buf;

void put_char(char c);
void put_str(char *s);
void put_int_hex(uint32_t num);
void clear(void);
char to_upper(char c);

int list_is_empty_c(struct list_head *list);
void list_pushback_c(struct list_head *item, struct list_head *list);
void list_pushfront_c(struct list_head *item, struct list_head *list);
void list_remove_c(struct list_head *item);
struct list_head *list_pop_c(struct list_head *list);
int list_find_c(struct list_head *list, struct list_head *item);

void pic_clearmask_c(int irq);

void idt_register_c(uint8_t vecnum, uint8_t gatetype, void (*base)(void));
void init_idt_table_c(void);
void register_intr_handler_c(uint32_t vecnum, void (*func)(void));
void clock_interrupt_c(void);

int intr_enable_c(void);
int intr_disable_c(void);
void set_intr_status(uint32_t status);

void init_thread(struct task_struct *pthread, const char *name, int prio);
struct task_struct *thread_start(const char *name, int prio, thread_func function, void *func_arg);
struct task_struct *running_thread(void);
void init_list(void);
void init_main_thread(void);
void schedule(void);
void thread_block(enum task_status stat);
void thread_unblock(struct task_struct *pthread);

void ioq_init_c(struct ioqueue *i);
int ioq_is_empty_c(struct ioqueue *i);
int ioq_is_full_c(struct ioqueue *i);
char ioq_getchar_c(struct ioqueue *i);
void ioq_putchar_c(char c, struct ioqueue *i);

void lock_init(struct lock *l);
void lock_acquire(struct lock *l);
void lock_release(struct lock *l);

void mem_init_c(void);
void *page_allocate_c(uint32_t cnt, uint32_t pool_flag);

void timer_init_c(void);
void console_init_c(void);
void console_put_char_c(char c);
void init_keyboard_c(void);
void init_all_c(void);

int is_enable_interrupts_c(void);
