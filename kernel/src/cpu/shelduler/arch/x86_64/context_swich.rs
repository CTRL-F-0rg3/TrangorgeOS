pub unsafe fn switch_to(next_task: *mut TaskStruct) {
    // Save the current task's context
    let current_task = get_current_task();
    if !current_task.is_null() {
        save_context(&mut (*current_task).context);
    }

    // Load the next task's context
    load_context(&(*next_task).context);

    // Update the current task pointer
    set_current_task(next_task);
}

// TODO: Implement the save_context and load_context functions using inline assembly to save and restore the CPU registers.