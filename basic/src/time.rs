//! 时间相关操作
//!
//! 提供跨架构统一时间接口。

use core::time::Duration;

use pconst::time::{TimeSpec, TimeVal};

/// 每秒纳秒数
pub const NANOS_PER_SEC: u64 = 1_000_000_000;
/// 每秒微秒数
pub const MICROS_PER_SEC: u64 = 1_000_000;
/// 每秒毫秒数
pub const MILLIS_PER_SEC: u64 = 1_000;

// 基础时间读取

/// 读取原始计时器值（架构相关）
#[inline]
pub fn read_timer() -> usize {
    arch::read_timer()
}

/// 读取系统启动后的 tick 数
#[inline]
pub fn current_ticks() -> u64 {
    arch::current_ticks()
}

/// 将 ticks 转换为纳秒
#[inline]
pub fn ticks_to_nanos(ticks: u64) -> u64 {
    arch::ticks_to_nanos(ticks)
}

/// 将纳秒转换为 ticks
#[inline]
pub fn nanos_to_ticks(nanos: u64) -> u64 {
    arch::nanos_to_ticks(nanos)
}

// 单调时间（自启动以来）

/// 获取自启动以来的单调时间（纳秒）
#[inline]
pub fn monotonic_time_nanos() -> u64 {
    arch::monotonic_time_nanos()
}

/// 获取自启动以来的单调时间（Duration）
#[inline]
pub fn monotonic_time() -> Duration {
    Duration::from_nanos(monotonic_time_nanos())
}

/// 读取自启动以来的毫秒数
#[inline]
pub fn read_time_ms() -> u64 {
    monotonic_time_nanos() / 1_000_000
}

/// 读取自启动以来的微秒数
#[inline]
pub fn read_time_us() -> u64 {
    monotonic_time_nanos() / 1_000
}

/// 读取自启动以来的纳秒数
#[inline]
pub fn read_time_ns() -> u64 {
    monotonic_time_nanos()
}

// 实时时间（Unix 纪元以来）

/// 获取 Unix 纪元以来的实时时间（纳秒）
#[inline]
pub fn wall_time_nanos() -> u64 {
    arch::wall_time_nanos()
}

/// 获取 Unix 纪元以来的实时时间（Duration）
#[inline]
pub fn wall_time() -> Duration {
    Duration::from_nanos(wall_time_nanos())
}

/// 获取纪元偏移（纳秒）
#[inline]
pub fn epochoffset_nanos() -> u64 {
    arch::epochoffset_nanos()
}

// 忙等

/// 忙等指定时长
pub fn busy_wait(dur: Duration) {
    let deadline = monotonic_time() + dur;
    busy_wait_until(deadline);
}

/// 忙等直到到达指定截止时间（单调时间）
pub fn busy_wait_until(deadline: Duration) {
    while monotonic_time() < deadline {
        core::hint::spin_loop();
    }
}

// 兼容 trait

/// 将时间类型转换为时钟 ticks 的 trait
pub trait ToClock {
    fn to_clock(&self) -> usize;
}

/// 获取当前时间的 trait
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
