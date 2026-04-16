//! 架构抽象辅助工具
//!
//! 提供与架构无关的 CPU 相关能力。

use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

// CPU ID 接口

/// 获取当前 CPU 编号（统一接口）。
#[inline(always)]
pub fn cpu_id() -> usize {
    // 统一走内核提供的查询入口，避免 domain 侧直接绑定某个架构的 percpu 细节。
    corelib::current_cpu_id()
}

// Per-CPU 数据结构

/// Per-CPU 本地数据包装
///
/// 该类型提供每核独占的数据存储，不依赖原子操作。
pub struct CpuLocal<T>(UnsafeCell<T>);

/// 安全性：每个 CPU 只访问自己的 CpuLocal 实例
unsafe impl<T> Sync for CpuLocal<T> {}

impl<T> CpuLocal<T> {
    /// 用给定初值创建 CpuLocal
    pub const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    /// 获取内部值的可变引用
    ///
    /// # Safety
    /// 只要中断关闭，或调用方保证同核无并发访问，即可安全使用。
    #[allow(clippy::mut_from_ref)]
    pub fn as_mut(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }

    /// 获取内部值的不可变引用
    pub fn get(&self) -> &T {
        unsafe { &*self.0.get() }
    }

    /// 获取内部值的裸指针
    pub fn as_ptr(&self) -> *mut T {
        self.0.get()
    }
}

impl<T> Deref for CpuLocal<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0.get() }
    }
}

impl<T> DerefMut for CpuLocal<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

// 内存屏障工具

/// 全内存屏障
#[inline(always)]
pub fn memory_barrier() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        core::arch::asm!("fence iorw, iorw", options(nostack, preserves_flags));
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("mfence", options(nostack, preserves_flags));
    }
}

/// 读屏障
#[inline(always)]
pub fn read_barrier() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        core::arch::asm!("fence ir, ir", options(nostack, preserves_flags));
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("lfence", options(nostack, preserves_flags));
    }
}

/// 写屏障
#[inline(always)]
pub fn write_barrier() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        core::arch::asm!("fence ow, ow", options(nostack, preserves_flags));
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("sfence", options(nostack, preserves_flags));
    }
}

/// 编译器屏障（禁止编译器重排）
#[inline(always)]
pub fn compiler_barrier() {
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}
