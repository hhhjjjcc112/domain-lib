use memory_addr::{PhysAddr, VirtAddr};
use x86_64::{PrivilegeLevel, registers::rflags::RFlags, structures::gdt::SegmentSelector};

/// x86_64 用户态陷阱帧。
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TrapFrame {
    // 内核字段，仅供 x86_64 trap 路径使用。
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
    // CPU 自动压栈部分
    pub vector: usize,
    pub error_code: usize,
    pub rip: usize,
    pub cs: usize,
    pub rflags_val: usize,
    pub rsp: usize,
    pub ss: usize,
}

impl TrapFrame {
    const USER_CODE_SELECTOR: usize = SegmentSelector::new(6, PrivilegeLevel::Ring3).0 as usize;
    const USER_DATA_SELECTOR: usize = SegmentSelector::new(5, PrivilegeLevel::Ring3).0 as usize;
    const USER_RFLAGS: usize = RFlags::INTERRUPT_FLAG.bits() as usize | 0x2;

    pub const OFFSET_K_CR3: usize = core::mem::offset_of!(TrapFrame, k_cr3);
    pub const OFFSET_K_SP: usize = core::mem::offset_of!(TrapFrame, k_sp);
    pub const RSP0_TOP_SIZE: usize = core::mem::size_of::<TrapFrame>();
    pub const USER_CONTEXT_SIZE: usize = Self::RSP0_TOP_SIZE;

    fn init_for_task(entry: usize, sp: usize, k_cr3: usize, k_sp: usize) -> Self {
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

    pub fn kernel_page_table_token(&self) -> usize {
        self.k_cr3
    }

    pub fn update_user_sp(&mut self, val: VirtAddr) {
        self.rsp = val.as_usize();
    }

    pub fn update_tls(&mut self, val: VirtAddr) {
        let _ = val;
    }

    pub fn update_result(&mut self, val: usize) {
        self.rax = val;
    }

    pub fn user_pc(&self) -> VirtAddr {
        VirtAddr::from(self.rip)
    }

    pub fn update_user_pc(&mut self, val: VirtAddr) {
        self.rip = val.as_usize();
    }

    pub fn from_raw_phy_ptr(ptr: PhysAddr) -> &'static mut Self {
        unsafe { &mut *(ptr.as_usize() as *mut usize as *mut Self) }
    }

    /// x86_64 syscall ABI：rax=号，rdi/rsi/rdx/r10/r8/r9=参数。
    pub fn parameters(&self) -> [usize; 7] {
        [
            self.rax, self.rdi, self.rsi, self.rdx, self.r10, self.r8, self.r9,
        ]
    }
}
