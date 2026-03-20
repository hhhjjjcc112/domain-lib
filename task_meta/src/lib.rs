//! Task metadata definitions
//!
//! Architecture-independent task context and metadata structures.

#![no_std]

mod continuation;

// Compile-time check: ensure valid architecture
#[cfg(not(any(target_arch = "riscv64", target_arch = "x86_64")))]
compile_error!("Unsupported architecture! Only riscv64 and x86_64 are supported.");

// ============================================================================
// RISC-V TaskContext
// ============================================================================

#[cfg(target_arch = "riscv64")]
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct TaskContext {
    /// Return address (ra)
    ra: usize,
    /// Stack pointer (sp)
    sp: usize,
    /// Callee-saved registers s0-s11
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

// ============================================================================
// x86-64 TaskContext
// ============================================================================

#[cfg(target_arch = "x86_64")]
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct FpSimdState {
    pub data: [u8; 512],
}

#[cfg(target_arch = "x86_64")]
impl FpSimdState {
    pub const fn new() -> Self {
        let mut data = [0u8; 512];
        data[0] = 0x7f;
        data[1] = 0x03;
        data[24] = 0x80;
        data[25] = 0x1f;
        Self { data }
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
    /// Return address (rip after ret)
    rip: usize,
    /// Stack pointer (rsp)
    rsp: usize,
    /// Callee-saved registers: rbx, rbp, r12-r15
    rbx: usize,
    rbp: usize,
    r12: usize,
    r13: usize,
    r14: usize,
    r15: usize,
    /// 任务级 FP/SIMD 状态（fxsave64/fxrstor64 格式，16 字节对齐）
    fp_simd: FpSimdState,
}

#[cfg(target_arch = "x86_64")]
impl TaskContext {
    pub const fn new(rip: usize, rsp: usize) -> Self {
        Self {
            rip,
            rsp,
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            fp_simd: FpSimdState::new(),
        }
    }

    pub const fn empty() -> Self {
        Self {
            rip: 0,
            rsp: 0,
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            fp_simd: FpSimdState::new(),
        }
    }

    pub fn set_sp(&mut self, sp: usize) {
        self.rsp = sp;
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
    // other information
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
    /// Create a new TaskMeta
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
