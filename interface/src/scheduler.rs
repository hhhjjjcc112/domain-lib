use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use shared_heap::DBox;
use task_meta::TaskSchedulingInfo;

use super::AlienResult;
use crate::Basic;

#[proxy(SchedulerDomainProxy, RwLock)]
pub trait SchedulerDomain: Basic + DowncastSync {
    fn init(&self) -> AlienResult<()>;
    /// 向调度器添加一个任务
    fn add_task(&self, scheduling_info: DBox<TaskSchedulingInfo>) -> AlienResult<()>;
    /// 选择下一个要运行的任务
    fn fetch_task(&self, info: DBox<TaskSchedulingInfo>) -> AlienResult<DBox<TaskSchedulingInfo>>;
}

impl_downcast!(sync SchedulerDomain);
