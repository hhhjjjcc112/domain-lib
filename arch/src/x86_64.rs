//! x86-64 architecture support
//!
//! Provides CPU-level operations for x86-64 architecture using native x86-64 naming.
//! References arceos axplat-x86-pc implementation.

use core::arch::{asm, x86_64::_rdtsc};
use core::sync::atomic::{AtomicU64, Ordering};
use raw_cpuid::CpuId;

// ============================================================================
// Privilege Level (x86-64 CPL - Current Privilege Level)
// ============================================================================

/// x86-64 Privilege Level (Ring 0-3)
/// 
/// x86-64 uses 4 privilege levels (rings), but modern OSes typically only use:
/// - Ring 0 (Kernel/Supervisor mode)
/// - Ring 3 (User mode)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PrivilegeLevel {
    /// Ring 3 - User mode
    User = 3,
    /// Ring 0 - Kernel mode (Supervisor)
    Kernel = 0,
}

impl PrivilegeLevel {
    /// Create from raw CPL value
    pub fn from_cpl(cpl: u8) -> Self {
        match cpl & 0x3 {
            0 => PrivilegeLevel::Kernel,
            _ => PrivilegeLevel::User,
        }
    }
    
    /// Convert to raw CPL value
    pub fn as_cpl(&self) -> u8 {
        *self as u8
    }
}

// ============================================================================
// RFLAGS Register
// ============================================================================

/// RFLAGS register bits
pub mod rflags_bits {
    /// Carry Flag
    pub const CF: usize = 1 << 0;
    /// Parity Flag
    pub const PF: usize = 1 << 2;
    /// Auxiliary Carry Flag
    pub const AF: usize = 1 << 4;
    /// Zero Flag
    pub const ZF: usize = 1 << 6;
    /// Sign Flag
    pub const SF: usize = 1 << 7;
    /// Trap Flag (single step)
    pub const TF: usize = 1 << 8;
    /// Interrupt Enable Flag
    pub const IF: usize = 1 << 9;
    /// Direction Flag
    pub const DF: usize = 1 << 10;
    /// Overflow Flag
    pub const OF: usize = 1 << 11;
    /// I/O Privilege Level (bits 12-13)
    pub const IOPL_MASK: usize = 0x3 << 12;
    /// Nested Task
    pub const NT: usize = 1 << 14;
    /// Resume Flag
    pub const RF: usize = 1 << 16;
    /// Virtual 8086 Mode
    pub const VM: usize = 1 << 17;
    /// Alignment Check
    pub const AC: usize = 1 << 18;
    /// Virtual Interrupt Flag
    pub const VIF: usize = 1 << 19;
    /// Virtual Interrupt Pending
    pub const VIP: usize = 1 << 20;
    /// ID Flag (CPUID available)
    pub const ID: usize = 1 << 21;
}

/// RFLAGS register wrapper for x86-64
/// 
/// The RFLAGS register contains status flags, control flags, and system flags.
#[derive(Debug, Default, Copy, Clone)]
pub struct Rflags(pub usize);

impl Rflags {
    /// Read current RFLAGS register value
    pub fn read() -> Self {
        let mut rflags: usize;
        unsafe {
            asm!(
                "pushfq",
                "pop {}",
                out(reg) rflags,
                options(nomem, preserves_flags)
            );
        }
        Rflags(rflags)
    }
    
    /// Get raw value
    pub fn bits(&self) -> usize {
        self.0
    }

    /// Set raw value
    pub fn set_bits(&mut self, val: usize) {
        self.0 = val;
    }

    // ========== Interrupt Flag (IF) operations ==========
    
    /// Check if interrupts are enabled (IF flag)
    pub fn interrupt_enabled(&self) -> bool {
        (self.0 & rflags_bits::IF) != 0
    }
    
    /// Enable interrupts (set IF flag)
    pub fn enable_interrupts(&mut self) {
        self.0 |= rflags_bits::IF;
    }
    
    /// Disable interrupts (clear IF flag)
    pub fn disable_interrupts(&mut self) {
        self.0 &= !rflags_bits::IF;
    }
    
    /// Set interrupt flag to specific value
    pub fn set_interrupt_flag(&mut self, value: bool) {
        if value {
            self.enable_interrupts();
        } else {
            self.disable_interrupts();
        }
    }

    // ========== I/O Privilege Level (IOPL) operations ==========
    
    /// Get I/O Privilege Level (0-3)
    pub fn iopl(&self) -> u8 {
        ((self.0 & rflags_bits::IOPL_MASK) >> 12) as u8
    }
    
    /// Set I/O Privilege Level
    pub fn set_iopl(&mut self, level: u8) {
        self.0 = (self.0 & !rflags_bits::IOPL_MASK) | (((level & 0x3) as usize) << 12);
    }
    
    // ========== Stored privilege level for trap frames ==========
    // We use a reserved bit position (bit 63) to store the return privilege level
    // since RFLAGS doesn't directly encode CPL
    
    const STORED_CPL_BIT: usize = 63;
    
    /// Get stored privilege level for trap return
    pub fn privilege_level(&self) -> PrivilegeLevel {
        if (self.0 >> Self::STORED_CPL_BIT) & 1 == 0 {
            PrivilegeLevel::User
        } else {
            PrivilegeLevel::Kernel
        }
    }
    
    /// Set stored privilege level for trap return
    pub fn set_privilege_level(&mut self, level: PrivilegeLevel) {
        match level {
            PrivilegeLevel::User => self.0 &= !(1 << Self::STORED_CPL_BIT),
            PrivilegeLevel::Kernel => self.0 |= 1 << Self::STORED_CPL_BIT,
        }
    }
    
    // ========== Default settings for new tasks ==========
    
    /// Create RFLAGS for a new user task
    pub fn new_user() -> Self {
        let mut flags = Rflags(0x202); // IF set, reserved bit 1 always set
        flags.set_privilege_level(PrivilegeLevel::User);
        flags
    }
    
    /// Create RFLAGS for a new kernel task
    pub fn new_kernel() -> Self {
        let mut flags = Rflags(0x202); // IF set
        flags.set_privilege_level(PrivilegeLevel::Kernel);
        flags
    }
}

// ============================================================================
// CPU identification
// ============================================================================

/// Get current CPU ID via CPUID instruction (returns Local APIC ID)
#[inline(always)]
pub fn cpu_id() -> usize {
    // Use raw-cpuid crate to avoid direct cpuid inline asm issues with rbx
    use raw_cpuid::CpuId;
    let cpuid = CpuId::new();
    cpuid
        .get_feature_info()
        .map(|info| info.initial_local_apic_id() as usize)
        .unwrap_or(0)
}

/// Get Local APIC ID (alias for cpu_id)
#[inline(always)]
pub fn apic_id() -> usize {
    cpu_id()
}

/// Get current CPU ID
#[inline(always)]
pub fn current_cpu_id() -> usize {
    cpu_id()
}

/// Get current CPU ID (RISC-V compatibility alias)
/// 
/// This function exists for compatibility with code that uses RISC-V terminology.
/// On x86-64, this returns the Local APIC ID.
#[inline(always)]
pub fn hart_id() -> usize {
    cpu_id()
}

// ============================================================================
// Interrupt control
// ============================================================================

/// Check if interrupts are enabled
pub fn is_interrupt_enable() -> bool {
    let mut rflags: usize;
    unsafe {
        asm!(
            "pushfq",
            "pop {}",
            out(reg) rflags,
            options(nomem, preserves_flags)
        );
    }
    (rflags & (1 << 9)) != 0 // IF flag
}

/// Disable interrupts
pub fn interrupt_disable() {
    unsafe {
        asm!("cli", options(nomem, nostack, preserves_flags));
    }
}

/// Enable interrupts
pub fn interrupt_enable() {
    unsafe {
        asm!("sti", options(nomem, nostack, preserves_flags));
    }
}

/// Enable external interrupts (no-op on x86, handled by APIC/PIC)
pub fn external_interrupt_enable() {}

/// Enable software interrupts (no-op on x86)
pub fn software_interrupt_enable() {}

/// Disable external interrupts (no-op on x86, handled by APIC/PIC)
pub fn external_interrupt_disable() {}

/// Enable timer interrupts (no-op on x86, handled by APIC timer setup)
pub fn timer_interrupt_enable() {}

// ============================================================================
// Time and TSC
// ============================================================================

/// TSC frequency in Hz (calibrated at runtime)
static TSC_FREQ_HZ: AtomicU64 = AtomicU64::new(4_000_000_000);

/// Initial TSC value at boot
static TSC_INIT: AtomicU64 = AtomicU64::new(0);

/// RTC epoch offset in nanoseconds
static EPOCH_OFFSET_NANOS: AtomicU64 = AtomicU64::new(0);

/// Initialize TSC frequency (should be called once at boot)
pub fn init_tsc_freq(freq_hz: u64) {
    TSC_FREQ_HZ.store(freq_hz, Ordering::SeqCst);
}

/// Initialize TSC baseline
pub fn init_tsc_baseline() {
    TSC_INIT.store(unsafe { _rdtsc() }, Ordering::SeqCst);
}

/// Set epoch offset for wall time
pub fn set_epoch_offset_nanos(offset: u64) {
    EPOCH_OFFSET_NANOS.store(offset, Ordering::SeqCst);
}

/// Get TSC frequency in Hz
pub fn tsc_frequency() -> u64 {
    TSC_FREQ_HZ.load(Ordering::Relaxed)
}

/// Read current TSC value (raw)
#[inline]
pub fn read_timer() -> usize {
    unsafe { _rdtsc() as usize }
}

/// Read current ticks since init
#[inline]
pub fn current_ticks() -> u64 {
    let current = unsafe { _rdtsc() };
    let init = TSC_INIT.load(Ordering::Relaxed);
    current.saturating_sub(init)
}

/// Read cycle counter (alias for read_timer)
#[inline]
pub fn read_cycle() -> usize {
    read_timer()
}

/// Convert ticks to nanoseconds
#[inline]
pub fn ticks_to_nanos(ticks: u64) -> u64 {
    let freq = TSC_FREQ_HZ.load(Ordering::Relaxed);
    if freq == 0 {
        return 0;
    }
    (ticks as u128 * 1_000_000_000 / freq as u128) as u64
}

/// Convert nanoseconds to ticks
#[inline]
pub fn nanos_to_ticks(nanos: u64) -> u64 {
    let freq = TSC_FREQ_HZ.load(Ordering::Relaxed);
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

/// Activate paging mode by setting CR3
pub fn activate_paging_mode(root_ppn: usize) {
    let root_paddr = root_ppn << 12;
    unsafe {
        asm!("mov cr3, {}", in(reg) root_paddr, options(nomem, nostack, preserves_flags));
    }
}

/// Flush all TLB entries (sfence.vma equivalent)
pub fn sfence_vma_all() {
    unsafe {
        let cr3: usize;
        asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
        asm!("mov cr3, {}", in(reg) cr3, options(nomem, nostack, preserves_flags));
    }
}

/// Flush TLB entry for a specific virtual address
pub fn sfence_vma(vaddr: usize) {
    unsafe {
        asm!("invlpg [{}]", in(reg) vaddr, options(nostack, preserves_flags));
    }
}

/// Allow access to user memory (SMAP override) - no-op without SMAP
pub fn allow_access_user_memory() {}

/// Disallow access to user memory - no-op without SMAP
pub fn disallow_access_user_memory() {}

// ============================================================================
// TSC Frequency Calibration (arceos style)
// ============================================================================

/// Calibrate TSC frequency using CPUID if available
/// 
/// Returns the TSC frequency in MHz, or None if not available via CPUID.
pub fn calibrate_tsc_freq_cpuid() -> Option<u64> {
    let cpuid = CpuId::new();
    
    // Try to get processor frequency directly
    if let Some(freq_info) = cpuid.get_processor_frequency_info() {
        let base_freq = freq_info.processor_base_frequency();
        if base_freq > 0 {
            return Some(base_freq as u64);
        }
    }
    
    // Try TSC frequency from extended leaf (leaf 0x15)
    if let Some(tsc_info) = cpuid.get_tsc_info() {
        let crystal_freq = tsc_info.nominal_frequency();
        if crystal_freq > 0 {
            let numerator = tsc_info.tsc_frequency().unwrap_or(0);
            let denominator = tsc_info.denominator();
            if numerator > 0 && denominator > 0 {
                let freq_hz = (numerator as u64) * (crystal_freq as u64) / (denominator as u64);
                return Some(freq_hz / 1_000_000);
            }
        }
    }
    
    None
}

/// Initialize TSC with auto-calibration
pub fn init_tsc() {
    // Try CPUID-based calibration first
    if let Some(freq_mhz) = calibrate_tsc_freq_cpuid() {
        init_tsc_freq(freq_mhz * 1_000_000);
    }
    // Otherwise use default (4GHz), can be re-calibrated later
    
    init_tsc_baseline();
}

// ============================================================================
// Control Registers
// ============================================================================

/// Read CR0 register
pub fn read_cr0() -> usize {
    let value: usize;
    unsafe {
        asm!("mov {}, cr0", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Read CR2 register (page fault linear address)
pub fn read_cr2() -> usize {
    let value: usize;
    unsafe {
        asm!("mov {}, cr2", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Read CR3 register (page table base)
pub fn read_cr3() -> usize {
    let value: usize;
    unsafe {
        asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Write CR3 register
pub fn write_cr3(value: usize) {
    unsafe {
        asm!("mov cr3, {}", in(reg) value, options(nomem, nostack, preserves_flags));
    }
}

/// Read CR4 register
pub fn read_cr4() -> usize {
    let value: usize;
    unsafe {
        asm!("mov {}, cr4", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

// ============================================================================
// MSR (Model Specific Registers)
// ============================================================================

/// Read MSR register
pub fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }
    ((high as u64) << 32) | (low as u64)
}

/// Write MSR register
pub fn write_msr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") low,
            in("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// EFER MSR address
pub const MSR_EFER: u32 = 0xC0000080;
/// FS base MSR address
pub const MSR_FS_BASE: u32 = 0xC0000100;
/// GS base MSR address
pub const MSR_GS_BASE: u32 = 0xC0000101;
/// Kernel GS base MSR address (for swapgs)
pub const MSR_KERNEL_GS_BASE: u32 = 0xC0000102;

// ============================================================================
// CPU Features Detection
// ============================================================================

/// Check if CPU supports x2APIC
pub fn has_x2apic() -> bool {
    CpuId::new()
        .get_feature_info()
        .map(|f| f.has_x2apic())
        .unwrap_or(false)
}

/// Check if CPU supports FSGSBASE instructions
pub fn has_fsgsbase() -> bool {
    CpuId::new()
        .get_extended_feature_info()
        .map(|f| f.has_fsgsbase())
        .unwrap_or(false)
}

/// Check if CPU supports SMAP (Supervisor Mode Access Prevention)
pub fn has_smap() -> bool {
    CpuId::new()
        .get_extended_feature_info()
        .map(|f| f.has_smap())
        .unwrap_or(false)
}

/// Check if CPU supports SMEP (Supervisor Mode Execution Prevention)
pub fn has_smep() -> bool {
    CpuId::new()
        .get_extended_feature_info()
        .map(|f| f.has_smep())
        .unwrap_or(false)
}

/// Get CPU vendor string
pub fn cpu_vendor() -> Option<&'static str> {
    CpuId::new().get_vendor_info().map(|v| {
        // Return a static reference by matching known vendors
        match v.as_str() {
            "GenuineIntel" => "GenuineIntel",
            "AuthenticAMD" => "AuthenticAMD",
            _ => "Unknown",
        }
    })
}

// ============================================================================
// Halt and Pause
// ============================================================================

/// Halt the CPU until the next interrupt
#[inline(always)]
pub fn halt() {
    unsafe {
        asm!("hlt", options(nomem, nostack));
    }
}

/// Pause instruction (spin-loop hint)
#[inline(always)]
pub fn pause() {
    unsafe {
        asm!("pause", options(nomem, nostack));
    }
}

/// Wait for interrupt (halt wrapper)
#[inline(always)]
pub fn wfi() {
    halt();
}

// ============================================================================
// ProcessorStatusIf Trait Implementation
// ============================================================================

use crate::traits::ProcessorStatusIf;

impl ProcessorStatusIf for Rflags {
    type PrivilegeMode = PrivilegeLevel;
    
    #[inline]
    fn read_current() -> Self {
        Self::read()
    }
    
    #[inline]
    fn interrupts_enabled(&self) -> bool {
        self.interrupt_enabled()
    }
    
    #[inline]
    fn enable_interrupts(&mut self) {
        Rflags::enable_interrupts(self)
    }
    
    #[inline]
    fn disable_interrupts(&mut self) {
        Rflags::disable_interrupts(self)
    }
    
    #[inline]
    fn get_privilege(&self) -> Self::PrivilegeMode {
        self.privilege_level()
    }
    
    #[inline]
    fn set_privilege(&mut self, mode: Self::PrivilegeMode) {
        self.set_privilege_level(mode)
    }
}
