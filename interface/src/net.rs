use core::net::SocketAddrV4;

use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use pconst::{
    io::PollEvents,
    net::{Domain, ShutdownFlag, SocketAddrIn, SocketType},
};
use shared_heap::{DBox, DVec};

use super::AlienResult;
use crate::{Basic, DeviceBase};

pub type SocketID = usize;

#[proxy(NetDomainProxy, RwLock, String)]
pub trait NetDomain: DeviceBase + Basic + DowncastSync {
    /// 初始化网络域，指定网卡域名称
    fn init(&self, nic_domain_name: &str) -> AlienResult<()>;
    /// 创建一个 socket，返回 socket id
    fn socket(&self, s_domain: Domain, ty: SocketType, protocol: usize) -> AlienResult<SocketID>;
    /// 创建一对相连的 socket
    fn socket_pair(&self, s_domain: Domain, ty: SocketType) -> AlienResult<(SocketID, SocketID)>;
    /// 删除指定的 socket
    fn remove_socket(&self, socket_id: SocketID) -> AlienResult<()>;
    /// 绑定 socket 到指定地址，可能返回新的 socket id（例如监听套接字）
    fn bind(&self, socket_id: SocketID, addr: &DBox<SocketAddrIn>)
        -> AlienResult<Option<SocketID>>;
    /// 将 socket 设置为监听状态，指定 backlog
    fn listen(&self, socket_id: SocketID, backlog: usize) -> AlienResult<()>;
    /// 接受一个连接并返回新的 socket id
    fn accept(&self, socket_id: SocketID) -> AlienResult<SocketID>;
    /// 连接到远端地址
    fn connect(&self, socket_id: SocketID, addr: &DBox<SocketAddrV4>) -> AlienResult<()>;

    /// 从 socket 接收数据，并填充 `SocketArgTuple`
    fn recv_from(
        &self,
        socket_id: SocketID,
        arg_tuple: DBox<SocketArgTuple>,
    ) -> AlienResult<DBox<SocketArgTuple>>;
    /// 发送数据到指定远端（或直接发送）并返回发送字节数
    fn sendto(
        &self,
        socket_id: SocketID,
        buf: &DVec<u8>,
        remote_addr: Option<&DBox<SocketAddrV4>>,
    ) -> AlienResult<usize>;
    /// 关闭或部分关闭 socket（根据 how）
    fn shutdown(&self, socket_id: SocketID, how: ShutdownFlag) -> AlienResult<()>;

    /// 获取远端地址信息
    fn remote_addr(
        &self,
        socket_id: SocketID,
        addr: DBox<SocketAddrIn>,
    ) -> AlienResult<DBox<SocketAddrIn>>;
    /// 获取本地地址信息
    fn local_addr(
        &self,
        socket_id: SocketID,
        addr: DBox<SocketAddrIn>,
    ) -> AlienResult<DBox<SocketAddrIn>>;
    /// 从 socket 的偏移位置读取数据（用于流式接口）
    fn read_at(
        &self,
        socket_id: SocketID,
        offset: u64,
        buf: DVec<u8>,
    ) -> AlienResult<(DVec<u8>, usize)>;
    /// 向 socket 的偏移位置写入数据，返回写入字节数
    fn write_at(&self, socket_id: SocketID, offset: u64, buf: &DVec<u8>) -> AlienResult<usize>;
    /// 查询 socket 的事件（poll）
    fn poll(&self, socket_id: SocketID, events: PollEvents) -> AlienResult<PollEvents>;
}

pub struct SocketArgTuple {
    pub buf: DVec<u8>,
    pub addr: DBox<SocketAddrIn>,
    pub len: usize,
}

impl_downcast!(sync NetDomain);
