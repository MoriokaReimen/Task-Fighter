use super::super::{Task, TaskPriority, TaskStatus};
use super::constants::{PROJECTS, TITLES, TASK_DETAILS, PRIORITIES};
use jiff::ToSpan;
use jiff::civil::Date;

const STATUSES: [TaskStatus; 4] = [
    TaskStatus::Pending,
    TaskStatus::WorkInProgress,
    TaskStatus::Complete,
    TaskStatus::Canceled,
];

#[must_use]
pub fn generate_task_sequence() -> Vec<Task> {
    let base_date = Date::new(1970, 1, 1).unwrap();
    let base_end_date = Date::new(1970, 1, 10).unwrap();

    (0..=2)
        .flat_map(|p| (0..=3).map(move |s| (p, s)))
        .enumerate()
        .map(|(count, (priority_num, status_num))| {
            let count_i32 = count as i32;
            let priority = TaskPriority::try_from(priority_num).unwrap();
            let status = TaskStatus::try_from(status_num).unwrap();

            let project = format!("Project: {}", PROJECTS[priority_num as usize]);
            let title = format!("Title: {}", TITLES[status_num as usize]);

            let end_date = if count < 6 {
                None
            } else {
                Some(base_end_date + count_i32.days())
            };

            Task {
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
            }
        })
        .collect()
}

#[must_use]
pub fn get_random_task() -> Task {
    let project_idx = rand::random_range(0..PROJECTS.len());
    let title_idx = rand::random_range(0..TITLES.len());
    let detail_idx = rand::random_range(0..TASK_DETAILS.len());
    let status_idx = rand::random_range(0..STATUSES.len());
    let priority_idx = rand::random_range(0..PRIORITIES.len());

    let start_day = rand::random_range(1..=28);
    let start_month = rand::random_range(1..=5);
    let duration = rand::random_range(1..=14);

    let start_date = Date::new(2026, start_month, start_day).unwrap();
    let due_date = start_date.checked_add(duration.days()).unwrap();

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

    let end_date = match status {
        TaskStatus::Complete | TaskStatus::Canceled => {
            let days_to_complete = rand::random_range(1..=(duration + 2));
            Some(start_date.checked_add(days_to_complete.days()).unwrap())
        }
        _ => None,
    };

    Task {
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
    }
}

#[must_use]
pub fn generate_random_tasks(count: i32) -> Vec<Task> {
    (0..count.max(0)).map(|_| get_random_task()).collect()
}
