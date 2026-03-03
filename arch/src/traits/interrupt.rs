//! Interrupt control trait

use super::Arch;

/// Interrupt control trait
///
/// Provides methods to control interrupt enable/disable at the processor level.
pub trait InterruptIf {
    /// Check if global interrupts are enabled
    fn is_interrupt_enabled() -> bool;
    
    /// Disable global interrupts
    fn interrupt_disable();
    
    /// Enable global interrupts
    fn interrupt_enable();
    
    /// Enable external interrupts (PLIC/APIC)
    fn external_interrupt_enable();
    
    /// Disable external interrupts
    fn external_interrupt_disable();
    
    /// Enable timer interrupts
    fn timer_interrupt_enable();
    
    /// Enable software interrupts (IPI)
    fn software_interrupt_enable();
}

// ============================================================================
// RISC-V implementation
// ============================================================================

#[cfg(target_arch = "riscv64")]
impl InterruptIf for Arch {
    #[inline]
    fn is_interrupt_enabled() -> bool {
        crate::riscv::is_interrupt_enable()
    }
    
    #[inline]
    fn interrupt_disable() {
        crate::riscv::interrupt_disable()
    }
    
    #[inline]
    fn interrupt_enable() {
        crate::riscv::interrupt_enable()
    }
    
    #[inline]
    fn external_interrupt_enable() {
        crate::riscv::external_interrupt_enable()
    }
    
    #[inline]
    fn external_interrupt_disable() {
        crate::riscv::external_interrupt_disable()
    }
    
    #[inline]
    fn timer_interrupt_enable() {
        crate::riscv::timer_interrupt_enable()
    }
    
    #[inline]
    fn software_interrupt_enable() {
        crate::riscv::software_interrupt_enable()
    }
}

// ============================================================================
// x86-64 implementation
// ============================================================================

#[cfg(target_arch = "x86_64")]
impl InterruptIf for Arch {
    #[inline]
    fn is_interrupt_enabled() -> bool {
        crate::x86_64::is_interrupt_enable()
    }
    
    #[inline]
    fn interrupt_disable() {
        crate::x86_64::interrupt_disable()
    }
    
    #[inline]
    fn interrupt_enable() {
        crate::x86_64::interrupt_enable()
    }
    
    #[inline]
    fn external_interrupt_enable() {
        crate::x86_64::external_interrupt_enable()
    }
    
    #[inline]
    fn external_interrupt_disable() {
        crate::x86_64::external_interrupt_disable()
    }
    
    #[inline]
    fn timer_interrupt_enable() {
        crate::x86_64::timer_interrupt_enable()
    }
    
    #[inline]
    fn software_interrupt_enable() {
        crate::x86_64::software_interrupt_enable()
    }
}
