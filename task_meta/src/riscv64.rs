/// RISC-V 任务切换上下文。
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
