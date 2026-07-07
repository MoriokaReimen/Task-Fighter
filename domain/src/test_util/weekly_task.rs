use super::super::{TaskPriority, WeeklyTask};
use super::constants::*;
use jiff::civil::Weekday;

const WEEK_DAYS: [Weekday; 7] = [
    Weekday::Monday,
    Weekday::Tuesday,
    Weekday::Wednesday,
    Weekday::Thursday,
    Weekday::Friday,
    Weekday::Saturday,
    Weekday::Sunday,
];

/// 優先度に基づいた週次タスクシーケンスを生成する
pub fn generate_weekly_task_sequence() -> Vec<WeeklyTask> {
    (0..=2)
        .enumerate()
        .map(|(idx, priority_num)| {
            // countは1から始まる（0番目の要素にアクセスしない仕様を維持）
            let count = (idx + 1) as i32;
            let count_usize = count as usize;

            let priority = TaskPriority::try_from(priority_num).unwrap();
            let project = format!("Project: {}", PROJECTS[count_usize]);
            let title = format!("Title: {}", TITLES[count_usize]);

            WeeklyTask {
                id: count,
                active: count % 2 == 0,
                project,
                title,
                detail: TASK_DETAILS[count_usize].to_string(),
                priority,
                start_day: WEEK_DAYS[(count % 6) as usize],
                due_day: WEEK_DAYS[((count % 6) + 1) as usize],
            }
        })
        .collect()
}

/// ランダムな属性を持つ単一の週次タスクを生成する
pub fn get_random_weekly_task() -> WeeklyTask {
    let project_idx = rand::random_range(0..PROJECTS.len());
    // バグ修正: TITLESの取得にPROJECTS.len()とPROJECTS配列を使っていたのを修正
    let title_idx = rand::random_range(0..TITLES.len());
    let detail_idx = rand::random_range(0..TASK_DETAILS.len());
    let priority_idx = rand::random_range(0..PRIORITIES.len());

    let start_day_index = rand::random_range(0..WEEK_DAYS.len());
    // バグ修正: 範囲が空（0..0）になる可能性を排除し、安全に終了曜日を決定
    let end_day_index = rand::random_range(start_day_index..WEEK_DAYS.len());

    WeeklyTask {
        id: 0,
        project: PROJECTS[project_idx].to_string(),
        title: TITLES[title_idx].to_string(),
        detail: TASK_DETAILS[detail_idx].to_string(),
        priority: PRIORITIES[priority_idx],
        start_day: WEEK_DAYS[start_day_index],
        due_day: WEEK_DAYS[end_day_index],
        active: rand::random(),
    }
}

/// 指定された数のランダムな週次タスクを含むリストを生成する
pub fn generate_random_weekly_task(count: i32) -> Vec<WeeklyTask> {
    (0..count.max(0))
        .map(|_| get_random_weekly_task())
        .collect()
}
