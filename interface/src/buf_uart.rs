use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use shared_heap::DVec;

use super::AlienResult;
use crate::{Basic, DeviceBase};

#[proxy(BufUartDomainProxy, RwLock, String)]
pub trait BufUartDomain: DeviceBase + Basic + DowncastSync {
    fn init(&self, uart_domain_name: &str) -> AlienResult<()>;
    /// 向 UART 写入一个字符
    fn putc(&self, ch: u8) -> AlienResult<()>;
    /// 从 UART 读取一个字符
    fn getc(&self) -> AlienResult<Option<u8>>;
    fn put_bytes(&self, buf: &DVec<u8>) -> AlienResult<usize>;
    /// 检查 UART 是否有数据可读
    fn have_data_to_get(&self) -> AlienResult<bool>;
    /// 检查 UART 是否有空间可写
    fn have_space_to_put(&self) -> AlienResult<bool> {
        Ok(true)
    }
}

impl_downcast!(sync BufUartDomain);
