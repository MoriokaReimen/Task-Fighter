use crate::monthly_task::MonthlyTask;
use anyhow::Result;
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MonthlyTaskFilterFlags: u32 {
        const Active           = 1 << 0;
        const Inactive         = 1 << 1;
        const PriorityLow      = 1 << 2;
        const PriorityMiddle   = 1 << 3;
        const PriorityHigh     = 1 << 4;
        const All = Self::Active.bits()
            | Self::Inactive.bits()
            | Self::PriorityLow.bits()
            | Self::PriorityMiddle.bits()
            | Self::PriorityHigh.bits();
        const AllPriorities = Self::PriorityLow.bits() | Self::PriorityMiddle.bits() | Self::PriorityHigh.bits();
    }
}

impl Default for MonthlyTaskFilterFlags {
    fn default() -> Self {
        MonthlyTaskFilterFlags::All
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MonthlyTaskSearchFlags: u32 {
        const SearchTitle      = 1 << 0;
        const SearchProject    = 1 << 1;
        const SearchDetail     = 1 << 2;
        const EnableRegex      = 1 << 3;
    }
}

impl Default for MonthlyTaskSearchFlags {
    fn default() -> Self {
        MonthlyTaskSearchFlags::SearchTitle
            | MonthlyTaskSearchFlags::SearchProject
            | MonthlyTaskSearchFlags::SearchDetail
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MonthlyTaskOrderFlags: u32 {
        const OrderByStartDate = 1 << 0;
        const OrderByDueDate   = 1 << 1;
        const OrderByPriority  = 1 << 2;
        const Reversed         = 1 << 3;
    }
}

impl Default for MonthlyTaskOrderFlags {
    fn default() -> Self {
        MonthlyTaskOrderFlags::OrderByPriority
    }
}

pub trait MonthlyTaskRecord {
    type AsyncOutput;
    fn get_next_monthly_task_id(&self) -> Result<i32>;
    fn fetch_one_monthly_task(&self, id: i32) -> Self::AsyncOutput;
    fn fetch_all_monthly_task(
        &self,
        filter_flags: MonthlyTaskFilterFlags,
        order_flags: MonthlyTaskOrderFlags,
    ) -> Self::AsyncOutput;
    fn search_monthly_task(
        &self,
        pattern: &str,
        search_flags: MonthlyTaskSearchFlags,
        filter_flags: MonthlyTaskFilterFlags,
        order_flags: MonthlyTaskOrderFlags,
    ) -> Self::AsyncOutput;
    fn insert_monthly_task(&self, task: &MonthlyTask) -> Self::AsyncOutput;
    fn update_monthly_task(&self, task: &MonthlyTask) -> Self::AsyncOutput;
    fn upsert_monthly_task(&self, task: &MonthlyTask) -> Self::AsyncOutput;
    fn delete_monthly_task(&self, id: i32) -> Self::AsyncOutput;
    fn sync_all_monthly_task(&self) -> Self::AsyncOutput;
}
