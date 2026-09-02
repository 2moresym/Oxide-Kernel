//! Low-level x86_64 context switching for kernel tasks.

use core::arch::global_asm;

// Save the callee-saved registers and stack pointer of the current task,
// restore the next task's state, then `ret` into the next task.
global_asm!(r#"
.global oxide_context_switch
.type oxide_context_switch, @function
oxide_context_switch:
    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15
    mov [rdi], rsp
    mov rsp, rsi
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret
"#);

unsafe extern "C" {
    pub fn oxide_context_switch(old_rsp: *mut u64, new_rsp: u64);
}
