//! CPU identification trait

use super::Arch;

/// CPU identification trait
///
/// Provides methods to query the current CPU/hart identifier.
pub trait CpuIf {
    /// Get current CPU/hart ID
    fn cpu_id() -> usize;
    
    /// Get current CPU ID (alias)
    #[inline]
    fn current_cpu_id() -> usize {
        Self::cpu_id()
    }
}

// ============================================================================
// RISC-V implementation
// ============================================================================

#[cfg(target_arch = "riscv64")]
impl CpuIf for Arch {
    #[inline]
    fn cpu_id() -> usize {
        crate::riscv::hart_id()
    }
}

// ============================================================================
// x86-64 implementation
// ============================================================================

#[cfg(target_arch = "x86_64")]
impl CpuIf for Arch {
    #[inline]
    fn cpu_id() -> usize {
        crate::x86_64::cpu_id()
    }
}
