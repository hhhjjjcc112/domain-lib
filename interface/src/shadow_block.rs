use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::{no_check, proxy};
use shared_heap::DVec;

use super::AlienResult;
use crate::{Basic, DeviceBase};

#[proxy(ShadowBlockDomainProxy, SRCU, String)]
pub trait ShadowBlockDomain: DeviceBase + Basic + DowncastSync {
    /// 初始化影子块设备域
    fn init(&self, blk_domain: &str) -> AlienResult<()>;
    #[no_check]
    /// 读取指定块的数据（影子块实现）
    fn read_block(&self, block: u32, data: DVec<u8>) -> AlienResult<DVec<u8>>;
    #[no_check]
    /// 写入指定块的数据（影子块实现）
    fn write_block(&self, block: u32, data: &DVec<u8>) -> AlienResult<usize>;
    #[no_check]
    /// 获取设备容量
    fn get_capacity(&self) -> AlienResult<u64>;
    #[no_check]
    /// 刷新设备缓存
    fn flush(&self) -> AlienResult<()>;
}

impl_downcast!(sync  ShadowBlockDomain);
