use super::super::{MonthlyTask, TaskPriority};
use super::constants::*;

pub fn generate_monthly_task_sequence() -> Vec<MonthlyTask> {
    (0..=2)
        .enumerate()
        .map(|(idx, priority_num)| {
            // countは1から始まる仕様を維持
            let count = (idx + 1) as i32;
            let count_usize = count as usize;
            
            let priority = TaskPriority::try_from(priority_num).unwrap();
            let project = format!("Project: {}", PROJECTS[count_usize]);
            let title = format!("Title: {}", TITLES[count_usize]);

            MonthlyTask {
                id: count,
                active: count % 2 == 0,
                project,
                title,
                detail: TASK_DETAILS[count_usize].to_string(),
                priority,
                start_day: count,
                due_day: count + 5,
            }
        })
        .collect()
}

pub fn get_random_monthly_task() -> MonthlyTask {
    let project_idx = rand::random_range(0..PROJECTS.len());
    // バグ修正: TITLESの取得にPROJECTSの配列と長さを使っていたのを修正
    let title_idx = rand::random_range(0..TITLES.len());
    let detail_idx = rand::random_range(0..TASK_DETAILS.len());
    let priority_idx = rand::random_range(0..PRIORITIES.len());
    
    let start_day = rand::random_range(1..=31);
    let due_day = rand::random_range(start_day..=31);

    MonthlyTask {
        id: 0,
        project: PROJECTS[project_idx].to_string(),
        title: TITLES[title_idx].to_string(),
        detail: TASK_DETAILS[detail_idx].to_string(),
        priority: PRIORITIES[priority_idx],
        start_day,
        due_day,
        active: rand::random(),
    }
}

pub fn generate_random_monthly_task(count: i32) -> Vec<MonthlyTask> {
    (0..count.max(0))
        .map(|_| get_random_monthly_task())
        .collect()
}
