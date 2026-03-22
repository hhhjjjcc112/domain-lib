use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;

use super::AlienResult;
use crate::Basic;

#[proxy(SysCallDomainProxy, SRCU)]
pub trait SysCallDomain: Basic + DowncastSync {
    /// 初始化系统调用域
    fn init(&self) -> AlienResult<()>;
    /// 执行系统调用 `syscall_id`，参数为 `args`，返回值为系统调用返回值或错误
    fn call(&self, syscall_id: usize, args: [usize; 6]) -> AlienResult<isize>;
}

impl_downcast!(sync SysCallDomain);
