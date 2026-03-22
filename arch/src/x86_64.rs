//! x86-64 架构支持。
//!
//! 提供 CPU 相关基础操作，命名保持 x86 语义。
#[percpu::def_percpu]
static CPU_ID: usize = 0;


use core::arch::x86_64::_rdtsc;
use core::sync::atomic::{AtomicU64, Ordering};
use raw_cpuid::CpuId;
use x86_64::{
    instructions::{interrupts, hlt},
    registers::{control, rflags},
};



#[inline(always)]
fn cpu_id_from_cpuid() -> usize {
    let cpuid = CpuId::new();
    cpuid
        .get_feature_info()
        .map(|info| info.initial_local_apic_id() as usize)
        .unwrap_or(0)
}

/// 早期读取当前CPU ID（始终走CPUID，避免依赖percpu状态）。
#[inline(always)]
pub fn cpu_id_early() -> usize {
    cpu_id_from_cpuid()
}

#[inline(always)]
pub fn cpu_id() -> usize {
    CPU_ID.read_current()
}

/// 初始化BSP的percpu，并写入当前CPU ID。
///
/// 需在 clear_bss() 之后调用。
pub fn init_percpu_primary(cpu_id: usize) {
    percpu::init_in_place().unwrap();
    percpu::init_percpu_reg(cpu_id);
    CPU_ID.write_current(cpu_id);
}

/// 初始化从核percpu寄存器，并写入当前CPU ID。
pub fn init_percpu_secondary(cpu_id: usize) {
    percpu::init_percpu_reg(cpu_id);
    CPU_ID.write_current(cpu_id);
}

// ==================== 特权级 ====================

/// x86-64 特权级（Ring 0-3）。
///
/// 实际常用 Ring 0 与 Ring 3。
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PrivilegeLevel {
    /// Ring 3 - 用户态
    User = 3,
    /// Ring 0 - 内核态（特权态）
    Kernel = 0,
}

impl PrivilegeLevel {
    /// 由 CPL 原始值构造。
    pub fn from_cpl(cpl: u8) -> Self {
        match cpl & 0x3 {
            0 => PrivilegeLevel::Kernel,
            _ => PrivilegeLevel::User,
        }
    }
    
    /// 转为 CPL 原始值。
    pub fn as_cpl(&self) -> u8 {
        *self as u8
    }
}

// ==================== RFLAGS ====================

/// RFLAGS 位定义。
pub mod rflags_bits {
    /// 进位标志
    pub const CF: usize = 1 << 0;
    /// 奇偶标志
    pub const PF: usize = 1 << 2;
    /// 辅助进位标志
    pub const AF: usize = 1 << 4;
    /// 零标志
    pub const ZF: usize = 1 << 6;
    /// 符号标志
    pub const SF: usize = 1 << 7;
    /// 陷阱标志（单步）
    pub const TF: usize = 1 << 8;
    /// 中断使能标志
    pub const IF: usize = 1 << 9;
    /// 方向标志
    pub const DF: usize = 1 << 10;
    /// 溢出标志
    pub const OF: usize = 1 << 11;
    /// I/O 特权级（12-13 位）
    pub const IOPL_MASK: usize = 0x3 << 12;
    /// 嵌套任务标志
    pub const NT: usize = 1 << 14;
    /// 恢复标志
    pub const RF: usize = 1 << 16;
    /// 虚拟 8086 模式
    pub const VM: usize = 1 << 17;
    /// 对齐检查
    pub const AC: usize = 1 << 18;
    /// 虚拟中断标志
    pub const VIF: usize = 1 << 19;
    /// 虚拟中断挂起
    pub const VIP: usize = 1 << 20;
    /// ID 标志（支持 CPUID）
    pub const ID: usize = 1 << 21;
}

/// x86-64 RFLAGS 包装类型。
#[derive(Debug, Default, Copy, Clone)]
pub struct Rflags(pub usize);

impl Rflags {
    #[inline]
    pub fn read_current() -> Self {
        Self::read()
    }

    /// 读取当前 RFLAGS。
    pub fn read() -> Self {
        Rflags(rflags::read().bits() as usize)
    }
    
    /// 获取原始位值。
    pub fn bits(&self) -> usize {
        self.0
    }

    /// 设置原始位值。
    pub fn set_bits(&mut self, val: usize) {
        self.0 = val;
    }

    // IF 位操作。
    
    /// 查询中断是否开启（IF 位）。
    pub fn interrupt_enabled(&self) -> bool {
        (self.0 & rflags_bits::IF) != 0
    }

    #[inline]
    pub fn interrupts_enabled(&self) -> bool {
        self.interrupt_enabled()
    }
    
    /// 开启中断（置 IF）。
    pub fn enable_interrupts(&mut self) {
        self.0 |= rflags_bits::IF;
    }
    
    /// 关闭中断（清 IF）。
    pub fn disable_interrupts(&mut self) {
        self.0 &= !rflags_bits::IF;
    }
    
    /// 设置 IF 位。
    pub fn set_interrupt_flag(&mut self, value: bool) {
        if value {
            self.enable_interrupts();
        } else {
            self.disable_interrupts();
        }
    }

    // IOPL 操作。
    
    /// 获取 IOPL（0-3）。
    pub fn iopl(&self) -> u8 {
        ((self.0 & rflags_bits::IOPL_MASK) >> 12) as u8
    }
    
    /// 设置 IOPL。
    pub fn set_iopl(&mut self, level: u8) {
        self.0 = (self.0 & !rflags_bits::IOPL_MASK) | (((level & 0x3) as usize) << 12);
    }
    
    // 陷入帧返回特权级缓存位。
    // RFLAGS 不直接编码 CPL，这里用 bit63 临时保存。
    
    const STORED_CPL_BIT: usize = 63;
    
    /// 获取保存的返回特权级。
    pub fn privilege_level(&self) -> PrivilegeLevel {
        if (self.0 >> Self::STORED_CPL_BIT) & 1 == 0 {
            PrivilegeLevel::User
        } else {
            PrivilegeLevel::Kernel
        }
    }
    
    /// 设置保存的返回特权级。
    pub fn set_privilege_level(&mut self, level: PrivilegeLevel) {
        match level {
            PrivilegeLevel::User => self.0 &= !(1 << Self::STORED_CPL_BIT),
            PrivilegeLevel::Kernel => self.0 |= 1 << Self::STORED_CPL_BIT,
        }
    }

    #[inline]
    pub fn get_privilege(&self) -> PrivilegeLevel {
        self.privilege_level()
    }

    #[inline]
    pub fn set_privilege(&mut self, level: PrivilegeLevel) {
        self.set_privilege_level(level);
    }
    
    // 新任务默认状态。
    
    /// 构造用户态任务初始 RFLAGS。
    pub fn new_user() -> Self {
        let mut flags = Rflags(0x202); // IF set, reserved bit 1 always set
        flags.set_privilege_level(PrivilegeLevel::User);
        flags
    }
    
    /// 构造内核态任务初始 RFLAGS。
    pub fn new_kernel() -> Self {
        let mut flags = Rflags(0x202); // IF set
        flags.set_privilege_level(PrivilegeLevel::Kernel);
        flags
    }
}

// 中断控制

/// 检查中断是否开启
pub fn is_interrupt_enable() -> bool {
    rflags::read().contains(rflags::RFlags::INTERRUPT_FLAG)
}

/// 关闭中断
#[inline(always)]
pub fn interrupt_disable() {
    interrupts::disable();
}

/// 开启中断
#[inline(always)]
pub fn interrupt_enable() {
    interrupts::enable();
}

/// 开启外部中断（x86 下为空实现，由 APIC/PIC 负责）
pub fn external_interrupt_enable() {}

/// 开启软件中断（x86 下为空实现）
pub fn software_interrupt_enable() {}

/// 关闭外部中断（x86 下为空实现，由 APIC/PIC 负责）
pub fn external_interrupt_disable() {}

/// 开启时钟中断（x86 下为空实现，由 APIC 定时器负责）
pub fn timer_interrupt_enable() {}

// 时间与 TSC

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

/// Activate paging mode by setting CR3
pub fn activate_paging_mode(page_table_token: usize) {
    unsafe {
        control::Cr3::write(
            x86_64::structures::paging::PhysFrame::containing_address(
                x86_64::PhysAddr::new(page_table_token as u64)
            ),
            control::Cr3Flags::empty()
        );
    }
}

/// Flush all TLB entries (sfence.vma equivalent)
pub fn sfence_vma_all() {
    unsafe {
        let (frame, flags) = control::Cr3::read();
        control::Cr3::write(frame, flags);
    }
}

/// Flush TLB entry for a specific virtual address
pub fn sfence_vma(vaddr: usize) {
    x86_64::instructions::tlb::flush(x86_64::VirtAddr::new(vaddr as u64));
}

/// Allow access to user memory (SMAP override) - no-op without SMAP
pub fn allow_access_user_memory() {}

/// Disallow access to user memory - no-op without SMAP
pub fn disallow_access_user_memory() {}

// TSC 频率校准（参考 arceos）

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

// 控制寄存器

/// 读取 CR0
pub fn read_cr0() -> usize {
    control::Cr0::read().bits() as usize
}

/// 读取 CR2（页故障线性地址）
pub fn read_cr2() -> usize {
    control::Cr2::read()
        .map(|addr| addr.as_u64() as usize)
        .unwrap_or(0)
}

/// 读取 CR3（页表基址）
pub fn read_cr3() -> usize {
    let (frame, _) = control::Cr3::read();
    frame.start_address().as_u64() as usize
}

/// 写入 CR3
pub fn write_cr3(value: usize) {
    unsafe {
        control::Cr3::write(
            x86_64::structures::paging::PhysFrame::containing_address(
                x86_64::PhysAddr::new(value as u64)
            ),
            control::Cr3Flags::empty()
        );
    }
}

/// 读取 CR4
pub fn read_cr4() -> usize {
    control::Cr4::read().bits() as usize
}

// CPU 特性检测

/// 检查 CPU 是否支持 x2APIC
pub fn has_x2apic() -> bool {
    CpuId::new()
        .get_feature_info()
        .map(|f| f.has_x2apic())
        .unwrap_or(false)
}

/// 检查 CPU 是否支持 FSGSBASE 指令
pub fn has_fsgsbase() -> bool {
    CpuId::new()
        .get_extended_feature_info()
        .map(|f| f.has_fsgsbase())
        .unwrap_or(false)
}

/// 检查 CPU 是否支持 SMAP（特权态访问保护）
pub fn has_smap() -> bool {
    CpuId::new()
        .get_extended_feature_info()
        .map(|f| f.has_smap())
        .unwrap_or(false)
}

/// 检查 CPU 是否支持 SMEP（特权态执行保护）
pub fn has_smep() -> bool {
    CpuId::new()
        .get_extended_feature_info()
        .map(|f| f.has_smep())
        .unwrap_or(false)
}

/// 获取 CPU 厂商字符串
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

// 停机与自旋提示

/// 挂起 CPU，直到下一次中断
#[inline(always)]
pub fn halt() {
    hlt();
}

/// 自旋提示指令
#[inline(always)]
pub fn pause() {
    core::hint::spin_loop();
}

/// 等待中断（halt 封装）
#[inline(always)]
pub fn wfi() {
    halt();
}

// FPU / SSE 初始化

/// 初始化 FPU/SSE 支持。
///
/// 需在每个 CPU 核心启动时各调用一次：
/// - 清 CR0.EM（禁止 FPU 模拟）、设 CR0.MP（监控协处理器）、清 CR0.TS（不触发任务切换陷阱）
/// - 设 CR4.OSFXSR（允许 FXSAVE/FXRSTOR 及 SSE 指令）
/// - 设 CR4.OSXMMEXCPT（允许未屏蔽 SSE 异常）
/// - 执行 FNINIT 重置 x87 FPU 到默认状态
pub fn init_fpu() {
    unsafe {
        // CR0: 清 EM/TS，置 MP。
        let mut cr0 = control::Cr0::read();
        cr0.remove(control::Cr0Flags::EMULATE_COPROCESSOR);
        cr0.insert(control::Cr0Flags::MONITOR_COPROCESSOR);
        cr0.remove(control::Cr0Flags::TASK_SWITCHED);
        control::Cr0::write(cr0);

        // CR4: 置 OSFXSR/OSXMMEXCPT。
        let mut cr4 = control::Cr4::read();
        cr4.insert(control::Cr4Flags::OSFXSR);
        cr4.insert(control::Cr4Flags::OSXMMEXCPT_ENABLE);
        control::Cr4::write(cr4);

        // 保留 x87 初始化。
        core::arch::asm!("fninit", options(nomem, nostack, preserves_flags));
    }
}

