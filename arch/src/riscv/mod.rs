//! RISC-V architecture support
//!
//! Provides CPU-level operations for RISC-V architecture.

mod regs;

use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

pub use regs::*;
use riscv::register::satp;

// ============================================================================
// CPU identification
// ============================================================================

/// Get current hart ID
#[inline(always)]
pub fn hart_id() -> usize {
    let mut id: usize;
    unsafe {
        asm!(
        "mv {},tp", out(reg)id,
        );
    }
    // lower 32 bits of tp register is the hart id
    id as u32 as usize
}

/// Get current CPU ID (alias for hart_id, unified interface)
#[inline(always)]
pub fn cpu_id() -> usize {
    hart_id()
}

/// Get current CPU ID (alias for hart_id)
#[inline(always)]
pub fn current_cpu_id() -> usize {
    hart_id()
}

// ============================================================================
// Interrupt control
// ============================================================================

/// Check if global interrupts are enabled
pub fn is_interrupt_enable() -> bool {
    riscv::register::sstatus::read().sie()
}

/// Disable global interrupts
pub fn interrupt_disable() {
    unsafe {
        riscv::register::sstatus::clear_sie();
    }
}

/// Enable global interrupts
pub fn interrupt_enable() {
    unsafe {
        riscv::register::sstatus::set_sie();
    }
}

/// Enable external interrupts
pub fn external_interrupt_enable() {
    unsafe {
        riscv::register::sie::set_sext();
    }
}

/// Enable software interrupts
pub fn software_interrupt_enable() {
    unsafe {
        riscv::register::sie::set_ssoft();
    }
}

/// Disable external interrupts
pub fn external_interrupt_disable() {
    unsafe {
        riscv::register::sie::clear_sext();
    }
}

/// Enable timer interrupts
pub fn timer_interrupt_enable() {
    unsafe {
        riscv::register::sie::set_stimer();
    }
}

// ============================================================================
// Time
// ============================================================================

/// Clock frequency in Hz (set by platform)
static CLOCK_FREQ_HZ: AtomicU64 = AtomicU64::new(12_500_000); // Default 12.5MHz

/// Initial timer value at boot
static TIME_INIT: AtomicU64 = AtomicU64::new(0);

/// RTC epoch offset in nanoseconds
static EPOCH_OFFSET_NANOS: AtomicU64 = AtomicU64::new(0);

/// Initialize clock frequency
pub fn init_clock_freq(freq_hz: u64) {
    CLOCK_FREQ_HZ.store(freq_hz, Ordering::SeqCst);
}

/// Initialize time baseline
pub fn init_time_baseline() {
    TIME_INIT.store(riscv::register::time::read() as u64, Ordering::SeqCst);
}

/// Set epoch offset for wall time
pub fn set_epoch_offset_nanos(offset: u64) {
    EPOCH_OFFSET_NANOS.store(offset, Ordering::SeqCst);
}

/// Get clock frequency in Hz
pub fn clock_frequency() -> u64 {
    CLOCK_FREQ_HZ.load(Ordering::Relaxed)
}

/// Read raw timer value
#[inline]
pub fn read_timer() -> usize {
    riscv::register::time::read()
}

/// Read current ticks since init
#[inline]
pub fn current_ticks() -> u64 {
    let current = riscv::register::time::read() as u64;
    let init = TIME_INIT.load(Ordering::Relaxed);
    current.saturating_sub(init)
}

/// Read cycle counter
pub fn read_cycle() -> usize {
    riscv::register::cycle::read()
}

/// Convert ticks to nanoseconds
#[inline]
pub fn ticks_to_nanos(ticks: u64) -> u64 {
    let freq = CLOCK_FREQ_HZ.load(Ordering::Relaxed);
    if freq == 0 {
        return 0;
    }
    (ticks as u128 * 1_000_000_000 / freq as u128) as u64
}

/// Convert nanoseconds to ticks
#[inline]
pub fn nanos_to_ticks(nanos: u64) -> u64 {
    let freq = CLOCK_FREQ_HZ.load(Ordering::Relaxed);
    (nanos as u128 * freq as u128 / 1_000_000_000) as u64
}

/// Get epoch offset in nanoseconds
#[inline]
pub fn epochoffset_nanos() -> u64 {
    EPOCH_OFFSET_NANOS.load(Ordering::Relaxed)
}

/// Get monotonic time in nanoseconds since boot
#[inline]
pub fn monotonic_time_nanos() -> u64 {
    ticks_to_nanos(current_ticks())
}

/// Get wall time in nanoseconds since Unix epoch
#[inline]
pub fn wall_time_nanos() -> u64 {
    monotonic_time_nanos() + epochoffset_nanos()
}

// ============================================================================
// Paging
// ============================================================================

/// Activate page table (Sv39 mode)
pub fn activate_paging_mode(root_ppn: usize) {
    unsafe {
        sfence_vma_all();
        satp::set(satp::Mode::Sv39, 0, root_ppn);
        sfence_vma_all();
    }
}

/// Flush all TLB entries
pub fn sfence_vma_all() {
    riscv::asm::sfence_vma_all()
}

/// Flush TLB entry for a specific virtual address
pub fn sfence_vma(vaddr: usize) {
    unsafe {
        asm!(
            "sfence.vma {}, zero",
            in(reg) vaddr,
            options(nostack, preserves_flags)
        );
    }
}

/// Allow access to user memory (set SUM bit)
pub fn allow_access_user_memory() {
    unsafe {
        riscv::register::sstatus::set_sum();
    }
}

/// Disallow access to user memory (clear SUM bit)
pub fn disallow_access_user_memory() {
    unsafe {
        riscv::register::sstatus::clear_sum();
    }
}

// ============================================================================
// Halt and Wait
// ============================================================================

/// Wait for interrupt
#[inline(always)]
pub fn wfi() {
    riscv::asm::wfi();
}

// ============================================================================
// ProcessorStatusIf Trait Implementation
// ============================================================================

use crate::traits::ProcessorStatusIf;

impl ProcessorStatusIf for ExtSstatus {
    type PrivilegeMode = SPP;
    
    #[inline]
    fn read_current() -> Self {
        Self::read()
    }
    
    #[inline]
    fn interrupts_enabled(&self) -> bool {
        self.sie()
    }
    
    #[inline]
    fn enable_interrupts(&mut self) {
        self.set_sie(true);
    }
    
    #[inline]
    fn disable_interrupts(&mut self) {
        self.set_sie(false);
    }
    
    #[inline]
    fn get_privilege(&self) -> Self::PrivilegeMode {
        self.spp()
    }
    
    #[inline]
    fn set_privilege(&mut self, mode: Self::PrivilegeMode) {
        self.set_spp(mode);
    }
}
