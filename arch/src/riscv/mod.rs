//! RISC-V 架构支持。
//!
//! 提供 CPU 相关基础操作。

mod regs;

use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

pub use regs::*;
use riscv::register::satp;

#[percpu::def_percpu]
static CPU_ID: usize = 0;

// ==================== CPU 标识 ====================

/// 获取当前 CPU ID。
#[inline(always)]
pub fn cpu_id() -> usize {
    CPU_ID.read_current()
}

/// 初始化 BSP 的 percpu 基址。
pub fn init_percpu_primary(cpu_id: usize) {
    percpu::init_in_place().unwrap();
    percpu::init_percpu_reg(cpu_id);
    CPU_ID.write_current(cpu_id);
}

/// 初始化从核的 percpu 基址。
pub fn init_percpu_secondary(cpu_id: usize) {
    percpu::init_percpu_reg(cpu_id);
    CPU_ID.write_current(cpu_id);
}

// ==================== 中断控制 ====================

/// 查询全局中断是否开启。
pub fn is_interrupt_enable() -> bool {
    riscv::register::sstatus::read().sie()
}

/// 关闭全局中断。
pub fn interrupt_disable() {
    unsafe {
        riscv::register::sstatus::clear_sie();
    }
}

/// 开启全局中断。
pub fn interrupt_enable() {
    unsafe {
        riscv::register::sstatus::set_sie();
    }
}

/// 开启外部中断。
pub fn external_interrupt_enable() {
    unsafe {
        riscv::register::sie::set_sext();
    }
}

/// 开启软件中断。
pub fn software_interrupt_enable() {
    unsafe {
        riscv::register::sie::set_ssoft();
    }
}

/// 关闭外部中断。
pub fn external_interrupt_disable() {
    unsafe {
        riscv::register::sie::clear_sext();
    }
}

/// 开启定时器中断。
pub fn timer_interrupt_enable() {
    unsafe {
        riscv::register::sie::set_stimer();
    }
}

// ==================== 时间相关 ====================

/// 时钟频率（Hz，由平台设置）。
static CLOCK_FREQ_HZ: AtomicU64 = AtomicU64::new(12_500_000); // Default 12.5MHz

/// 启动时计时器基线值。
static TIME_INIT: AtomicU64 = AtomicU64::new(0);

/// 纪元偏移（纳秒）。
static EPOCH_OFFSET_NANOS: AtomicU64 = AtomicU64::new(0);

/// 初始化时钟频率。
pub fn init_clock_freq(freq_hz: u64) {
    CLOCK_FREQ_HZ.store(freq_hz, Ordering::SeqCst);
}

/// 初始化时间基线。
pub fn init_time_baseline() {
    TIME_INIT.store(riscv::register::time::read() as u64, Ordering::SeqCst);
}

/// 设置墙钟时间偏移。
pub fn set_epoch_offset_nanos(offset: u64) {
    EPOCH_OFFSET_NANOS.store(offset, Ordering::SeqCst);
}

/// 获取时钟频率（Hz）。
pub fn clock_frequency() -> u64 {
    CLOCK_FREQ_HZ.load(Ordering::Relaxed)
}

/// 读取原始计时器值。
#[inline]
pub fn read_timer() -> usize {
    riscv::register::time::read()
}

/// 读取自基线以来的 tick。
#[inline]
pub fn current_ticks() -> u64 {
    let current = riscv::register::time::read() as u64;
    let init = TIME_INIT.load(Ordering::Relaxed);
    current.saturating_sub(init)
}

/// 读取周期计数器。
pub fn read_cycle() -> usize {
    riscv::register::cycle::read()
}

/// ticks 转纳秒。
#[inline]
pub fn ticks_to_nanos(ticks: u64) -> u64 {
    let freq = CLOCK_FREQ_HZ.load(Ordering::Relaxed);
    if freq == 0 {
        return 0;
    }
    (ticks as u128 * 1_000_000_000 / freq as u128) as u64
}

/// 纳秒转 ticks。
#[inline]
pub fn nanos_to_ticks(nanos: u64) -> u64 {
    let freq = CLOCK_FREQ_HZ.load(Ordering::Relaxed);
    (nanos as u128 * freq as u128 / 1_000_000_000) as u64
}

/// 获取纪元偏移（纳秒）。
#[inline]
pub fn epochoffset_nanos() -> u64 {
    EPOCH_OFFSET_NANOS.load(Ordering::Relaxed)
}

/// 获取单调时间（纳秒）。
#[inline]
pub fn monotonic_time_nanos() -> u64 {
    ticks_to_nanos(current_ticks())
}

/// 获取墙钟时间（纳秒）。
#[inline]
pub fn wall_time_nanos() -> u64 {
    monotonic_time_nanos() + epochoffset_nanos()
}

// ==================== 分页相关 ====================

/// 激活页表（Sv39）。
pub fn activate_paging_mode(page_table_token: usize) {
    sfence_vma_all();
    satp::write(page_table_token);
    sfence_vma_all();
}

/// 刷新全部 TLB。
pub fn sfence_vma_all() {
    riscv::asm::sfence_vma_all()
}

/// 刷新指定虚拟地址的 TLB。
pub fn sfence_vma(vaddr: usize) {
    unsafe {
        asm!(
            "sfence.vma {}, zero",
            in(reg) vaddr,
            options(nostack, preserves_flags)
        );
    }
}

/// 允许内核访问用户内存（SUM=1）。
pub fn allow_access_user_memory() {
    unsafe {
        riscv::register::sstatus::set_sum();
    }
}

/// 禁止内核访问用户内存（SUM=0）。
pub fn disallow_access_user_memory() {
    unsafe {
        riscv::register::sstatus::clear_sum();
    }
}

// ==================== 等待/停机 ====================

/// 等待中断。
#[inline(always)]
pub fn wfi() {
    riscv::asm::wfi();
}
