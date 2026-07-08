use crate::task::Task;
use anyhow::Result;
use bitflags::bitflags;

pub type PlotResult = Result<Vec<(i32, i32, i32, i32)>>;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TaskFilterFlags: u32 {
        const Zero             = 0;
        const Active           = 1 << 0;
        const Inactive         = 1 << 1;
        const PriorityLow      = 1 << 2;
        const PriorityMiddle   = 1 << 3;
        const PriorityHigh     = 1 << 4;
        const StatusPending    = 1 << 5;
        const StatusWIP        = 1 << 6;
        const StatusComplete   = 1 << 7;
        const StatusCanceled   = 1 << 8;

        const ALL = Self::Active.bits()
            | Self::Inactive.bits()
            | Self::PriorityLow.bits()
            | Self::PriorityMiddle.bits()
            | Self::PriorityHigh.bits()
            | Self::StatusPending.bits()
            | Self::StatusWIP.bits()
            | Self::StatusComplete.bits()
            | Self::StatusCanceled.bits();

        const ALL_PRIORITIES = Self::PriorityLow.bits() | Self::PriorityMiddle.bits() | Self::PriorityHigh.bits();
        const ALL_STATUSES   = Self::StatusPending.bits() | Self::StatusWIP.bits() | Self::StatusComplete.bits() | Self::StatusCanceled.bits();
    }
}

impl Default for TaskFilterFlags {
    fn default() -> Self {
        TaskFilterFlags::ALL
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TaskSearchFlags: u32 {
        const Zero             = 0;
        const SearchTitle      = 1 << 0;
        const SearchProject    = 1 << 1;
        const SearchDetail     = 1 << 2;
        const EnableRegex      = 1 << 3;
    }
}

impl Default for TaskSearchFlags {
    fn default() -> Self {
        TaskSearchFlags::SearchTitle
            | TaskSearchFlags::SearchProject
            | TaskSearchFlags::SearchDetail
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TaskOrderFlags: u32 {
        const Zero             = 0;
        const OrderByStatus    = 1 << 0;
        const OrderByStartDate = 1 << 1;
        const OrderByDueDate   = 1 << 2;
        const OrderByEntryDate = 1 << 3;
        const OrderByEndDate   = 1 << 4;
        const OrderByPriority  = 1 << 5;
        const OrderByProgress  = 1 << 6;
        const OrderByTimeSpent = 1 << 7;
        const Reversed         = 1 << 8;
    }
}

impl Default for TaskOrderFlags {
    fn default() -> Self {
        TaskOrderFlags::OrderByPriority
    }
}

pub trait TaskRecord {
    type AsyncOutput;
    fn get_next_task_id(&self) -> Result<i32>;
    fn fetch_one_task(&self, id: i32) -> Self::AsyncOutput;
    fn fetch_all_task(
        &self,
        filter_flags: TaskFilterFlags,
        order_flags: TaskOrderFlags,
    ) -> Self::AsyncOutput;
    fn search_task(
        &self,
        pattern: &str,
        search_flags: TaskSearchFlags,
        filter_flags: TaskFilterFlags,
        order_flags: TaskOrderFlags,
    ) -> Self::AsyncOutput;
    fn insert_task(&self, task: &Task) -> Self::AsyncOutput;
    fn update_task(&self, task: &Task) -> Self::AsyncOutput;
    fn upsert_task(&self, task: &Task) -> Self::AsyncOutput;
    fn get_plot_data(&self) -> Self::AsyncOutput;
    fn mail_daily(&self, tasks: &[Task]) -> Self::AsyncOutput;
}
