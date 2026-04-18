//! 任务元数据定义
//!
//! 提供与架构无关的任务上下文与调度元数据结构。

#![no_std]

mod continuation;
#[cfg(target_arch = "riscv64")]
mod riscv64;
#[cfg(target_arch = "x86_64")]
mod x86_64;

// 编译期检查：仅支持 riscv64 与 x86_64
#[cfg(not(any(target_arch = "riscv64", target_arch = "x86_64")))]
compile_error!("Unsupported architecture! Only riscv64 and x86_64 are supported.");

#[cfg(target_arch = "riscv64")]
pub use self::riscv64::TaskContext;
#[cfg(target_arch = "x86_64")]
pub use self::x86_64::TaskContext;

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
    GetCpusAllowed,
    ExitOver(usize),
    SetPriority(i8),
    GetPriority,
    #[cfg(target_arch = "x86_64")]
    SetUserFsBase(usize),
    #[cfg(target_arch = "x86_64")]
    GetUserFsBase,
    #[cfg(target_arch = "x86_64")]
    SetUserGsBase(usize),
    #[cfg(target_arch = "x86_64")]
    GetUserGsBase,
}

#[derive(Debug, Copy, Clone)]
pub enum OperationResult {
    Current(Option<usize>),
    CpusAllowed(usize),
    KstackTop(usize),
    Null,
    ExitOver(bool),
    Priority(i8),
    #[cfg(target_arch = "x86_64")]
    FsBase(usize),
    #[cfg(target_arch = "x86_64")]
    GsBase(usize),
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

    pub fn cpus_allowed(&self) -> usize {
        match self {
            OperationResult::CpusAllowed(cpus_allowed) => *cpus_allowed,
            _ => panic!("OperationResult is not CpusAllowed"),
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

    #[cfg(target_arch = "x86_64")]
    pub fn fs_base(&self) -> usize {
        match self {
            OperationResult::FsBase(fs_base) => *fs_base,
            _ => panic!("OperationResult is not FsBase"),
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn gs_base(&self) -> usize {
        match self {
            OperationResult::GsBase(gs_base) => *gs_base,
            _ => panic!("OperationResult is not GsBase"),
        }
    }
}
