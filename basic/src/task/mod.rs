//! Task context and trap frame definitions
//!
//! Architecture-specific trap frame implementations.
//! Each architecture uses its native naming conventions.

// Compile-time check: ensure valid architecture
#[cfg(not(any(target_arch = "riscv64", target_arch = "x86_64")))]
compile_error!("Unsupported architecture! Only riscv64 and x86_64 are supported.");

#[cfg(target_arch = "riscv64")]
use arch::{PRIVILEGE_USER, ProcessorStatus};
use memory_addr::{PhysAddr, VirtAddr};
pub use task_meta::TaskContext;

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

// ============================================================================
// RISC-V TrapFrame
// ============================================================================

#[cfg(target_arch = "riscv64")]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TrapFrame {
    /// General purpose registers x0-x31
    x: [usize; 32],
    /// Supervisor exception program counter
    sepc: usize,
    /// Kernel SATP value
    k_satp: usize,
    /// Kernel stack pointer
    k_sp: usize,
    /// Trap handler address
    trap_handler: usize,
    /// Hart ID
    hart_id: usize,
    /// Supervisor status (using unified ProcessorStatus type)
    pub sstatus: ProcessorStatus,
    /// Floating point status
    fg: [usize; 2],
}

#[cfg(target_arch = "riscv64")]
impl TrapFrame {
    fn init_for_task(
        entry: usize,
        sp: usize,
        k_satp: usize,
        k_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = ProcessorStatus::read_current();
        sstatus.enable_interrupts();
        sstatus.set_privilege(PRIVILEGE_USER);
        sstatus.disable_interrupts();
        let mut res = Self {
            x: [0; 32],
            sepc: entry,
            k_satp,
            k_sp,
            trap_handler,
            hart_id: 0,
            sstatus,
            fg: [0; 2],
        };
        res.x[2] = sp; // sp register
        res
    }

    pub fn new_user(entry: VirtAddr, sp: VirtAddr, k_sp: VirtAddr) -> Self {
        let kernel_satp = corelib::kernel_satp();
        let user_trap_vector = corelib::trap_from_user();
        Self::init_for_task(
            entry.as_usize(),
            sp.as_usize(),
            kernel_satp,
            k_sp.as_usize(),
            user_trap_vector,
        )
    }

    pub fn update_k_sp(&mut self, val: VirtAddr) {
        self.k_sp = val.as_usize();
    }

    pub fn update_user_sp(&mut self, val: VirtAddr) {
        self.x[2] = val.as_usize();
    }

    pub fn update_tp(&mut self, val: VirtAddr) {
        self.x[4] = val.as_usize();
    }

    pub fn update_result(&mut self, val: usize) {
        self.x[10] = val;
    }

    pub fn sepc(&self) -> VirtAddr {
        VirtAddr::from(self.sepc)
    }

    pub fn update_sepc(&mut self, val: VirtAddr) {
        self.sepc = val.as_usize();
    }

    pub fn from_raw_phy_ptr(ptr: PhysAddr) -> &'static mut Self {
        unsafe { &mut *(ptr.as_usize() as *mut usize as *mut Self) }
    }

    pub fn parameters(&self) -> [usize; 7] {
        [
            self.x[17], self.x[10], self.x[11], self.x[12], self.x[13], self.x[14], self.x[15],
        ]
    }
}

// ============================================================================
// x86-64 TrapFrame
// ============================================================================

#[cfg(target_arch = "x86_64")]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TrapFrame {
    // Callee-saved registers
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbp: usize,
    pub rbx: usize,
    // Scratch registers
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,
    // Pushed by CPU on interrupt/exception
    pub vector: usize,
    pub error_code: usize,
    pub rip: usize,
    pub cs: usize,
    pub rflags_val: usize,
    pub rsp: usize,
    pub ss: usize,
    // Kernel state（k_cr3@176, k_sp@184, trap_handler@192, cpu_id@200，合计 208 字节，16 字节对齐）
    k_cr3: usize,
    k_sp: usize,
    trap_handler: usize,
    cpu_id: usize,
    /// FPU/SSE 状态区（fxsave64/fxrstor64，512 字节，偏移 208）
    fx_state: [u64; 64],
}

#[cfg(target_arch = "x86_64")]
impl TrapFrame {
    pub const OFFSET_K_CR3: usize = core::mem::offset_of!(TrapFrame, k_cr3);
    pub const OFFSET_K_SP: usize = core::mem::offset_of!(TrapFrame, k_sp);
    pub const OFFSET_TRAP_HANDLER: usize = core::mem::offset_of!(TrapFrame, trap_handler);
    pub const USER_CONTEXT_SIZE: usize = Self::OFFSET_K_CR3;

    fn init_for_task(
        entry: usize,
        sp: usize,
        k_cr3: usize,
        k_sp: usize,
        trap_handler: usize,
    ) -> Self {
        Self {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbp: 0,
            rbx: 0,
            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rsi: 0,
            rdi: 0,
            rdx: 0,
            rcx: 0,
            rax: 0,
            vector: 0,
            error_code: 0,
            rip: entry,
            cs: 0x23,        // User code segment (ring 3)
            rflags_val: 0x202,   // IF flag set
            rsp: sp,
            ss: 0x1b,        // User data segment (ring 3)
            k_cr3,
            k_sp,
            trap_handler,
            cpu_id: 0,
            fx_state: Self::INITIAL_FX_STATE,
        }
    }

    pub fn new_user(entry: VirtAddr, sp: VirtAddr, k_sp: VirtAddr) -> Self {
        let kernel_satp = corelib::kernel_satp();
        let user_trap_vector = corelib::trap_from_user();
        Self::init_for_task(
            entry.as_usize(),
            sp.as_usize(),
            kernel_satp,
            k_sp.as_usize(),
            user_trap_vector,
        )
    }

    pub fn update_k_sp(&mut self, val: VirtAddr) {
        self.k_sp = val.as_usize();
    }

    pub fn kernel_sp(&self) -> VirtAddr {
        VirtAddr::from(self.k_sp)
    }

    pub fn update_user_sp(&mut self, val: VirtAddr) {
        self.rsp = val.as_usize();
    }

    pub fn update_tp(&mut self, val: VirtAddr) {
        // x86-64 uses fs/gs for TLS, not a general register
        // For now, store in r15 as a placeholder
        self.r15 = val.as_usize();
    }

    pub fn update_result(&mut self, val: usize) {
        self.rax = val;
    }

    pub fn sepc(&self) -> VirtAddr {
        VirtAddr::from(self.rip)
    }

    pub fn update_sepc(&mut self, val: VirtAddr) {
        self.rip = val.as_usize();
    }

    pub fn from_raw_phy_ptr(ptr: PhysAddr) -> &'static mut Self {
        unsafe { &mut *(ptr.as_usize() as *mut usize as *mut Self) }
    }

    /// Get syscall parameters
    /// x86-64 syscall convention: rax=syscall#, rdi, rsi, rdx, r10, r8, r9
    pub fn parameters(&self) -> [usize; 7] {
        [
            self.rax, self.rdi, self.rsi, self.rdx, self.r10, self.r8, self.r9,
        ]
    }

    /// x86-64 标准初始 FPU/SSE 状态（fxsave64 格式，512 字节）：
    /// - FCW = 0x037F（屏蔽全部 x87 异常，80 位精度）
    /// - MXCSR = 0x1F80（屏蔽全部 SSE 异常）
    const INITIAL_FX_STATE: [u64; 64] = {
        let mut s = [0u64; 64];
        s[0] = 0x037F;  // bytes 0-1: FCW
        s[3] = 0x1F80;  // bytes 24-27: MXCSR（小端：低 32 位）
        s
    };

    /// 将当前 CPU 的 FPU/SSE 状态保存到此 TrapFrame。
    /// 用于用户态进入内核时保存用户的浮点上下文。
    #[inline]
    pub fn save_fx_state(&mut self) {
        unsafe {
            core::arch::asm!(
                "fxsave64 [{ptr}]",
                ptr = in(reg) self.fx_state.as_mut_ptr(),
                options(nostack, preserves_flags)
            );
        }
    }

    /// 从此 TrapFrame 恢复 FPU/SSE 状态到 CPU。
    /// 用于返回用户态前恢复用户的浮点上下文。
    #[inline]
    pub fn restore_fx_state(&self) {
        unsafe {
            core::arch::asm!(
                "fxrstor64 [{ptr}]",
                ptr = in(reg) self.fx_state.as_ptr(),
                options(nostack, preserves_flags)
            );
        }
    }
}
