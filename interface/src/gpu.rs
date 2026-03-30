use core::ops::Range;

use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use shared_heap::DVec;

use super::AlienResult;
use crate::{Basic, DeviceBase, VirtioInitInfo};

#[proxy(GpuDomainProxy,RwLock,VirtioInitInfo)]
pub trait GpuDomain: DeviceBase + Basic + DowncastSync {
    /// 初始化 GPU 设备域
    fn init(&self, device_info: &VirtioInitInfo) -> AlienResult<()>;
    /// 刷新 GPU 缓冲/提交命令
    fn flush(&self) -> AlienResult<()>;
    /// 在 GPU 缓冲区 `offset` 位置填充数据，返回写入字节数
    fn fill(&self, offset: u32, buf: &DVec<u8>) -> AlienResult<usize>;
    /// 获取 GPU 缓冲区的地址范围
    fn buffer_range(&self) -> AlienResult<Range<usize>>;
}

impl_downcast!(sync GpuDomain);
