use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use shared_heap::{DBox, DVec};
use vfscore::{fstype::FileSystemFlags, inode::InodeAttr, superblock::SuperType, utils::*};

use super::AlienResult;
use crate::{Basic, DirEntryWrapper, InodeID};

#[proxy(FsDomainProxy, RwLock)]
pub trait FsDomain: Basic + DowncastSync {
    /// 初始化文件系统
    fn init(&self) -> AlienResult<()>;
    /// 在指定挂载点 `mp` 上挂载设备（可选 dev_inode）并返回根 inode
    fn mount(&self, mp: &DVec<u8>, dev_inode: Option<DBox<MountInfo>>) -> AlienResult<InodeID>;
    /// 获取根 inode id
    fn root_inode_id(&self) -> AlienResult<InodeID>;
    /// 释放指定的 inode
    fn drop_inode(&self, inode: InodeID) -> AlienResult<()>;

    /// 获取目录项的名称
    fn dentry_name(&self, inode: InodeID, buf: DVec<u8>) -> AlienResult<(DVec<u8>, usize)>;
    /// 获取目录项的路径
    fn dentry_path(&self, inode: InodeID, buf: DVec<u8>) -> AlienResult<(DVec<u8>, usize)>;
    /// `domain_ident` 是父文件系统域名称
    /// 设置目录项的父域和父 inode
    fn dentry_set_parent(
        &self,
        inode: InodeID,
        domain_ident: &DVec<u8>,
        parent: InodeID,
    ) -> AlienResult<()>;
    /// 获取目录项的父 inode（如果存在）
    fn dentry_parent(&self, inode: InodeID) -> AlienResult<Option<InodeID>>;

    /// 将目录项设置为挂载点
    fn dentry_to_mount_point(
        &self,
        inode: InodeID,
        domain_ident: &DVec<u8>,
        mount_inode_id: InodeID,
    ) -> AlienResult<()>;
    /// 获取目录项的挂载点信息（域标识与挂载 inode）
    fn dentry_mount_point(
        &self,
        inode: InodeID,
        domain_ident: DVec<u8>,
    ) -> AlienResult<Option<(DVec<u8>, InodeID)>>;
    /// 清除目录项的挂载点信息
    fn dentry_clear_mount_point(&self, inode: InodeID) -> AlienResult<()>;
    /// 在目录 `inode` 下按 `name` 查找子项的 inode
    fn dentry_find(&self, inode: InodeID, name: &DVec<u8>) -> AlienResult<Option<InodeID>>;
    /// 从目录中移除名为 `name` 的目录项
    fn dentry_remove(&self, inode: InodeID, name: &DVec<u8>) -> AlienResult<()>;

    // 文件操作
    /// 从指定偏移 `offset` 读取文件数据
    fn read_at(&self, inode: InodeID, offset: u64, buf: DVec<u8>)
        -> AlienResult<(DVec<u8>, usize)>;
    /// 在指定偏移 `offset` 写入数据并返回写入字节数
    fn write_at(&self, inode: InodeID, offset: u64, buf: &DVec<u8>) -> AlienResult<usize>;
    /// 读取目录项（按索引）并返回目录项结构
    fn readdir(
        &self,
        inode: InodeID,
        start_index: usize,
        entry: DBox<DirEntryWrapper>,
    ) -> AlienResult<DBox<DirEntryWrapper>>;
    /// 查询文件/节点的可用事件掩码
    fn poll(&self, inode: InodeID, mask: VfsPollEvents) -> AlienResult<VfsPollEvents>;
    /// I/O 控制操作（ioctl）
    fn ioctl(&self, inode: InodeID, cmd: u32, arg: usize) -> AlienResult<usize>;
    /// 刷新文件相关缓冲区
    fn flush(&self, inode: InodeID) -> AlienResult<()>;
    /// 将文件数据同步到存储设备
    fn fsync(&self, inode: InodeID) -> AlienResult<()>;

    // inode 操作
    /// 删除子目录（仅删除空目录）
    fn rmdir(&self, parent: InodeID, name: &DVec<u8>) -> AlienResult<()>;
    /// 查询 inode 的权限信息
    fn node_permission(&self, inode: InodeID) -> AlienResult<VfsNodePerm>;
    /// 在父目录下创建新节点并返回其 inode id
    fn create(
        &self,
        parent: InodeID,
        name: &DVec<u8>,
        ty: VfsNodeType,
        perm: VfsNodePerm,
        rdev: Option<u64>,
    ) -> AlienResult<InodeID>;
    /// 创建硬链接并返回目标 inode
    fn link(&self, parent: InodeID, name: &DVec<u8>, src: InodeID) -> AlienResult<InodeID>;
    /// 删除目录项或链接
    fn unlink(&self, parent: InodeID, name: &DVec<u8>) -> AlienResult<()>;
    /// 创建符号链接并返回其 inode
    fn symlink(&self, parent: InodeID, name: &DVec<u8>, link: &DVec<u8>) -> AlienResult<InodeID>;
    /// 在父目录中查找名称对应的 inode
    fn lookup(&self, parent: InodeID, name: &DVec<u8>) -> AlienResult<InodeID>;
    /// 读取符号链接的内容
    fn readlink(&self, inode: InodeID, buf: DVec<u8>) -> AlienResult<(DVec<u8>, usize)>;
    /// 设置 inode 的属性
    fn set_attr(&self, inode: InodeID, attr: InodeAttr) -> AlienResult<()>;
    /// 获取 inode 的属性信息
    fn get_attr(&self, inode: InodeID) -> AlienResult<VfsFileStat>;
    /// 获取 inode 的类型
    fn inode_type(&self, inode: InodeID) -> AlienResult<VfsNodeType>;
    /// 截断文件到指定长度
    fn truncate(&self, inode: InodeID, len: u64) -> AlienResult<()>;
    /// 重命名或移动文件/目录
    fn rename(
        &self,
        old_parent: InodeID,
        old_name: &DVec<u8>,
        new_parent: InodeID,
        new_name: &DVec<u8>,
        flags: VfsRenameFlag,
    ) -> AlienResult<()>;
    /// 更新 inode 的时间信息
    fn update_time(&self, inode: InodeID, time: VfsTime, now: VfsTimeSpec) -> AlienResult<()>;

    // 超级块操作
    /// 同步文件系统，可选择是否等待完成
    fn sync_fs(&self, wait: bool) -> AlienResult<()>;
    /// 获取文件系统统计信息
    fn stat_fs(&self, fs_stat: DBox<VfsFsStat>) -> AlienResult<DBox<VfsFsStat>>;
    /// 获取文件系统的超级块类型
    fn super_type(&self) -> AlienResult<SuperType>;

    // 文件系统信息
    /// 销毁/卸载超级块
    fn kill_sb(&self) -> AlienResult<()>;
    /// 获取文件系统标志
    fn fs_flag(&self) -> AlienResult<FileSystemFlags>;
    /// 获取文件系统名称
    fn fs_name(&self, name: DVec<u8>) -> AlienResult<(DVec<u8>, usize)>;
    /// 获取文件系统魔数（magic）
    fn fs_magic(&self) -> AlienResult<u128>;
}

impl_downcast!(sync FsDomain);

#[proxy(DevFsDomainProxy, RwLock)]
pub trait DevFsDomain: FsDomain + DowncastSync {
    fn register(&self, rdev: u64, device_domain_name: &DVec<u8>) -> AlienResult<()>;
}

impl_downcast!(sync DevFsDomain);

pub struct MountInfo {
    pub mount_inode_id: InodeID,
    pub domain_ident: [u8; 32],
}
