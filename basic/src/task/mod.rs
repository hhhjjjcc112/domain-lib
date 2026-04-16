//! 任务上下文与陷阱帧定义
//!
//! 提供按架构实现的陷阱帧，并统一对外接口。

// 编译期检查：仅支持 riscv64 与 x86_64
#[cfg(not(any(target_arch = "riscv64", target_arch = "x86_64")))]
compile_error!("Unsupported architecture! Only riscv64 and x86_64 are supported.");

#[cfg(target_arch = "riscv64")]
mod riscv64;
#[cfg(target_arch = "x86_64")]
mod x86_64;

use memory_addr::VirtAddr;
pub use task_meta::TaskContext;

#[cfg(target_arch = "riscv64")]
pub use self::riscv64::TrapFrame;
#[cfg(target_arch = "x86_64")]
pub use self::x86_64::TrapFrame;

pub trait TaskContextExt {
    fn new_user(k_sp: VirtAddr) -> Self;
    fn new_kernel(func_ptr: *const (), k_sp: VirtAddr) -> Self;
}

impl TaskContextExt for TaskContext {
    fn new_user(k_sp: VirtAddr) -> Self {
        TaskContext::new(corelib::trap_to_user(), k_sp.as_usize())
    }
    fn new_kernel(func_ptr: *const (), k_sp: VirtAddr) -> Self {
        TaskContext::new(func_ptr as usize, k_sp.as_usize())
    }
}
