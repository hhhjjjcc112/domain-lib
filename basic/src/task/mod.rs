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
#[cfg(target_arch = "x86_64")]
use x86_64::{
    PrivilegeLevel,
    registers::rflags::RFlags,
    structures::gdt::SegmentSelector,
};

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

// x86_64 trap上下文
#[cfg(target_arch = "x86_64")]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TrapFrame {
    // 内核字段，供 trap handler 使用
    k_cr3: usize,
    k_sp: usize,

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
}

#[cfg(target_arch = "x86_64")]
impl TrapFrame {
    const USER_CODE_SELECTOR: usize = SegmentSelector::new(6, PrivilegeLevel::Ring3).0 as usize;
    const USER_DATA_SELECTOR: usize = SegmentSelector::new(5, PrivilegeLevel::Ring3).0 as usize;
    const USER_RFLAGS: usize = RFlags::INTERRUPT_FLAG.bits() as usize | 0x2;

    pub const OFFSET_K_CR3: usize = core::mem::offset_of!(TrapFrame, k_cr3);
    pub const OFFSET_K_SP: usize = core::mem::offset_of!(TrapFrame, k_sp);
    pub const RSP0_TOP_SIZE: usize = core::mem::size_of::<TrapFrame>();
    pub const USER_CONTEXT_SIZE: usize = Self::RSP0_TOP_SIZE;

    fn init_for_task(
        entry: usize,
        sp: usize,
        k_cr3: usize,
        k_sp: usize,
    ) -> Self {
        Self {
            k_cr3,
            k_sp,
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
            cs: Self::USER_CODE_SELECTOR,
            rflags_val: Self::USER_RFLAGS,
            rsp: sp,
            ss: Self::USER_DATA_SELECTOR,
        }
    }

    pub fn new_user(entry: VirtAddr, sp: VirtAddr, k_sp: VirtAddr) -> Self {
        let kernel_satp = corelib::kernel_satp();
        Self::init_for_task(
            entry.as_usize(),
            sp.as_usize(),
            kernel_satp,
            k_sp.as_usize(),
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
}
