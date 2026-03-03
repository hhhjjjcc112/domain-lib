//! Halt and wait operations trait

use super::Arch;

/// Halt and wait operations trait
///
/// Provides methods to halt/wait the processor.
pub trait HaltIf {
    /// Wait for interrupt (halt CPU until next interrupt)
    fn wfi();
    
    /// Halt the CPU
    fn halt();
}

// ============================================================================
// RISC-V implementation
// ============================================================================

#[cfg(target_arch = "riscv64")]
impl HaltIf for Arch {
    #[inline]
    fn wfi() {
        crate::riscv::wfi()
    }
    
    #[inline]
    fn halt() {
        crate::riscv::wfi()
    }
}

// ============================================================================
// x86-64 implementation
// ============================================================================

#[cfg(target_arch = "x86_64")]
impl HaltIf for Arch {
    #[inline]
    fn wfi() {
        crate::x86_64::wfi()
    }
    
    #[inline]
    fn halt() {
        crate::x86_64::halt()
    }
}
