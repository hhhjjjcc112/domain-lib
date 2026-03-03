//! User memory access control trait

use super::Arch;

/// User memory access control trait
///
/// Provides methods to control kernel access to user memory.
/// - RISC-V: Uses SUM (Supervisor User Memory access) bit in sstatus
/// - x86-64: Uses SMAP (Supervisor Mode Access Prevention) via AC flag
pub trait MemoryAccessIf {
    /// Allow kernel to access user memory
    fn allow_user_memory_access();
    
    /// Disallow kernel access to user memory
    fn disallow_user_memory_access();
}

// ============================================================================
// RISC-V implementation
// ============================================================================

#[cfg(target_arch = "riscv64")]
impl MemoryAccessIf for Arch {
    #[inline]
    fn allow_user_memory_access() {
        crate::riscv::allow_access_user_memory()
    }
    
    #[inline]
    fn disallow_user_memory_access() {
        crate::riscv::disallow_access_user_memory()
    }
}

// ============================================================================
// x86-64 implementation
// ============================================================================

#[cfg(target_arch = "x86_64")]
impl MemoryAccessIf for Arch {
    #[inline]
    fn allow_user_memory_access() {
        crate::x86_64::allow_access_user_memory()
    }
    
    #[inline]
    fn disallow_user_memory_access() {
        crate::x86_64::disallow_access_user_memory()
    }
}
