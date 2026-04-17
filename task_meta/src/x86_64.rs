use x86_64::{
    VirtAddr,
    registers::model_specific::{FsBase, KernelGsBase},
};

/// x86_64 FXSAVE/FXRSTOR 状态块。
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

impl Default for FpSimdState {
    fn default() -> Self {
        Self::new()
    }
}

/// x86_64 任务切换上下文。
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, align(16))]
pub struct TaskContext {
    /// 调度入口使用的内核栈顶
    kstack_top: usize,
    /// 栈指针（rsp）
    rsp: usize,
    /// TLS 对应 FS 基址
    fs_base: usize,
    /// 用户态 GS 基址缓存
    gs_base: usize,
    /// 任务级 FP/SIMD 状态
    fp_simd: FpSimdState,
}

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

impl TaskContext {
    pub fn new(entry: usize, kstack_top: usize) -> Self {
        let mut ctx = Self {
            kstack_top: 0,
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
    pub fn save_fsgs(&mut self) {
        self.fs_base = FsBase::read().as_u64() as usize;
        self.gs_base = KernelGsBase::read().as_u64() as usize;
    }

    #[inline]
    pub fn restore_fsgs(&self) {
        FsBase::write(VirtAddr::new(self.fs_base as u64));
        KernelGsBase::write(VirtAddr::new(self.gs_base as u64));
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
