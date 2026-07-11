use super::super::{TaskPriority, WeeklyTask};
use super::constants::{PRIORITIES, PROJECTS, TASK_DETAILS, TITLES};
use anyhow::Result;
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

pub fn generate_weekly_task_sequence() -> Result<Vec<WeeklyTask>> {
    (0..=2)
        .enumerate()
        .map(|(idx, priority_num)| {
            // countは1から始まる（0番目の要素にアクセスしない仕様を維持）
            let count = (idx + 1) as i32;
            let count_usize = count as usize;

            let priority = TaskPriority::try_from(priority_num)?;
            let project = format!("Project: {}", PROJECTS[count_usize]);
            let title = format!("Title: {}", TITLES[count_usize]);

            // 修正：クロージャ全体から Result を返すため、Ok() で包む
            Ok(WeeklyTask {
                id: count,
                active: count % 2 == 0,
                project,
                title,
                detail: TASK_DETAILS[count_usize].to_string(),
                priority,
                start_day: WEEK_DAYS[(count % 6) as usize],
                due_day: WEEK_DAYS[((count % 6) + 1) as usize],
            })
        })
        .collect()
}

#[must_use]
pub fn get_random_weekly_task() -> WeeklyTask {
    let project_idx = rand::random_range(0..PROJECTS.len());
    let title_idx = rand::random_range(0..TITLES.len());
    let detail_idx = rand::random_range(0..TASK_DETAILS.len());
    let priority_idx = rand::random_range(0..PRIORITIES.len());

    let start_day_index = rand::random_range(0..WEEK_DAYS.len());

    // 修正：空の範囲（a..a）によるパニックを完全に防ぐため、包含的範囲「..=」を使用
    // これにより start_day_index == 6 の場合でも 6..=6 となり安全に日曜日が選ばれます
    let end_day_index = rand::random_range(start_day_index..=WEEK_DAYS.len() - 1);

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

pub fn generate_random_weekly_task(count: i32) -> Result<Vec<WeeklyTask>> {
    // 修正：get_random_weekly_task は Result ではなく生値を返すため、
    // map 内で Ok() に包んでから collect を通すことで Result<Vec<WeeklyTask>> を満たします
    Ok((0..count.max(0))
        .map(|_| get_random_weekly_task())
        .collect())
}
