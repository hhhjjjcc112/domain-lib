use core::ops::Range;

use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use shared_heap::DVec;

use super::AlienResult;
use crate::{Basic, DeviceBase};

#[proxy(NetDeviceDomainProxy,RwLock, Range<usize>)]
pub trait NetDeviceDomain: DeviceBase + Basic + DowncastSync {
    fn init(&self, device_info: &Range<usize>) -> AlienResult<()>;
    /// 网卡的 MAC 地址。
    fn mac_address(&self) -> AlienResult<[u8; 6]>;

    /// 是否可发送数据包。
    fn can_transmit(&self) -> AlienResult<bool>;

    /// 是否可接收数据包。
    fn can_receive(&self) -> AlienResult<bool>;

    /// 接收队列大小。
    fn rx_queue_size(&self) -> AlienResult<usize>;

    /// 发送队列大小。
    fn tx_queue_size(&self) -> AlienResult<usize>;

    /// 非阻塞发送缓冲区中的数据包。
    fn transmit(&self, tx_buf: &DVec<u8>) -> AlienResult<()>;

    /// 从网络接收一个数据包并写入缓冲区，返回该缓冲区及长度。
    ///
    /// 接收前，驱动应先把若干可用缓冲区放入接收队列。
    ///
    /// 若当前没有到达数据包，应返回“稍后重试”类错误。
    fn receive(&self, rx_buf: DVec<u8>) -> AlienResult<(DVec<u8>, usize)>;
}

impl_downcast!(sync NetDeviceDomain);
