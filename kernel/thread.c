#include "thread.h"
#include "memory.h"
struct task_struct *main_thread;
struct list_head ready_list_head;
struct list_head all_list_head;
struct list_head block_list_head;
static void kernel_thread(thread_func* function, void* func_arg){
    intr_enable();
    function(func_arg);
}
struct task_struct* running_thread(){
    uint32_t esp;
    asm volatile("mov %%esp, %0" : "=g"(esp));
    return (struct task_struct*)PAGE_ALIGN_DOWN(esp);
}
void thread_create(struct task_struct* pthread, thread_func function, void* func_arg){
    pthread->self_kstack -= sizeof(struct intr_stack);
    pthread->self_kstack -= sizeof(struct thread_stack);
    struct thread_stack* kthread_stack = (struct thread_stack*)pthread->self_kstack;
    kthread_stack->eip = kernel_thread;
    kthread_stack->function = function;
    kthread_stack->func_arg = func_arg;
    kthread_stack->ebp = kthread_stack->ebx = kthread_stack->edi = kthread_stack->esi = 0;
}

static int streq(const char *a, const char *b) {
    while (*a && *a == *b) {
        a++;
        b++;
    }
    return *a == *b;
}

void init_thread(struct task_struct* pthread, char* name, int prio){
    if(!streq(name, "main")) memset(pthread, 0, PAGE_SIZE);
    strcpy(pthread->name, name);
    if(pthread == main_thread){
        pthread->status = TASK_RUNNING;
    }else{pthread->status = TASK_READY;}
    pthread->self_kstack = (uint32_t*)((uint32_t)pthread + PAGE_SIZE);
    pthread->elapsed_ticks = 0;
    pthread->priority = prio;
    pthread->ticks = prio;
    pthread->pgdir = NULL;
    pthread->stack_magic = MAGIC_NUM;
    put_str("init ");
    put_str(name);
    put_str(" done \n");
}
struct task_struct* thread_start(char* name, int prio, thread_func function, void* func_arg){
    struct task_struct* thread = page_allocate(1, PF_KERNEL);
    init_thread(thread, name, prio);
    thread_create(thread, function, func_arg);
    assert(!list_find(&ready_list_head, &thread->general_tag));
    list_pushback(&thread->general_tag, &ready_list_head);
    assert(!list_find(&all_list_head, &thread->all_list_tag));
    list_pushback(&thread->all_list_tag, &all_list_head);
    put_str("start done \n");
    return thread;
}
void init_list(){
    list_init(&ready_list_head);
    list_init(&all_list_head);
    put_str("list done \n");
}

void init_main_thread(){
    main_thread = running_thread();
    init_thread(main_thread, "main", 31);
    assert(!(list_find(&all_list_head, &main_thread->all_list_tag)));
    list_pushback(&main_thread->all_list_tag, &all_list_head);
    put_str("main done \n");
}

void schedule(){
    assert(!(is_enable_interrupts()));
    struct task_struct* cur = running_thread();
    if(cur->status == TASK_RUNNING){
        assert(!list_find(&ready_list_head, &cur->general_tag));
        list_pushback(&cur->general_tag, &ready_list_head);
        cur->ticks = cur->priority;
        cur->status = TASK_READY;
    }
    else {}
    struct task_struct* next = to_entry(list_pop(&ready_list_head), struct task_struct, general_tag); 
    next->status = TASK_RUNNING;
    switch_to(cur, next);
}

void print_task_struct(struct task_struct *task) {
    put_str("Task Information:\n");

    // 打印 self_kstack
    put_str("  self_kstack: ");
    if (task->self_kstack == NULL) {
        put_str("NULL\n");
    } else {
        put_int_hex((uint32_t)task->self_kstack);
        put_char('\n');
    }

    // 打印 status
    put_str("  status: ");
    switch (task->status) {
        case TASK_RUNNING:
            put_str("TASK_RUNNING\n");
            break;
        case TASK_READY:
            put_str("TASK_READY\n");
            break;
        case TASK_BLOCKED:
            put_str("TASK_BLOCKED\n");
            break;
        case TASK_WAITING:
            put_str("TASK_WAITING\n");
            break;
        default:
            put_str("UNKNOWN\n");
            break;
    }

    // 打印 priority
    put_str("  priority: ");
    put_int_hex(task->priority);
    put_char('\n');

    // 打印 name
    put_str("  name: ");
    put_str(task->name);
    put_char('\n');

    // 打印 ticks
    put_str("  ticks: ");
    put_int_hex(task->ticks);
    put_char('\n');

    // 打印 elapsed_ticks
    put_str("  elapsed_ticks: ");
    put_int_hex(task->elapsed_ticks);
    put_char('\n');

    // 打印 pgdir
    put_str("  pgdir: ");
    if (task->pgdir == NULL) {
        put_str("NULL\n");
    } else {
        put_int_hex((uint32_t)task->pgdir);
        put_char('\n');
    }

    // 打印 stack_magic
    put_str("  stack_magic: ");
    put_int_hex(task->stack_magic);
    put_char('\n');
}
// thread_block(); 由当前线程调用，是的线程不占用处理机
// thread_unblock(); 由锁的持有人调用，用于唤醒被阻塞的线程
/** 
 * thread_block 由当前线程调度，改变thread_status 之后使之不进入就绪队列（也就是线程调度的时候不能选中）
 * 直至满足某种条件的时候（由其他线程唤醒）之后才能恢复执行
 */
void thread_block(enum task_status stat){
    assert((stat == TASK_BLOCKED) 
        ||  (stat == TASK_WAITING)
        ||  (stat == TASK_HANGING)
        ); 
        uint32_t old_status = intr_disable();
        struct task_struct *cur = running_thread(); 
        cur->status = stat;
        schedule();
        set_intr_status(old_status);
} 
 //thread_unblock() 由其他线程调用唤醒指定线程（唤醒条件由其他线程满足并且调用）
void thread_unblock(struct task_struct *pthread){
    uint32_t old_status = intr_disable();
    assert(pthread->status == TASK_BLOCKED
        ||  pthread->status == TASK_WAITING
        || pthread->status == TASK_HANGING
        );
    assert(!list_find(&ready_list_head, &pthread->general_tag));
    list_pushfront(&pthread->general_tag, &ready_list_head);
    pthread->status = TASK_READY;
    set_intr_status(old_status);

    
}
