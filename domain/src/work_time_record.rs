use crate::work_time::WorkTime;
use anyhow::Result;
use jiff::civil::Date;

pub trait WorkTimeRecord {
    type AsyncOutput;

    fn next_work_time_id(&self) -> Result<i32>;
    fn find_work_time_by_date(&self, task_id: i32, date: &Date) -> Self::AsyncOutput;
    fn list_work_time_by_task(&self, task_id: i32) -> Self::AsyncOutput;
    fn insert_work_time(&self, work_time: &WorkTime) -> Self::AsyncOutput;
    fn update_work_time(&self, work_time: &WorkTime) -> Self::AsyncOutput;
    fn upsert_work_time(&self, work_time: &WorkTime) -> Self::AsyncOutput;
    fn get_total_work_time_by_task(&self, task_id: i32) -> Self::AsyncOutput;
    fn get_total_work_time_by_date(&self, date: &Date) -> Self::AsyncOutput;
    fn get_total_work_time_history(&self, start_date: &Date, end_date: &Date) -> Self::AsyncOutput;
    fn get_total_work_time_ratio(&self, start_date: &Date, end_date: &Date) -> Self::AsyncOutput;
}
