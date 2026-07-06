use super::super::{DailyTask, Task, TaskPriority, TaskStatus};
use super::constants::*;
use jiff::ToSpan;
use jiff::civil::Date;

const PRIORITIES: [TaskPriority; 3] = [TaskPriority::Low, TaskPriority::Medium, TaskPriority::High];

pub fn generate_daily_task_sequence() -> Vec<DailyTask> {
    let mut ret: Vec<DailyTask> = Vec::new();

    let mut count: i32 = 1;
    for priority_num in 0..=2 {
        let priority = TaskPriority::try_from(priority_num).unwrap();
        let project = format!("Project: {}", PROJECTS[count as usize]);
        let title = format!("Title: {}", TITLES[count as usize]);
        let mut daily_task = DailyTask::default();
        daily_task.id = count as i32;
        daily_task.active = count % 2 == 0;
        daily_task.project = project;
        daily_task.title = title;
        daily_task.detail = TASK_DETAILS[count as usize].to_string();
        daily_task.priority = priority;
        ret.push(daily_task);
        count += 1;
    }
    ret
}

pub fn get_random_daily_task() -> DailyTask {
    let mut daily_task = DailyTask::default();

    let project_idx = rand::random_range(0..PROJECTS.len());
    daily_task.project = PROJECTS[project_idx].to_string();

    let title_idx = rand::random_range(0..PROJECTS.len());
    daily_task.title = format!("{}", PROJECTS[title_idx]);

    let detail_idx = rand::random_range(0..TASK_DETAILS.len());
    daily_task.detail = TASK_DETAILS[detail_idx].to_string();

    let priority_idx = rand::random_range(0..PRIORITIES.len());
    daily_task.priority = PRIORITIES[priority_idx];
    daily_task.active = rand::random();
    daily_task
}

pub fn generate_random_daily_task(count: i32) -> Vec<DailyTask> {
    let mut ret: Vec<DailyTask> = Vec::new();
    for _ in [0..count] {
        let mut daily_task = get_random_daily_task();
        ret.push(daily_task);
    }

    ret
}
