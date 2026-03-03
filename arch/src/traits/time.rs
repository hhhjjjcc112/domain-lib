//! Time and counter operations trait

use super::Arch;

/// Time and counter operations trait
///
/// Provides methods for reading timers/counters and converting between ticks and time units.
pub trait TimeIf {
    /// Initialize time subsystem with frequency in Hz
    fn init_time(freq_hz: u64);
    
    /// Initialize time baseline (record initial counter value)
    fn init_time_baseline();
    
    /// Set epoch offset for wall time (nanoseconds since Unix epoch)
    fn set_epoch_offset_nanos(offset: u64);
    
    /// Get counter/TSC frequency in Hz
    fn frequency() -> u64;
    
    /// Read raw timer/counter value
    fn read_timer() -> usize;
    
    /// Read current ticks since init
    fn current_ticks() -> u64;
    
    /// Read cycle counter
    fn read_cycle() -> usize;
    
    /// Convert ticks to nanoseconds
    fn ticks_to_nanos(ticks: u64) -> u64;
    
    /// Convert nanoseconds to ticks
    fn nanos_to_ticks(nanos: u64) -> u64;
    
    /// Get monotonic time in nanoseconds since boot
    #[inline]
    fn monotonic_time_nanos() -> u64 {
        Self::ticks_to_nanos(Self::current_ticks())
    }
    
    /// Get wall time in nanoseconds since Unix epoch
    fn wall_time_nanos() -> u64;
}

// ============================================================================
// RISC-V implementation
// ============================================================================

#[cfg(target_arch = "riscv64")]
impl TimeIf for Arch {
    #[inline]
    fn init_time(freq_hz: u64) {
        crate::riscv::init_clock_freq(freq_hz)
    }
    
    #[inline]
    fn init_time_baseline() {
        crate::riscv::init_time_baseline()
    }
    
    #[inline]
    fn set_epoch_offset_nanos(offset: u64) {
        crate::riscv::set_epoch_offset_nanos(offset)
    }
    
    #[inline]
    fn frequency() -> u64 {
        crate::riscv::clock_frequency()
    }
    
    #[inline]
    fn read_timer() -> usize {
        crate::riscv::read_timer()
    }
    
    #[inline]
    fn current_ticks() -> u64 {
        crate::riscv::current_ticks()
    }
    
    #[inline]
    fn read_cycle() -> usize {
        crate::riscv::read_cycle()
    }
    
    #[inline]
    fn ticks_to_nanos(ticks: u64) -> u64 {
        crate::riscv::ticks_to_nanos(ticks)
    }
    
    #[inline]
    fn nanos_to_ticks(nanos: u64) -> u64 {
        crate::riscv::nanos_to_ticks(nanos)
    }
    
    #[inline]
    fn wall_time_nanos() -> u64 {
        crate::riscv::wall_time_nanos()
    }
}

// ============================================================================
// x86-64 implementation
// ============================================================================

#[cfg(target_arch = "x86_64")]
impl TimeIf for Arch {
    #[inline]
    fn init_time(freq_hz: u64) {
        crate::x86_64::init_tsc_freq(freq_hz)
    }
    
    #[inline]
    fn init_time_baseline() {
        crate::x86_64::init_tsc_baseline()
    }
    
    #[inline]
    fn set_epoch_offset_nanos(offset: u64) {
        crate::x86_64::set_epoch_offset_nanos(offset)
    }
    
    #[inline]
    fn frequency() -> u64 {
        crate::x86_64::tsc_frequency()
    }
    
    #[inline]
    fn read_timer() -> usize {
        crate::x86_64::read_timer()
    }
    
    #[inline]
    fn current_ticks() -> u64 {
        crate::x86_64::current_ticks()
    }
    
    #[inline]
    fn read_cycle() -> usize {
        crate::x86_64::read_cycle()
    }
    
    #[inline]
    fn ticks_to_nanos(ticks: u64) -> u64 {
        crate::x86_64::ticks_to_nanos(ticks)
    }
    
    #[inline]
    fn nanos_to_ticks(nanos: u64) -> u64 {
        crate::x86_64::nanos_to_ticks(nanos)
    }
    
    #[inline]
    fn wall_time_nanos() -> u64 {
        crate::x86_64::wall_time_nanos()
    }
}
