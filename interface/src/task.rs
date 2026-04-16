use core::ops::Range;

use downcast_rs::{DowncastSync, impl_downcast};
use gproxy::proxy;
use pod::Pod;
use shared_heap::{DBox, DVec};

use super::AlienResult;
use crate::{Basic, vfs::InodeID};
#[proxy(TaskDomainProxy, RwLock)]
pub trait TaskDomain: Basic + DowncastSync {
    /// 初始化任务域
    fn init(&self) -> AlienResult<()>;
    /// 返回页表 token 以及陷阱帧在虚拟地址空间中的虚拟地址
    fn page_table_token_with_trap_frame_virt_addr(&self) -> AlienResult<(usize, usize)>;
    /// 返回陷阱帧的物理地址
    fn trap_frame_phy_addr(&self) -> AlienResult<usize>;
    /// 获取/设置临时堆信息
    fn heap_info(&self, tmp_heap_info: DBox<TmpHeapInfo>) -> AlienResult<DBox<TmpHeapInfo>>;
    /// 根据 fd 获取对应的 inode id
    fn get_fd(&self, fd: usize) -> AlienResult<InodeID>;
    /// 为 inode 分配一个新的 fd
    fn add_fd(&self, inode: InodeID) -> AlienResult<usize>;
    /// 移除并返回 fd 对应的 inode id
    fn remove_fd(&self, fd: usize) -> AlienResult<InodeID>;
    /// 获取文件系统信息（如根 inode 等）
    fn fs_info(&self) -> AlienResult<(InodeID, InodeID)>;
    /// 设置当前工作目录（cwd）为指定 inode
    fn set_cwd(&self, inode: InodeID) -> AlienResult<()>;
    /// 读取并更新当前任务的 umask
    fn do_umask(&self, mask: u32) -> AlienResult<u32>;
    /// 将数据拷贝到用户地址空间
    fn copy_to_user(&self, dst: usize, buf: &[u8]) -> AlienResult<()>;
    /// 从用户地址空间拷贝数据到内核
    fn copy_from_user(&self, src: usize, buf: &mut [u8]) -> AlienResult<()>;
    /// 从用户空间读取以 NUL 结尾的字符串
    fn read_string_from_user(&self, src: usize, buf: DVec<u8>) -> AlienResult<(DVec<u8>, usize)>;
    /// 获取当前线程/进程 pid
    fn current_pid(&self) -> AlienResult<usize>;
    /// 获取当前线程/进程的父 pid
    fn current_ppid(&self) -> AlienResult<usize>;
    /// 获取当前线程/进程组 ID
    fn current_pgid(&self) -> AlienResult<usize>;
    /// 获取当前线程/进程会话 ID
    fn current_sid(&self) -> AlienResult<usize>;
    /// 设置或查询程序 break（sbrk/brk）
    fn do_brk(&self, addr: usize) -> AlienResult<isize>;
    /// 创建进程/线程（clone）
    fn do_clone(
        &self,
        flags: usize,
        stack: usize,
        ptid: usize,
        tls: usize,
        ctid: usize,
    ) -> AlienResult<isize>;
    /// 等待子进程（wait4）
    fn do_wait4(
        &self,
        pid: isize,
        exit_code_ptr: usize,
        options: u32,
        _rusage: usize,
    ) -> AlienResult<isize>;
    /// 执行 execve
    fn do_execve(
        &self,
        filename_ptr: usize,
        argv_ptr: usize,
        envp_ptr: usize,
    ) -> AlienResult<isize>;
    /// 设置线程 ID 存放地址
    fn do_set_tid_address(&self, tidptr: usize) -> AlienResult<isize>;
    #[cfg(target_arch = "x86_64")]
    /// 设置当前任务的 FS TLS 基址
    fn do_set_fs_base(&self, fs_base: usize) -> AlienResult<()>;
    #[cfg(target_arch = "x86_64")]
    /// 获取当前任务的 FS TLS 基址
    fn do_get_fs_base(&self) -> AlienResult<usize>;
    #[cfg(target_arch = "x86_64")]
    /// 设置当前任务的用户 GS 基址
    fn do_set_gs_base(&self, gs_base: usize) -> AlienResult<()>;
    #[cfg(target_arch = "x86_64")]
    /// 获取当前任务的用户 GS 基址
    fn do_get_gs_base(&self) -> AlienResult<usize>;
    /// 读取指定进程的进程组 ID
    fn do_get_pgid(&self, pid: usize) -> AlienResult<usize>;
    /// 读取指定进程的会话 ID
    fn do_get_sid(&self, pid: usize) -> AlienResult<usize>;
    /// 设置当前任务的进程组 ID
    fn do_set_pgid(&self, pid: usize, pgid: usize) -> AlienResult<isize>;
    /// 创建新会话
    fn do_set_sid(&self) -> AlienResult<isize>;
    /// 内存映射（mmap）
    fn do_mmap(
        &self,
        start: usize,
        len: usize,
        prot: u32,
        flags: u32,
        fd: usize,
        offset: usize,
    ) -> AlienResult<isize>;
    /// 解除内存映射（munmap）
    fn do_munmap(&self, start: usize, len: usize) -> AlienResult<isize>;
    /// 信号处理：设置/查询 sigaction
    fn do_sigaction(&self, signum: u8, act: usize, oldact: usize) -> AlienResult<isize>;
    /// 信号屏蔽（sigprocmask）
    fn do_sigprocmask(&self, how: usize, set: usize, oldset: usize) -> AlienResult<isize>;
    /// 文件控制（fcntl）
    fn do_fcntl(&self, fd: usize, cmd: usize) -> AlienResult<(InodeID, usize)>;
    /// 设置/查询资源限制（prlimit）
    fn do_prlimit(
        &self,
        pid: usize,
        resource: usize,
        new_limit: usize,
        old_limit: usize,
    ) -> AlienResult<isize>;
    /// 复制 fd（dup）
    fn do_dup(&self, old_fd: usize, new_fd: Option<usize>) -> AlienResult<isize>;
    /// 创建管道（pipe2）
    fn do_pipe2(&self, r: InodeID, w: InodeID, pipe: usize) -> AlienResult<isize>;
    /// 退出进程/线程
    fn do_exit(&self, exit_code: isize) -> AlienResult<isize>;
    /// 将物理设备映射到用户空间
    fn do_mmap_device(&self, phy_addr_range: Range<usize>) -> AlienResult<isize>;
    /// 设置进程优先级
    fn do_set_priority(&self, which: i32, who: u32, priority: i32) -> AlienResult<()>;
    /// 获取进程优先级
    fn do_get_priority(&self, which: i32, who: u32) -> AlienResult<i32>;
    /// 设置/获取信号栈
    fn do_signal_stack(&self, ss: usize, oss: usize) -> AlienResult<isize>;
    /// 更改内存保护属性（mprotect）
    fn do_mprotect(&self, addr: usize, len: usize, prot: u32) -> AlienResult<isize>;
    /// 处理加载阶段的缺页异常
    fn do_load_page_fault(&self, addr: usize) -> AlienResult<()>;
    /// futex 操作
    fn do_futex(
        &self,
        uaddr: usize,
        futex_op: u32,
        val: u32,
        timeout: usize,
        uaddr2: usize,
        val3: u32,
    ) -> AlienResult<isize>;
}

#[derive(Debug, Default)]
pub struct TmpHeapInfo {
    pub start: usize,
    pub current: usize,
}

impl dyn TaskDomain {
    pub fn read_val_from_user<T: Pod>(&self, src: usize) -> AlienResult<T> {
        let mut val = T::new_uninit();
        self.copy_from_user(src, val.as_bytes_mut())?;
        Ok(val)
    }

    pub fn write_val_to_user<T: Pod>(&self, dst: usize, val: &T) -> AlienResult<()> {
        self.copy_to_user(dst, val.as_bytes())
    }
}

impl_downcast!(sync TaskDomain);
