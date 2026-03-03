//! Time-related operations
//!
//! Provides unified time interfaces across architectures.

use core::time::Duration;

use pconst::time::{TimeSpec, TimeVal};

/// Number of nanoseconds per second
pub const NANOS_PER_SEC: u64 = 1_000_000_000;
/// Number of microseconds per second
pub const MICROS_PER_SEC: u64 = 1_000_000;
/// Number of milliseconds per second
pub const MILLIS_PER_SEC: u64 = 1_000;

// ============================================================================
// Basic time reading
// ============================================================================

/// Read raw timer value (architecture-specific)
#[inline]
pub fn read_timer() -> usize {
    arch::read_timer()
}

/// Read current ticks since system init
#[inline]
pub fn current_ticks() -> u64 {
    arch::current_ticks()
}

/// Convert ticks to nanoseconds
#[inline]
pub fn ticks_to_nanos(ticks: u64) -> u64 {
    arch::ticks_to_nanos(ticks)
}

/// Convert nanoseconds to ticks
#[inline]
pub fn nanos_to_ticks(nanos: u64) -> u64 {
    arch::nanos_to_ticks(nanos)
}

// ============================================================================
// Monotonic time (since boot)
// ============================================================================

/// Get monotonic time in nanoseconds since boot
#[inline]
pub fn monotonic_time_nanos() -> u64 {
    arch::monotonic_time_nanos()
}

/// Get monotonic time as Duration since boot
#[inline]
pub fn monotonic_time() -> Duration {
    Duration::from_nanos(monotonic_time_nanos())
}

/// Read time in milliseconds since boot
#[inline]
pub fn read_time_ms() -> u64 {
    monotonic_time_nanos() / 1_000_000
}

/// Read time in microseconds since boot
#[inline]
pub fn read_time_us() -> u64 {
    monotonic_time_nanos() / 1_000
}

/// Read time in nanoseconds since boot
#[inline]
pub fn read_time_ns() -> u64 {
    monotonic_time_nanos()
}

// ============================================================================
// Wall time (since Unix epoch)
// ============================================================================

/// Get wall time in nanoseconds since Unix epoch
#[inline]
pub fn wall_time_nanos() -> u64 {
    arch::wall_time_nanos()
}

/// Get wall time as Duration since Unix epoch
#[inline]
pub fn wall_time() -> Duration {
    Duration::from_nanos(wall_time_nanos())
}

/// Get epoch offset in nanoseconds
#[inline]
pub fn epochoffset_nanos() -> u64 {
    arch::epochoffset_nanos()
}

// ============================================================================
// Busy waiting
// ============================================================================

/// Busy wait for the given duration
pub fn busy_wait(dur: Duration) {
    let deadline = monotonic_time() + dur;
    busy_wait_until(deadline);
}

/// Busy wait until reaching the given deadline (monotonic time)
pub fn busy_wait_until(deadline: Duration) {
    while monotonic_time() < deadline {
        core::hint::spin_loop();
    }
}

// ============================================================================
// Compatibility traits
// ============================================================================

/// Trait to convert time types to clock ticks
pub trait ToClock {
    fn to_clock(&self) -> usize;
}

/// Trait to get current time
pub trait TimeNow {
    fn now() -> Self;
}

impl ToClock for TimeSpec {
    fn to_clock(&self) -> usize {
        let nanos = self.tv_sec as u64 * NANOS_PER_SEC + self.tv_nsec as u64;
        nanos_to_ticks(nanos) as usize
    }
}

impl TimeNow for TimeSpec {
    fn now() -> Self {
        let nanos = monotonic_time_nanos();
        Self {
            tv_sec: (nanos / NANOS_PER_SEC) as usize,
            tv_nsec: (nanos % NANOS_PER_SEC) as usize,
        }
    }
}

impl TimeNow for TimeVal {
    fn now() -> Self {
        let nanos = monotonic_time_nanos();
        Self {
            tv_sec: (nanos / NANOS_PER_SEC) as usize,
            tv_usec: ((nanos % NANOS_PER_SEC) / 1_000) as usize,
        }
    }
}
