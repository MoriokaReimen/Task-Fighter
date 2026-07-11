use super::super::{DailyTask, TaskPriority};
use super::constants::{PRIORITIES, PROJECTS, TASK_DETAILS, TITLES};
use anyhow::Result;

pub fn generate_daily_task_sequence() -> Result<Vec<DailyTask>> {
    (0..=2)
        .enumerate()
        .map(|(idx, priority_num)| {
            let count = (idx + 1) as i32;
            let count_usize = count as usize;

            // 修正：unwrap() を ? 演算子に変更
            let priority = TaskPriority::try_from(priority_num)?;
            let project = format!("Project: {}", PROJECTS[count_usize]);
            let title = format!("Title: {}", TITLES[count_usize]);

            // 修正：Ok() で包んで Result を返す
            Ok(DailyTask {
                id: count,
                active: count % 2 == 0,
                project,
                title,
                detail: TASK_DETAILS[count_usize].to_string(),
                priority,
            })
        })
        .collect()
}

#[must_use]
pub fn get_random_daily_task() -> DailyTask {
    let project_idx = rand::random_range(0..PROJECTS.len());
    let title_idx = rand::random_range(0..TITLES.len());
    let detail_idx = rand::random_range(0..TASK_DETAILS.len());
    let priority_idx = rand::random_range(0..PRIORITIES.len());

    DailyTask {
        id: 0,
        project: PROJECTS[project_idx].to_string(),
        title: TITLES[title_idx].to_string(),
        detail: TASK_DETAILS[detail_idx].to_string(),
        priority: PRIORITIES[priority_idx],
        active: rand::random(),
    }
}

pub fn generate_random_daily_task(count: i32) -> Result<Vec<DailyTask>> {
    // 修正：他のランダム生成関数とインターフェースを合わせ、Result<Vec<DailyTask>> を返す形に
    Ok((0..count.max(0)).map(|_| get_random_daily_task()).collect())
}
