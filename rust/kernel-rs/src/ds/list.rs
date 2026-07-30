#[repr(C)]
pub struct ListHead {
    pub next: *mut ListHead,
    pub prev: *mut ListHead,
}

pub fn list_init(list: *mut ListHead) {
    unsafe {
        (*list).next = list;
        (*list).prev = list;
    }
}

pub fn list_is_empty(list: *const ListHead) -> bool {
    unsafe { (*list).next as *const ListHead == list }
}

pub fn list_pushfront(item: *mut ListHead, list: *mut ListHead) {
    unsafe {
        (*item).next = (*list).next;
        (*item).prev = list;
        (*(*list).next).prev = item;
        (*list).next = item;
    }
}

pub fn list_pushback(item: *mut ListHead, list: *mut ListHead) {
    unsafe {
        (*item).next = list;
        (*item).prev = (*list).prev;
        (*(*list).prev).next = item;
        (*list).prev = item;
    }
}

pub fn list_append_front(dst: *mut ListHead, src: *mut ListHead) {
    if list_is_empty(src) {
        return;
    }
    unsafe {
        (*(*src).prev).next = (*dst).next;
        (*(*dst).next).prev = (*src).prev;
        (*dst).next = (*src).next;
        (*(*src).next).prev = dst;
    }
    list_init(src);
}

pub fn list_append_back(dst: *mut ListHead, src: *mut ListHead) {
    if list_is_empty(src) {
        return;
    }
    unsafe {
        (*(*src).next).prev = (*dst).prev;
        (*(*dst).prev).next = (*src).next;
        (*dst).prev = (*src).prev;
        (*(*src).prev).next = dst;
    }
    list_init(src);
}

pub fn list_rotate_forward(list: *mut ListHead) {
    if list_is_empty(list) {
        return;
    }
    let first = unsafe { (*list).next };
    list_remove(first);
    list_pushback(first, list);
}

pub fn list_rotate_backward(list: *mut ListHead) {
    if list_is_empty(list) {
        return;
    }
    let last = unsafe { (*list).prev };
    list_remove(last);
    list_pushfront(last, list);
}

pub fn list_remove(item: *mut ListHead) {
    unsafe {
        (*(*item).prev).next = (*item).next;
        (*(*item).next).prev = (*item).prev;
    }
    list_init(item);
}

pub fn list_pop(list: *mut ListHead) -> *mut ListHead {
    if list_is_empty(list) {
        return core::ptr::null_mut();
    }
    let first = unsafe { (*list).next };
    list_remove(first);
    first
}

pub fn list_find(list: *mut ListHead, item: *mut ListHead) -> bool {
    let mut p = unsafe { (*list).next };
    while p != list {
        if p == item {
            return true;
        }
        p = unsafe { (*p).next };
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn list_init_c(list: *mut ListHead) { list_init(list); }
#[unsafe(no_mangle)]
pub extern "C" fn list_is_empty_c(list: *mut ListHead) -> i32 {
    list_is_empty(list) as i32
}
#[unsafe(no_mangle)]
pub extern "C" fn list_pushfront_c(item: *mut ListHead, list: *mut ListHead) {
    list_pushfront(item, list);
}
#[unsafe(no_mangle)]
pub extern "C" fn list_pushback_c(item: *mut ListHead, list: *mut ListHead) {
    list_pushback(item, list);
}
#[unsafe(no_mangle)]
pub extern "C" fn list_append_front_c(dst: *mut ListHead, src: *mut ListHead) {
    list_append_front(dst, src);
}
#[unsafe(no_mangle)]
pub extern "C" fn list_append_back_c(dst: *mut ListHead, src: *mut ListHead) {
    list_append_back(dst, src);
}
#[unsafe(no_mangle)]
pub extern "C" fn list_rotate_forward_c(list: *mut ListHead) {
    list_rotate_forward(list);
}
#[unsafe(no_mangle)]
pub extern "C" fn list_rotate_backward_c(list: *mut ListHead) {
    list_rotate_backward(list);
}
#[unsafe(no_mangle)]
pub extern "C" fn list_remove_c(item: *mut ListHead) { list_remove(item); }
#[unsafe(no_mangle)]
pub extern "C" fn list_pop_c(list: *mut ListHead) -> *mut ListHead { list_pop(list) }
#[unsafe(no_mangle)]
pub extern "C" fn list_find_c(list: *mut ListHead, item: *mut ListHead) -> i32 {
    list_find(list, item) as i32
}
