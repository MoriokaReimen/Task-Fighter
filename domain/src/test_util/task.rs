use super::super::{Task, TaskPriority, TaskStatus};
use super::constants::*;
use jiff::ToSpan;
use jiff::civil::Date;

const STATUSES: [TaskStatus; 4] = [
    TaskStatus::Pending,
    TaskStatus::WorkInProgress,
    TaskStatus::Complete,
    TaskStatus::Canceled,
];
const PRIORITIES: [TaskPriority; 3] = [TaskPriority::Low, TaskPriority::Medium, TaskPriority::High];

pub fn generate_task_sequence() -> Vec<Task> {
    let mut ret: Vec<Task> = Vec::new();
    let mut count: i32 = 0;
    for (priority_num, status_num) in (0..=2).flat_map(|p| (0..=3).map(move |s| (p, s))) {
        let priority = TaskPriority::try_from(priority_num).unwrap();
        let status = TaskStatus::try_from(status_num).unwrap();
        let project = format!("Project: {}", PROJECTS[priority_num as usize]);
        let title = format!("Title: {}", TITLES[status_num as usize]);
        let mut task = Task::default();
        task.id = count as i32;
        task.active = count % 2 == 0;
        task.status = status;
        task.project = project;
        task.title = title;
        task.detail = TASK_DETAILS[count as usize].to_string();
        task.start_date = Date::new(1970, 1, 1).unwrap();
        task.due_date = Date::new(1970, 1, 1).unwrap() + count.days();
        task.priority = priority;
        task.progress = 8.33 * count as f32;
        task.time_spent = 10.0 * count as f32;
        task.entry_date = Date::new(1970, 1, 1).unwrap();
        task.end_date = if count < 6 {
            None
        } else {
            Some(Date::new(1970, 1, 10).unwrap() + count.days())
        };
        ret.push(task);
        count += 1;
    }

    ret
}

pub fn get_random_task() -> Task {
    let mut task = Task::default();

    let project_idx = rand::random_range(0..PROJECTS.len());
    task.project = PROJECTS[project_idx].to_string();

    let title_idx = rand::random_range(0..PROJECTS.len());
    task.title = format!("{}", PROJECTS[title_idx]);

    let detail_idx = rand::random_range(0..TASK_DETAILS.len());
    task.detail = TASK_DETAILS[detail_idx].to_string();

    let status_idx = rand::random_range(0..STATUSES.len());
    task.status = STATUSES[status_idx];

    let priority_idx = rand::random_range(0..PRIORITIES.len());
    task.priority = PRIORITIES[priority_idx];

    let start_day = rand::random_range(1..=28);
    let start_month = rand::random_range(1..=5);
    task.start_date = Date::new(2026, start_month, start_day).unwrap();

    let duration = rand::random_range(1..=14);
    task.due_date = task
        .start_date
        .checked_add(jiff::ToSpan::days(duration))
        .unwrap();

    task.entry_date = task.start_date;

    task.progress = match task.status {
        TaskStatus::Pending => 0.0,
        TaskStatus::Complete => 100.0,
        TaskStatus::Canceled => rand::random_range(0.0..50.0),
        TaskStatus::WorkInProgress => rand::random_range(5.0..95.0),
    };
    task.time_spent = if task.progress > 0.0 {
        rand::random_range(0.5..24.0)
    } else {
        0.0
    };

    task.end_date = if task.status == TaskStatus::Complete || task.status == TaskStatus::Canceled {
        let days_to_complete = rand::random_range(1..=(duration + 2));
        let end_date = task
            .start_date
            .checked_add(jiff::ToSpan::days(days_to_complete))
            .unwrap();
        Some(end_date)
    } else {
        None
    };

    task.active = rand::random();

    task
}

pub fn generate_random_tasks(count: i32) -> Vec<Task> {
    let mut ret: Vec<Task> = Vec::new();
    for _ in [0..count] {
        let task = get_random_task();
        ret.push(task);
    }

    ret
}
