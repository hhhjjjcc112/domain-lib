use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use shared_heap::DVec;

use super::AlienResult;
use crate::Basic;

#[proxy(LogDomainProxy, SRCU)]
pub trait LogDomain: Basic + DowncastSync {
    fn init(&self) -> AlienResult<()>;
    fn log(&self, level: Level, msg: &DVec<u8>) -> AlienResult<()>;
    fn set_max_level(&self, level: LevelFilter) -> AlienResult<()>;
}

impl_downcast!(sync LogDomain);

#[repr(usize)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Level {
    /// `error` 级别。
    ///
    /// 表示严重错误。
    // 这里与 LevelFilter 的判别值保持对齐。
    Error = 1,
    /// `warn` 级别。
    ///
    /// 表示潜在风险。
    Warn,
    /// `info` 级别。
    ///
    /// 表示常规信息。
    Info,
    /// `debug` 级别。
    ///
    /// 表示调试信息。
    Debug,
    /// `trace` 级别。
    ///
    /// 表示最细粒度的跟踪信息。
    Trace,
}

/// 日志过滤级别枚举。
#[repr(usize)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum LevelFilter {
    /// 关闭所有日志。
    Off,
    /// 对应 `Error`。
    Error,
    /// 对应 `Warn`。
    Warn,
    /// 对应 `Info`。
    Info,
    /// 对应 `Debug`。
    Debug,
    /// 对应 `Trace`。
    Trace,
}
