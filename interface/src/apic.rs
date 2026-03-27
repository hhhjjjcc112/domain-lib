use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use shared_heap::DVec;

use super::AlienResult;
use crate::Basic;

#[proxy(APICDomainProxy, RwLock)]
pub trait APICDomain: Basic + DowncastSync {
    /// 初始化 APIC 域。
    fn init(&self) -> AlienResult<()>;
    /// 处理指定 IRQ 对应的外部中断。
    fn handle_irq(&self, irq: usize) -> AlienResult<()>;
    /// 注册 IRQ 到设备域名。
    fn register_irq(&self, irq: usize, device_domain_name: &DVec<u8>) -> AlienResult<()>;
    /// 导出 IRQ 统计信息。
    fn irq_info(&self, buf: DVec<u8>) -> AlienResult<DVec<u8>>;
}

impl_downcast!(sync APICDomain);
