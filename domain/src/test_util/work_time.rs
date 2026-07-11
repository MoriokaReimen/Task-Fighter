use super::super::WorkTime;
use anyhow::Result;
use jiff::civil::Date;

pub fn get_random_work_time(task_id: i32) -> Result<WorkTime> {
    let random_day = rand::random_range(1..=28);
    let random_month = rand::random_range(1..=12);
    let time_spent = rand::random_range(0.5..24.0);
    let date = Date::new(2026, random_month, random_day)?;
    Ok(WorkTime {
        id: 0,
        task_id,
        date,
        time_spent,
    })
}

pub fn generate_random_work_time(task_id: i32, count: i32) -> Result<Vec<WorkTime>> {
    (0..count.max(0))
        .map(|_| get_random_work_time(task_id))
        .collect::<Result<_, _>>()
}
