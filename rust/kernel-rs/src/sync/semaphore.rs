use crate::ds::list::ListHead;
use crate::task::thread::{task_struct, TaskStruct, TASK_BLOCKED};

#[repr(C)]
pub struct Semaphore {
    pub value: u8,
    pub waiter_head: ListHead,
}

#[repr(C)]
pub struct Lock {
    pub holder: *mut task_struct,
    pub semaphore: Semaphore,
    pub lock_rpt_nr: u32,
}

pub fn sema_init(sema: *mut Semaphore, value: u8) {
    unsafe {
        (*sema).value = value;
        crate::ds::list::list_init(&raw mut (*sema).waiter_head);
    }
}

pub fn lock_init(plock: *mut Lock) {
    unsafe {
        (*plock).holder = core::ptr::null_mut();
        (*plock).lock_rpt_nr = 0;
        sema_init(&raw mut (*plock).semaphore, 1);
    }
}

pub fn sema_wait(sema: *mut Semaphore) {
    unsafe {
        use crate::interrupt::isr::set_intr_status;
        use crate::task::thread::{running_thread, thread_block};
        use crate::ds::list::{list_find_c, list_pushback_c};
        let old_status = crate::interrupt::isr::intr_disable_c();
        let cur = running_thread();

        while (*sema).value == 0 {
            assert!(list_find_c(
                &raw mut (*sema).waiter_head,
                &raw mut (*cur).general_tag,
            ) == 0);
            list_pushback_c(&raw mut (*cur).general_tag, &raw mut (*sema).waiter_head);
            thread_block(TASK_BLOCKED);
        }

        (*sema).value -= 1;
        assert!((*sema).value == 0);
        set_intr_status(old_status);
    }
}

pub fn sema_post(sema: *mut Semaphore) {
    unsafe {
        use crate::ds::list::{list_is_empty_c, list_pop_c};
        use crate::task::thread::thread_unblock;
        use crate::interrupt::isr::{intr_disable_c, set_intr_status};

        let old_status = intr_disable_c();
        assert!((*sema).value == 0);
        if list_is_empty_c(&raw mut (*sema).waiter_head) == 0 {
            let thread_blocked = list_pop_c(&raw mut (*sema).waiter_head);
            let thread = thread_blocked
                .cast::<u8>()
                .sub(core::mem::offset_of!(TaskStruct, general_tag))
                .cast::<TaskStruct>();
            thread_unblock(thread);
        }
        (*sema).value += 1;
        assert!((*sema).value == 1);
        set_intr_status(old_status);
    }
}

pub fn lock_acquire(plock: *mut Lock) {
    unsafe {
        use crate::task::thread::running_thread;

        if (*plock).holder != running_thread() {
            sema_wait(&raw mut (*plock).semaphore);
            (*plock).holder = running_thread();
            assert!((*plock).lock_rpt_nr == 0);
            (*plock).lock_rpt_nr = 1;
        } else {
            (*plock).lock_rpt_nr += 1;
        }
    }
}

pub fn lock_release(plock: *mut Lock) {
    unsafe {
        use crate::task::thread::running_thread;

        assert!((*plock).holder == running_thread());
        if (*plock).lock_rpt_nr > 1 {
            (*plock).lock_rpt_nr -= 1;
            return;
        }
        assert!((*plock).lock_rpt_nr == 1);
        (*plock).holder = core::ptr::null_mut();
        (*plock).lock_rpt_nr = 0;
        sema_post(&raw mut (*plock).semaphore);
    }
}
