//! 任务元数据定义
//!
//! 提供与架构无关的任务上下文与调度元数据结构。

#![no_std]

mod continuation;

// 编译期检查：仅支持 riscv64 与 x86_64
#[cfg(not(any(target_arch = "riscv64", target_arch = "x86_64")))]
compile_error!("Unsupported architecture! Only riscv64 and x86_64 are supported.");

// RISC-V 任务上下文

#[cfg(target_arch = "riscv64")]
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct TaskContext {
    /// 返回地址（ra）
    ra: usize,
    /// 栈指针（sp）
    sp: usize,
    /// 被调用者保存寄存器 s0-s11
    s: [usize; 12],
}

#[cfg(target_arch = "riscv64")]
impl TaskContext {
    pub const fn new(ra: usize, sp: usize) -> Self {
        Self { ra, sp, s: [0; 12] }
    }

    pub const fn empty() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    pub fn set_sp(&mut self, sp: usize) {
        self.sp = sp;
    }
}

// x86-64 任务上下文

#[cfg(target_arch = "x86_64")]
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct FpSimdState {
    pub fcw: u16,
    pub fsw: u16,
    pub ftw: u16,
    pub fop: u16,
    pub fip: u64,
    pub fdp: u64,
    pub mxcsr: u32,
    pub mxcsr_mask: u32,
    pub st: [u64; 16],
    pub xmm: [u64; 32],
    padding: [u64; 12],
}

#[cfg(target_arch = "x86_64")]
impl FpSimdState {
    pub const fn new() -> Self {
        Self {
            fcw: 0x37f,
            fsw: 0,
            ftw: 0xffff,
            fop: 0,
            fip: 0,
            fdp: 0,
            mxcsr: 0x1f80,
            mxcsr_mask: 0,
            st: [0; 16],
            xmm: [0; 32],
            padding: [0; 12],
        }
    }
}


#[cfg(target_arch = "x86_64")]
impl Default for FpSimdState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_arch = "x86_64")]
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, align(16))]
pub struct TaskContext {
    /// 内核栈顶（用于更新 TSS.rsp0）
    kstack_top: usize,
    /// 栈指针（rsp）
    rsp: usize,
    /// TLS 对应 FS 基址
    fs_base: usize,
    /// 用户态 GS 基址（写 IA32_KERNEL_GS_BASE）
    gs_base: usize,
    /// 任务级 FP/SIMD 状态（fxsave64/fxrstor64 格式，16 字节对齐）
    fp_simd: FpSimdState,
}

#[cfg(target_arch = "x86_64")]
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct ContextSwitchFrame {
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,
    rbx: usize,
    rbp: usize,
    rip: usize,
}

#[cfg(target_arch = "x86_64")]
impl TaskContext {
    pub fn new(entry: usize, kstack_top: usize) -> Self {
        let mut ctx = Self {
            kstack_top: 0,
            // 先暂存入口地址，后续 set_sp 会用它构造首帧。
            rsp: entry,
            fs_base: 0,
            gs_base: 0,
            fp_simd: FpSimdState::new(),
        };
        if kstack_top != 0 {
            ctx.init_stack_frame(entry, kstack_top);
        }
        ctx
    }

    pub const fn empty() -> Self {
        Self {
            kstack_top: 0,
            rsp: 0,
            fs_base: 0,
            gs_base: 0,
            fp_simd: FpSimdState::new(),
        }
    }

    pub fn set_sp(&mut self, sp: usize) {
        // 若上下文尚未初始化切换栈帧，rsp 中暂存的是入口地址。
        let entry = if self.kstack_top == 0 { self.rsp } else { 0 };
        if entry != 0 {
            self.init_stack_frame(entry, sp);
        } else {
            self.kstack_top = sp;
        }
    }

    #[inline]
    pub fn kstack_top(&self) -> usize {
        self.kstack_top
    }

    #[inline]
    pub fn fs_base(&self) -> usize {
        self.fs_base
    }

    #[inline]
    pub fn gs_base(&self) -> usize {
        self.gs_base
    }

    #[inline]
    pub fn set_fs_base(&mut self, fs_base: usize) {
        self.fs_base = fs_base;
    }

    #[inline]
    pub fn set_gs_base(&mut self, gs_base: usize) {
        self.gs_base = gs_base;
    }

    #[inline]
    pub fn save_fp_simd(&mut self) {
        unsafe {
            core::arch::x86_64::_fxsave64(&mut self.fp_simd as *mut _ as *mut u8);
        }
    }

    #[inline]
    pub fn restore_fp_simd(&self) {
        unsafe {
            core::arch::x86_64::_fxrstor64(&self.fp_simd as *const _ as *const u8);
        }
    }

    fn init_stack_frame(&mut self, entry: usize, kstack_top: usize) {
        unsafe {
            let frame_ptr = (kstack_top as *mut ContextSwitchFrame).sub(1);
            core::ptr::write(
                frame_ptr,
                ContextSwitchFrame {
                    rip: entry,
                    ..ContextSwitchFrame::default()
                },
            );
            self.rsp = frame_ptr as usize;
        }
        self.kstack_top = kstack_top;
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TaskBasicInfo {
    pub tid: usize,
    pub status: TaskStatus,
    pub context: TaskContext,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TaskMeta {
    pub task_basic_info: TaskBasicInfo,
    pub scheduling_info: TaskSchedulingInfo,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct TaskSchedulingInfo {
    pub tid: usize,
    pub nice: i8,
    // 其他调度信息
    pub cpus_allowed: usize,
}

impl TaskSchedulingInfo {
    pub const fn new(tid: usize, nice: i8, cpu_allowed: usize) -> Self {
        Self {
            tid,
            nice,
            cpus_allowed: cpu_allowed,
        }
    }

    pub fn set_nice(&mut self, nice: i8) {
        self.nice = nice;
    }

    pub fn nice(&self) -> i8 {
        self.nice
    }
}

impl TaskMeta {
    /// 创建新的 TaskMeta
    pub const fn new(basic_info: TaskBasicInfo, scheduling_info: TaskSchedulingInfo) -> Self {
        Self {
            task_basic_info: basic_info,
            scheduling_info,
        }
    }
    pub fn basic_info(&self) -> TaskBasicInfo {
        self.task_basic_info
    }
    pub fn scheduling_info(&self) -> TaskSchedulingInfo {
        self.scheduling_info
    }
}

impl TaskBasicInfo {
    pub const fn new(tid: usize, context: TaskContext) -> Self {
        Self {
            tid,
            status: TaskStatus::Ready,
            context,
        }
    }

    pub fn tid(&self) -> usize {
        self.tid
    }
    pub fn get_context_raw_ptr(&self) -> *const TaskContext {
        &self.context as *const TaskContext as *mut _
    }
    pub fn get_context_raw_mut_ptr(&mut self) -> *mut TaskContext {
        &mut self.context as *mut TaskContext
    }
    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
    }
    pub fn status(&self) -> TaskStatus {
        self.status
    }

    pub fn task_context(&mut self) -> &mut TaskContext {
        &mut self.context
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub enum TaskStatus {
    /// 就绪态
    #[default]
    Ready,
    /// 运行态
    Running,
    /// 等待一个事件
    Waiting,
    /// 僵尸态，等待父进程回收资源
    Zombie,
    /// 终止态
    Terminated,
}

#[derive(Debug, Copy, Clone)]
pub enum TaskOperation {
    Create(TaskMeta),
    Wait,
    Wakeup(usize),
    Yield,
    Exit,
    Remove(usize),
    Current,
    ExitOver(usize),
    SetPriority(i8),
    GetPriority,
}

#[derive(Debug, Copy, Clone)]
pub enum OperationResult {
    Current(Option<usize>),
    KstackTop(usize),
    Null,
    ExitOver(bool),
    Priority(i8),
}

impl OperationResult {
    pub fn current_tid(&self) -> Option<usize> {
        match self {
            OperationResult::Current(tid) => *tid,
            _ => panic!("OperationResult is not Current"),
        }
    }

    pub fn kstack_top(&self) -> usize {
        match self {
            OperationResult::KstackTop(top) => *top,
            _ => panic!("OperationResult is not KstackTop"),
        }
    }
    pub fn is_exit_over(&self) -> bool {
        match self {
            OperationResult::ExitOver(is_exit) => *is_exit,
            _ => panic!("OperationResult is not ExitOver"),
        }
    }
    pub fn priority(&self) -> i8 {
        match self {
            OperationResult::Priority(priority) => *priority,
            _ => panic!("OperationResult is not Priority"),
        }
    }
}
