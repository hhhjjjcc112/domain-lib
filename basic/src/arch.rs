//! Architecture abstraction helpers
//!
//! Provides architecture-neutral CPU-related utilities.

use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

// ============================================================================
// CPU ID functions
// ============================================================================

/// Get current CPU ID (unified interface)
#[inline(always)]
pub fn cpu_id() -> usize {
    arch::cpu_id()
}

/// Get current CPU ID (RISC-V compatibility alias)
#[inline(always)]
pub fn hart_id() -> usize {
    arch::cpu_id()
}

/// Get current CPU ID (x86-64 compatibility alias)
#[inline(always)]
pub fn apic_id() -> usize {
    arch::cpu_id()
}

// ============================================================================
// Per-CPU data structure
// ============================================================================

/// Per-CPU local data wrapper
/// 
/// This type provides per-CPU data storage without requiring atomic operations,
/// as each CPU only accesses its own instance.
pub struct CpuLocal<T>(UnsafeCell<T>);

/// Safety: Each CPU only accesses its own CpuLocal instance
unsafe impl<T> Sync for CpuLocal<T> {}

impl<T> CpuLocal<T> {
    /// Create a new CpuLocal with the given initial value
    pub const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    /// Get a mutable reference to the inner value
    /// 
    /// # Safety
    /// This is safe as long as interrupts are disabled or the caller
    /// ensures no concurrent access from the same CPU.
    #[allow(clippy::mut_from_ref)]
    pub fn as_mut(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
    
    /// Get an immutable reference to the inner value
    pub fn get(&self) -> &T {
        unsafe { &*self.0.get() }
    }
    
    /// Get a raw pointer to the inner value
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

// ============================================================================
// Architecture detection
// ============================================================================

/// Check if running on RISC-V architecture
#[cfg(target_arch = "riscv64")]
pub const fn is_riscv() -> bool {
    true
}

#[cfg(not(target_arch = "riscv64"))]
pub const fn is_riscv() -> bool {
    false
}

/// Check if running on x86-64 architecture
#[cfg(target_arch = "x86_64")]
pub const fn is_x86_64() -> bool {
    true
}

#[cfg(not(target_arch = "x86_64"))]
pub const fn is_x86_64() -> bool {
    false
}

// ============================================================================
// Memory barrier utilities
// ============================================================================

/// Full memory barrier
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

/// Read memory barrier
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

/// Write memory barrier
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

/// Compiler barrier (prevents compiler reordering)
#[inline(always)]
pub fn compiler_barrier() {
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

