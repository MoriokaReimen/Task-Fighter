use crate::task::{Task, TaskPriority};
use anyhow::{Result, bail};
use jiff::civil::{Date, Weekday};

pub struct WeeklyTask {
    pub id: i32,
    pub active: bool,
    pub project: String,
    pub title: String,
    pub detail: String,
    pub priority: TaskPriority,
    pub start_day: Weekday,
    pub end_day: Weekday,
}

impl Default for WeeklyTask {
    fn default() -> Self {
        Self {
            id: 0,
            active: true,
            project: String::new(),
            title: String::new(),
            detail: String::new(),
            priority: TaskPriority::Low,
            start_day: Weekday::Monday,
            end_day: Weekday::Friday,
        }
    }
}

impl WeeklyTask {
    pub fn is_valid(&self) -> bool {
        !self.project.trim().is_empty()
            && !self.title.trim().is_empty()
            && (self.start_day as u8) <= (self.end_day as u8)
    }

    pub fn create_task(&self, today: &Date) -> Result<Task> {
        if !self.is_valid() {
            bail!("Invalid weekly task");
        }

        // 💡 改善：計算の基準となる「起点日」であることが明確な名前に変更
        let calculation_base_date = today.tomorrow()?;

        // 💡 改善：単なる start_date ではなく「タスクの開始日/締切日」であることがわかる名前に変更
        let task_start_date = calculation_base_date.nth_weekday(-1, self.start_day)?;
        let task_due_date = calculation_base_date.nth_weekday(1, self.end_day)?;

        // 💡 改善：何月度なのかを表すフォーマット用であることを明示
        let formatted_month = task_start_date.strftime("%B");
        let week_of_month = ((task_start_date.day() - 1) / 7) + 1;

        let title_with_week = format!(
            "{} for {} Week {}",
            self.title, formatted_month, week_of_month
        );

        Ok(Task {
            id: self.id,
            project: self.project.clone(),
            title: title_with_week,
            detail: self.detail.clone(),
            priority: self.priority,
            start_date: task_start_date,
            due_date: task_due_date,
            entry_date: *today,
            ..Task::default()
        })
    }
}
