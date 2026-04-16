use arch::{PRIVILEGE_USER, ProcessorStatus};
use memory_addr::{PhysAddr, VirtAddr};

/// RISC-V 用户态陷阱帧。
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
    /// 内核 percpu 基址
    kernel_percpu: usize,
    /// S 态状态
    pub sstatus: ProcessorStatus,
    /// 浮点状态
    fg: [usize; 2],
}

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
            kernel_percpu: 0,
            sstatus,
            fg: [0; 2],
        };
        res.x[2] = sp;
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

    pub fn update_tls(&mut self, val: VirtAddr) {
        self.x[4] = val.as_usize();
    }

    pub fn update_result(&mut self, val: usize) {
        self.x[10] = val;
    }

    pub fn user_pc(&self) -> VirtAddr {
        VirtAddr::from(self.sepc)
    }

    pub fn update_user_pc(&mut self, val: VirtAddr) {
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
