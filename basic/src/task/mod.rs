//! 任务上下文与陷阱帧定义
//!
//! 提供按架构实现的陷阱帧，并统一对外接口。

// 编译期检查：仅支持 riscv64 与 x86_64
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

// RISC-V 陷阱帧

#[cfg(target_arch = "riscv64")]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TrapFrame {
    /// 通用寄存器 x0-x31
    x: [usize; 32],
    /// S 态异常返回地址
    sepc: usize,
    /// 内核页表令牌
    k_satp: usize,
    /// 内核栈指针
    k_sp: usize,
    /// 陷阱处理入口地址
    trap_handler: usize,
    /// CPU 编号
    cpu_id: usize,
    /// S 态状态（统一 ProcessorStatus 类型）
    pub sstatus: ProcessorStatus,
    /// 浮点状态
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
            cpu_id: 0,
            sstatus,
            fg: [0; 2],
        };
        res.x[2] = sp; // sp 寄存器
        res
    }

    pub fn new_user(entry: VirtAddr, sp: VirtAddr, k_sp: VirtAddr) -> Self {
        let kernel_page_table_token = corelib::kernel_page_table_token();
        let user_trap_vector = corelib::trap_from_user();
        Self::init_for_task(
            entry.as_usize(),
            sp.as_usize(),
            kernel_page_table_token,
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

// x86_64 陷阱帧
#[cfg(target_arch = "x86_64")]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TrapFrame {
    // 内核字段，供 trap handler 使用
    k_cr3: usize,
    k_sp: usize,

    // 被调用者保存寄存器
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbp: usize,
    pub rbx: usize,
    // 临时寄存器
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,
    // CPU 在中断/异常时自动压栈的部分
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
        let kernel_page_table_token = corelib::kernel_page_table_token();
        Self::init_for_task(
            entry.as_usize(),
            sp.as_usize(),
            kernel_page_table_token,
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
        // x86-64 用 fs/gs 承载 TLS，不使用通用寄存器。
        // 当前先临时写入 r15 占位。
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

    /// 获取系统调用参数
    /// x86-64 调用约定：rax=号，rdi/rsi/rdx/r10/r8/r9 为参数
    pub fn parameters(&self) -> [usize; 7] {
        [
            self.rax, self.rdi, self.rsi, self.rdx, self.r10, self.r8, self.r9,
        ]
    }
}
