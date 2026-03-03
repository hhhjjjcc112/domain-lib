//! Processor status register operations trait

/// Processor status register operations trait
///
/// Provides unified interface for manipulating processor status registers.
/// - RISC-V: sstatus register (ExtSstatus)
/// - x86-64: RFLAGS register
pub trait ProcessorStatusIf: Sized + Clone + Copy {
    /// Associated privilege mode type
    type PrivilegeMode: Clone + Copy;
    
    /// Read current processor status
    fn read_current() -> Self;
    
    /// Check if interrupts are enabled
    fn interrupts_enabled(&self) -> bool;
    
    /// Enable interrupts in status
    fn enable_interrupts(&mut self);
    
    /// Disable interrupts in status
    fn disable_interrupts(&mut self);
    
    /// Get privilege level for trap return
    fn get_privilege(&self) -> Self::PrivilegeMode;
    
    /// Set privilege level for trap return
    fn set_privilege(&mut self, mode: Self::PrivilegeMode);
}
