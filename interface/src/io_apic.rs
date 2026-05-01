use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use shared_heap::DVec;

use super::AlienResult;
use crate::Basic;

#[derive(Clone, Copy, Debug)]
pub struct IoAPICHooks {
    pub ioapic_base: usize,
}

#[proxy(IoAPICDomainProxy, RwLock, IoAPICHooks)]
pub trait IoAPICDomain: Basic + DowncastSync {
    /// 初始化 I/O APIC 域。
    fn init(&self, hooks: &IoAPICHooks) -> AlienResult<()>;
    /// 配置 IRQ 重定向项。
    fn configure_irq(&self, irq: u8, vector: u8, dest_cpu: u8) -> AlienResult<()>;
    /// 使能或屏蔽指定 IRQ。
    fn set_irq_enable(&self, vector: usize, enabled: bool) -> AlienResult<()>;
    /// 返回 I/O APIC 最大重定向项数。
    fn ioapic_max_entries(&self) -> AlienResult<u8>;
    /// 处理指定 IRQ 对应的外部中断。
    fn handle_irq(&self, irq: usize) -> AlienResult<()>;
    /// 注册 IRQ 到设备域名。
    fn register_irq(&self, irq: usize, device_domain_name: &DVec<u8>) -> AlienResult<()>;
    /// 导出 IRQ 统计信息。
    fn irq_info(&self, buf: DVec<u8>) -> AlienResult<DVec<u8>>;
}

impl_downcast!(sync IoAPICDomain);