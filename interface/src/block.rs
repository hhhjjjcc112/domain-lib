use core::ops::Range;

use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use shared_heap::DVec;

use super::AlienResult;
use crate::{Basic, DeviceBase};

#[proxy(BlkDomainProxy,RwLock,Range<usize>)]
pub trait BlkDeviceDomain: DeviceBase + Basic + DowncastSync {
    /// 初始化块设备驱动
    fn init(&self, device_info: &Range<usize>) -> AlienResult<()>;
    /// 读取指定块的数据并返回填充后的缓冲区
    fn read_block(&self, block: u32, data: DVec<u8>) -> AlienResult<DVec<u8>>;
    /// 写入指定块的数据，返回写入字节数
    fn write_block(&self, block: u32, data: &DVec<u8>) -> AlienResult<usize>;
    /// 获取设备容量（以字节为单位）
    fn get_capacity(&self) -> AlienResult<u64>;
    /// 刷新设备缓存/持久化元数据
    fn flush(&self) -> AlienResult<()>;
}

impl_downcast!(sync  BlkDeviceDomain);
