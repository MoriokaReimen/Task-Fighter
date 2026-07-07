use crate::task::{Task, TaskPriority};
use anyhow::{Result, bail};
use jiff::civil::Date;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct MonthlyTask {
    pub id: i32,
    pub active: bool,
    pub project: String,
    pub title: String,
    pub detail: String,
    pub priority: TaskPriority,
    pub start_day: i16,
    pub due_day: i16,
}

impl Default for MonthlyTask {
    fn default() -> Self {
        Self {
            id: 0,
            active: true,
            project: String::new(),
            title: String::new(),
            detail: String::new(),
            priority: TaskPriority::Low,
            start_day: 1,
            due_day: 1,
        }
    }
}

impl MonthlyTask {
    pub fn is_valid(&self) -> bool {
        !self.project.trim().is_empty()
            && !self.title.trim().is_empty()
            && (1..=31).contains(&self.start_day)
            && (1..=31).contains(&self.due_day)
            && self.start_day <= self.due_day
    }

    pub fn create_task(&self, today: &Date) -> Result<Task> {
        if !self.is_valid() {
            bail!("Invalid monthly task");
        }
        let max_days = today.days_in_month() as i16;
        let resolved_start_day = self.start_day.clamp(1, max_days) as i8;
        let resolved_due_day = self.due_day.clamp(1, max_days) as i8;
        let task_start_date = today.with().day(resolved_start_day).build()?;
        let task_due_date = today.with().day(resolved_due_day).build()?;

        let title_with_month = format!("{} for {}", self.title, task_start_date.strftime("%Y/%m"));

        Ok(Task {
            id: self.id,
            project: self.project.clone(),
            title: title_with_month,
            detail: self.detail.clone(),
            priority: self.priority,
            start_date: task_start_date,
            due_date: task_due_date,
            entry_date: *today,
            ..Task::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::Date;

    // 共通で使える有効なMonthlyTaskのヘルパー
    fn valid_monthly_task() -> MonthlyTask {
        MonthlyTask {
            id: 100,
            active: true,
            project: "Billing".to_string(),
            title: "Monthly Invoice".to_string(),
            detail: "Send invoice to clients".to_string(),
            priority: TaskPriority::High,
            start_day: 10,
            due_day: 15,
        }
    }

    #[test]
    fn test_default_impl() {
        let default_task = MonthlyTask::default();
        assert_eq!(default_task.id, 0);
        assert!(default_task.active);
        assert!(default_task.project.is_empty());
        assert!(default_task.title.is_empty());
        assert!(default_task.detail.is_empty());
        assert_eq!(default_task.priority, TaskPriority::Low);
        assert_eq!(default_task.start_day, 1);
        assert_eq!(default_task.due_day, 1);
    }

    #[test]
    fn test_is_valid_success() {
        let task = valid_monthly_task();
        assert!(task.is_valid());
    }

    #[test]
    fn test_is_valid_failures() {
        // 1. プロジェクト名が空
        let mut task = valid_monthly_task();
        task.project = "  ".to_string();
        assert!(!task.is_valid());

        // 2. タイトルが空
        let mut task = valid_monthly_task();
        task.title = "".to_string();
        assert!(!task.is_valid());

        // 3. start_day が範囲外 (0)
        let mut task = valid_monthly_task();
        task.start_day = 0;
        assert!(!task.is_valid());

        // 4. due_day が範囲外 (32)
        let mut task = valid_monthly_task();
        task.due_day = 32;
        assert!(!task.is_valid());

        // 5. start_day が due_day より後ろ
        let mut task = valid_monthly_task();
        task.start_day = 20;
        task.due_day = 10;
        assert!(!task.is_valid());
    }

    #[test]
    fn test_create_task_success() {
        let monthly_task = valid_monthly_task();
        // 2026年7月（31日まである月）
        let today = Date::new(2026, 7, 7).unwrap();

        let result = monthly_task.create_task(&today);
        assert!(result.is_ok());

        let created_task = result.unwrap();

        assert_eq!(created_task.id, monthly_task.id);
        assert_eq!(created_task.project, monthly_task.project);
        assert_eq!(created_task.detail, monthly_task.detail);
        assert_eq!(created_task.priority, monthly_task.priority);

        // タイトルの年月フォーマットの検証
        assert_eq!(created_task.title, "Monthly Invoice for 2026/07");

        // 日付オブジェクトが正しくマッピングされているか
        assert_eq!(created_task.start_date, Date::new(2026, 7, 10).unwrap());
        assert_eq!(created_task.due_date, Date::new(2026, 7, 15).unwrap());
        assert_eq!(created_task.entry_date, today);
    }

    #[test]
    fn test_create_task_clamp_at_month_end() {
        // 31日として設定されたタスク
        let mut monthly_task = valid_monthly_task();
        monthly_task.start_day = 30;
        monthly_task.due_day = 31;

        // うるう年ではない2026年の2月（28日まで）を指定
        let today = Date::new(2026, 2, 15).unwrap();

        let result = monthly_task.create_task(&today);
        assert!(result.is_ok());

        let created_task = result.unwrap();

        // 28日にクランプ（丸め込み）されていることを検証
        assert_eq!(created_task.start_date, Date::new(2026, 2, 28).unwrap());
        assert_eq!(created_task.due_date, Date::new(2026, 2, 28).unwrap());
        assert_eq!(created_task.title, "Monthly Invoice for 2026/02");
    }

    #[test]
    fn test_create_task_invalid_failure() {
        let mut invalid_task = valid_monthly_task();
        invalid_task.start_day = 31;
        invalid_task.due_day = 1; // バリデーションエラーになる設定

        let today = Date::new(2026, 7, 7).unwrap();
        let result = invalid_task.create_task(&today);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid monthly task");
    }
}
