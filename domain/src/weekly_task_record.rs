use crate::weekly_task::WeeklyTask;
use anyhow::Result;
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct WeeklyTaskFilterFlags: u32 {
        const PriorityLow      = 1 << 2;
        const PriorityMiddle   = 1 << 3;
        const PriorityHigh     = 1 << 4;
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

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct WeeklyTaskOrderFlags: u32 {
        const OrderByStartDate = 1 << 1;
        const OrderByDueDate   = 1 << 2;
        const OrderByPriority  = 1 << 5;
        const Reversed         = 1 << 8;
    }
}

pub trait WeeklyTaskRecord {
    type AsyncOutput;
    fn get_next_id(&self) -> Result<i32>;
    fn fetch_one(&self, id: i32) -> Self::AsyncOutput;
    fn fetch_all(
        &self,
        filter_flags: WeeklyTaskFilterFlags,
        order_flags: WeeklyTaskOrderFlags,
    ) -> Self::AsyncOutput;
    fn search(
        &self,
        pattern: &str,
        search_flags: WeeklyTaskSearchFlags,
        filter_flags: WeeklyTaskFilterFlags,
        order_flags: WeeklyTaskOrderFlags,
    ) -> Self::AsyncOutput;
    fn insert(&self, task: &WeeklyTask) -> Self::AsyncOutput;
    fn update(&self, task: &WeeklyTask) -> Self::AsyncOutput;
    fn upsert(&self, task: &WeeklyTask) -> Self::AsyncOutput;
    fn delete(&self, task: &WeeklyTask) -> Self::AsyncOutput;
}
