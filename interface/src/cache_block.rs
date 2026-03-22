use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use shared_heap::DVec;

use super::AlienResult;
use crate::{Basic, DeviceBase};

#[proxy(CacheBlkDomainProxy, RwLock, String)]
pub trait CacheBlkDeviceDomain: DeviceBase + Basic + DowncastSync {
    /// 初始化缓存块设备域
    fn init(&self, blk_domain_name: &str) -> AlienResult<()>;
    /// 从偏移 `offset` 读取数据并返回填充后的缓冲区
    fn read(&self, offset: u64, buf: DVec<u8>) -> AlienResult<DVec<u8>>;
    /// 在偏移 `offset` 写入数据，返回写入字节数
    fn write(&self, offset: u64, buf: &DVec<u8>) -> AlienResult<usize>;
    /// 获取设备容量
    fn get_capacity(&self) -> AlienResult<u64>;
    /// 刷新缓存到后端设备
    fn flush(&self) -> AlienResult<()>;
}

impl_downcast!(sync CacheBlkDeviceDomain);
