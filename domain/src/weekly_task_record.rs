use crate::weekly_task::WeeklyTask;
use anyhow::Result;
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct WeeklyTaskFilterFlags: u32 {
        const Active           = 1 << 0;
        const Inactive         = 1 << 1;
        const PriorityLow      = 1 << 2;
        const PriorityMiddle   = 1 << 3;
        const PriorityHigh     = 1 << 4;
    }
}

impl Default for WeeklyTaskFilterFlags {
    fn default() -> Self {
        WeeklyTaskFilterFlags::Active
            | WeeklyTaskFilterFlags::Inactive
            | WeeklyTaskFilterFlags::PriorityLow
            | WeeklyTaskFilterFlags::PriorityMiddle
            | WeeklyTaskFilterFlags::PriorityHigh
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct WeeklyTaskSearchFlags: u32 {
        const SearchTitle      = 1 << 0;
        const SearchProject    = 1 << 1;
        const SearchDetail     = 1 << 2;
        const EnableRegex      = 1 << 3;
    }
}

impl Default for WeeklyTaskSearchFlags {
    fn default() -> Self {
        WeeklyTaskSearchFlags::SearchTitle
            | WeeklyTaskSearchFlags::SearchProject
            | WeeklyTaskSearchFlags::SearchDetail
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct WeeklyTaskOrderFlags: u32 {
        const OrderByStartDay  = 1 << 1;
        const OrderByDueDay    = 1 << 2;
        const OrderByPriority  = 1 << 5;
        const Reversed         = 1 << 8;
    }
}

impl Default for WeeklyTaskOrderFlags {
    fn default() -> Self {
        WeeklyTaskOrderFlags::OrderByPriority
    }
}

pub trait WeeklyTaskRecord {
    type AsyncOutput;
    fn get_next_weekly_task_id(&self) -> Result<i32>;
    fn fetch_one_weekly_task(&self, id: i32) -> Self::AsyncOutput;
    fn fetch_all_weekly_task(
        &self,
        filter_flags: WeeklyTaskFilterFlags,
        order_flags: WeeklyTaskOrderFlags,
    ) -> Self::AsyncOutput;
    fn search_weekly_task(
        &self,
        pattern: &str,
        search_flags: WeeklyTaskSearchFlags,
        filter_flags: WeeklyTaskFilterFlags,
        order_flags: WeeklyTaskOrderFlags,
    ) -> Self::AsyncOutput;
    fn insert_weekly_task(&self, task: &WeeklyTask) -> Self::AsyncOutput;
    fn update_weekly_task(&self, task: &WeeklyTask) -> Self::AsyncOutput;
    fn upsert_weekly_task(&self, task: &WeeklyTask) -> Self::AsyncOutput;
    fn delete_weekly_task(&self, id: i32) -> Self::AsyncOutput;
    fn sync_all_weekly_task(&self) -> Self::AsyncOutput;
}
