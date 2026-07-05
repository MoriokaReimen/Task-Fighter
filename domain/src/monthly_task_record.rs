use crate::monthly_task::MonthlyTask;
use anyhow::Result;
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MonthlyTaskFilterFlags: u32 {
        const PriorityLow      = 1 << 2;
        const PriorityMiddle   = 1 << 3;
        const PriorityHigh     = 1 << 4;
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

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MonthlyTaskOrderFlags: u32 {
        const OrderByStartDate = 1 << 1;
        const OrderByDueDate   = 1 << 2;
        const OrderByPriority  = 1 << 5;
        const Reversed         = 1 << 8;
    }
}

pub trait MonthlyTaskRecord {
    type AsyncOutput;
    fn get_next_id(&self) -> Result<i32>;
    fn fetch_one(&self, id: i32) -> Self::AsyncOutput;
    fn fetch_all(
        &self,
        filter_flags: MonthlyTaskFilterFlags,
        order_flags: MonthlyTaskOrderFlags,
    ) -> Self::AsyncOutput;
    fn search(
        &self,
        pattern: &str,
        search_flags: MonthlyTaskSearchFlags,
        filter_flags: MonthlyTaskFilterFlags,
        order_flags: MonthlyTaskOrderFlags,
    ) -> Self::AsyncOutput;
    fn insert(&self, task: &MonthlyTask) -> Self::AsyncOutput;
    fn update(&self, task: &MonthlyTask) -> Self::AsyncOutput;
    fn upsert(&self, task: &MonthlyTask) -> Self::AsyncOutput;
    fn delete(&self, task: &MonthlyTask) -> Self::AsyncOutput;
}
