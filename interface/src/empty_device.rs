use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use shared_heap::DVec;

use super::AlienResult;
use crate::Basic;

#[proxy(EmptyDeviceDomainProxy, SRCU)]
pub trait EmptyDeviceDomain: Basic + DowncastSync {
    /// 初始化空设备域（占位设备）
    fn init(&self) -> AlienResult<()>;
    /// 读取数据，返回填充后的缓冲区
    fn read(&self, data: DVec<u8>) -> AlienResult<DVec<u8>>;
    /// 写入数据并返回写入字节数
    fn write(&self, data: &DVec<u8>) -> AlienResult<usize>;
}

impl_downcast!(sync EmptyDeviceDomain);
