use super::super::{Task, TaskPriority, TaskStatus};
use super::constants::{PRIORITIES, PROJECTS, TASK_DETAILS, TITLES};
use anyhow::Result;
use jiff::ToSpan;
use jiff::civil::Date;

const STATUSES: [TaskStatus; 4] = [
    TaskStatus::Pending,
    TaskStatus::WorkInProgress,
    TaskStatus::Complete,
    TaskStatus::Canceled,
];

pub fn generate_task_sequence() -> Result<Vec<Task>> {
    let base_date = Date::new(1970, 1, 1)?;
    let base_end_date = Date::new(1970, 1, 10)?;

    (0..=2)
        .flat_map(|p| (0..=3).map(move |s| (p, s)))
        .enumerate()
        .map(|(count, (priority_num, status_num))| {
            let count_i32 = count as i32;
            let priority = TaskPriority::try_from(priority_num)?;
            let status = TaskStatus::try_from(status_num)?;

            let project = format!("Project: {}", PROJECTS[priority_num as usize]);
            let title = format!("Title: {}", TITLES[status_num as usize]);

            let end_date = if count < 6 {
                None
            } else {
                Some(base_end_date + count_i32.days())
            };

            // ここは map の中なので、全体を Ok() で包んで Result を返し、
            // 最後に一括で .collect() します
            Ok(Task {
                id: count_i32,
                active: count % 2 == 0,
                status,
                project,
                title,
                detail: TASK_DETAILS[count].to_string(),
                start_date: base_date,
                due_date: base_date + count_i32.days(),
                priority,
                progress: 8.33 * count_i32 as f32,
                time_spent: 10.0 * count_i32 as f32,
                entry_date: base_date,
                end_date,
            })
        })
        .collect()
}

pub fn get_random_task() -> Result<Task> {
    let project_idx = rand::random_range(0..PROJECTS.len());
    let title_idx = rand::random_range(0..TITLES.len());
    let detail_idx = rand::random_range(0..TASK_DETAILS.len());
    let status_idx = rand::random_range(0..STATUSES.len());
    let priority_idx = rand::random_range(0..PRIORITIES.len());

    let start_day: i8 = rand::random_range(1..=28);
    let start_month: i8 = rand::random_range(1..=5);
    let duration: i32 = rand::random_range(1..=14);

    let start_date = Date::new(2026, start_month, start_day)?;
    let due_date = start_date.checked_add(duration.days())?;

    let status = STATUSES[status_idx];

    let progress = match status {
        TaskStatus::Pending => 0.0,
        TaskStatus::Complete => 100.0,
        TaskStatus::Canceled => rand::random_range(0.0..50.0),
        TaskStatus::WorkInProgress => rand::random_range(5.0..95.0),
    };

    let time_spent = if progress > 0.0 {
        rand::random_range(0.5..24.0)
    } else {
        0.0
    };

    // 修正：match 構文の閉じカッコ「};」と、None ケースを正しく追加
    let end_date = match status {
        TaskStatus::Complete | TaskStatus::Canceled => {
            let days_to_complete: i32 = rand::random_range(1..=(duration + 2));
            Some(start_date.checked_add(days_to_complete.days())?)
        }
        _ => None,
    };

    Ok(Task {
        id: 0,
        project: PROJECTS[project_idx].to_string(),
        title: TITLES[title_idx].to_string(),
        detail: TASK_DETAILS[detail_idx].to_string(),
        status,
        priority: PRIORITIES[priority_idx],
        start_date,
        due_date,
        entry_date: start_date,
        progress,
        time_spent,
        end_date,
        active: rand::random(),
    })
}

pub fn generate_random_tasks(count: i32) -> Result<Vec<Task>> {
    (0..count.max(0)).map(|_| get_random_task()).collect()
}
