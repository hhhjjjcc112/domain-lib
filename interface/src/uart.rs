use core::ops::Range;

use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use shared_heap::DVec;

use super::{AlienError, AlienResult};
use crate::{Basic, DeviceBase};
#[proxy(UartDomainProxy,RwLock,Range<usize>)]
pub trait UartDomain: DeviceBase + Basic + DowncastSync {
    fn init(&self, device_info: &Range<usize>) -> AlienResult<()>;
    /// 向 UART 写入一个字符
    fn putc(&self, ch: u8) -> AlienResult<()>;
    /// 从 UART 读取一个字符
    fn getc(&self) -> AlienResult<Option<u8>>;

    /// 向 UART 写入多个字节并返回写入字节数
    fn put_bytes(&self, buf: &DVec<u8>) -> AlienResult<usize>;
    /// 检查 UART 是否有数据可读
    fn have_data_to_get(&self) -> AlienResult<bool>;
    /// 检查 UART 是否有空间可写
    fn have_space_to_put(&self) -> AlienResult<bool> {
        Ok(true)
    }
    fn enable_receive_interrupt(&self) -> AlienResult<()>;
    fn disable_receive_interrupt(&self) -> AlienResult<()>;
    fn enable_transmit_interrupt(&self) -> AlienResult<()> {
        Err(AlienError::ENOSYS)
    }
    fn disable_transmit_interrupt(&self) -> AlienResult<()> {
        Err(AlienError::ENOSYS)
    }
}

impl_downcast!(sync UartDomain);
