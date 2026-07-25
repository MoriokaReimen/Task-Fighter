use crate::weekly_task::WeeklyTask;
use bitflags::bitflags;
use uuid::Uuid;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct WeeklyTaskFilterFlags: u32 {
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

impl Default for WeeklyTaskFilterFlags {
    fn default() -> Self {
        Self::All
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
        Self::SearchTitle | Self::SearchProject | Self::SearchDetail
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
        Self::OrderByPriority
    }
}

pub trait WeeklyTaskRecord {
    type AsyncOutput;
    fn fetch_one_weekly_task(&self, uuid: Uuid) -> Self::AsyncOutput;
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
    fn delete_weekly_task(&self, uuid: Uuid) -> Self::AsyncOutput;
    fn sync_all_weekly_task(&self) -> Self::AsyncOutput;
}
