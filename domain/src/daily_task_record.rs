use crate::daily_task::DailyTask;
use anyhow::Result;
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DailyTaskFilterFlags: u32 {
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

impl Default for DailyTaskFilterFlags {
    fn default() -> Self {
        Self::All
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DailyTaskSearchFlags: u32 {
        const SearchTitle      = 1 << 0;
        const SearchProject    = 1 << 1;
        const SearchDetail     = 1 << 2;
        const EnableRegex      = 1 << 3;
    }
}

impl Default for DailyTaskSearchFlags {
    fn default() -> Self {
        Self::SearchTitle | Self::SearchProject | Self::SearchDetail
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DailyTaskOrderFlags: u32 {
        const OrderByStartDate = 1 << 1;
        const OrderByDueDate   = 1 << 2;
        const OrderByPriority  = 1 << 5;
        const Reversed         = 1 << 8;
    }
}

impl Default for DailyTaskOrderFlags {
    fn default() -> Self {
        Self::OrderByPriority
    }
}

pub trait DailyTaskRecord {
    type AsyncOutput;
    fn get_next_daily_task_id(&self) -> Result<i32>;
    fn fetch_one_daily_task(&self, id: i32) -> Self::AsyncOutput;
    fn fetch_all_daily_task(
        &self,
        filter_flags: DailyTaskFilterFlags,
        order_flags: DailyTaskOrderFlags,
    ) -> Self::AsyncOutput;
    fn search_daily_task(
        &self,
        pattern: &str,
        search_flags: DailyTaskSearchFlags,
        filter_flags: DailyTaskFilterFlags,
        order_flags: DailyTaskOrderFlags,
    ) -> Self::AsyncOutput;
    fn insert_daily_task(&self, task: &DailyTask) -> Self::AsyncOutput;
    fn update_daily_task(&self, task: &DailyTask) -> Self::AsyncOutput;
    fn upsert_daily_task(&self, task: &DailyTask) -> Self::AsyncOutput;
    fn delete_daily_task(&self, id: i32) -> Self::AsyncOutput;
    fn sync_all_daily_task(&self) -> Self::AsyncOutput;
}
