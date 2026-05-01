use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;

use super::AlienResult;
use crate::Basic;

#[derive(Clone, Copy, Debug)]
pub struct LocalAPICHooks {
    pub xapic_base: usize,
}

#[proxy(LocalAPICDomainProxy, SRCU, LocalAPICHooks)]
pub trait LocalAPICDomain: Basic + DowncastSync {
    /// 初始化 Local APIC 域。
    fn init(&self, hooks: &LocalAPICHooks) -> AlienResult<()>;
    /// 直接编程下一次 APIC timer 触发点。
    fn set_timer(&self, next_deadline: usize) -> AlienResult<()>;
    /// 发送 APIC EOI。
    fn eoi(&self) -> AlienResult<()>;
    /// 向指定 CPU 发送 IPI。
    fn send_ipi(&self, target_cpu: usize, vector: u8) -> AlienResult<()>;
    /// 向自身发送 IPI。
    fn send_ipi_self(&self, vector: u8) -> AlienResult<()>;
    /// 向除自身外的所有 CPU 发送 IPI。
    fn send_ipi_all_excluding_self(&self, vector: u8) -> AlienResult<()>;
    /// 读取 Local APIC 错误状态寄存器（ESR）。
    fn get_error_status(&self) -> AlienResult<u32>;
}

impl_downcast!(sync LocalAPICDomain);
