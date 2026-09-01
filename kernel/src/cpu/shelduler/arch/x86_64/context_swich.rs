pub unsafe fn switch_to(next_task: *mut TaskStruct) {
    let current_task = get_current_task();
    if !current_task.is_null() {
        save_context(&mut (*current_task).context);
    }


    load_context(&(*next_task).context);
    set_current_task(next_task);
}
