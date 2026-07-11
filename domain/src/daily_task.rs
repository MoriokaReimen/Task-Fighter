use crate::task::{Task, TaskPriority};
use anyhow::{Result, bail};
use jiff::civil::Date;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct DailyTask {
    pub id: i32,
    pub active: bool,
    pub project: String,
    pub title: String,
    pub detail: String,
    pub priority: TaskPriority,
}

impl Default for DailyTask {
    fn default() -> Self {
        Self {
            id: 0,
            active: true,
            project: String::new(),
            title: String::new(),
            detail: String::new(),
            priority: TaskPriority::Low,
        }
    }
}

impl DailyTask {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.project.trim().is_empty() && !self.title.trim().is_empty()
    }

    pub fn create_task(&self, today: &Date) -> Result<Task> {
        if !self.is_valid() {
            bail!("Invalid daily task.");
        }

        let title_with_date = format!("{} for {}", self.title, today.strftime("%Y/%m/%d"));

        let task_target_date = *today;

        Ok(Task {
            id: self.id,
            project: self.project.clone(),
            title: title_with_date,
            detail: self.detail.clone(),
            priority: self.priority,
            start_date: task_target_date,
            due_date: task_target_date,
            entry_date: task_target_date,
            ..Task::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::Date;

    // 共通で使える有効なDailyTaskのヘルパー
    fn valid_daily_task() -> DailyTask {
        DailyTask {
            id: 42,
            active: true,
            project: "Routine".to_string(),
            title: "Daily Review".to_string(),
            detail: "Check Slack and emails".to_string(),
            priority: TaskPriority::High,
        }
    }

    #[test]
    fn test_default_impl() {
        let default_task = DailyTask::default();
        assert_eq!(default_task.id, 0);
        assert!(default_task.active);
        assert!(default_task.project.is_empty());
        assert!(default_task.title.is_empty());
        assert!(default_task.detail.is_empty());
        assert_eq!(default_task.priority, TaskPriority::Low);
    }

    #[test]
    fn test_is_valid_with_valid_data() {
        let task = valid_daily_task();
        assert!(task.is_valid());
    }

    #[test]
    fn test_is_valid_with_invalid_data() {
        // プロジェクト名が空
        let mut task = valid_daily_task();
        task.project = String::new();
        assert!(!task.is_valid());

        // タイトルがスペースのみ
        let mut task = valid_daily_task();
        task.title = "   ".to_string();
        assert!(!task.is_valid());
    }

    #[test]
    fn test_create_task_success() {
        let daily_task = valid_daily_task();
        let today = Date::new(2026, 7, 7).unwrap();

        let result = daily_task.create_task(&today);
        assert!(result.is_ok());

        let created_task = result.unwrap();

        // 期待通りのマッピングがされているか検証
        assert_eq!(created_task.id, daily_task.id);
        assert_eq!(created_task.project, daily_task.project);
        assert_eq!(created_task.detail, daily_task.detail);
        assert_eq!(created_task.priority, daily_task.priority);

        // タイトルの日付フォーマットが正しいか検証
        assert_eq!(created_task.title, "Daily Review for 2026/07/07");

        // 各種日付が正しく設定されているか検証
        assert_eq!(created_task.start_date, today);
        assert_eq!(created_task.due_date, today);
        assert_eq!(created_task.entry_date, today);
    }

    #[test]
    fn test_create_task_invalid_failure() {
        let mut invalid_task = valid_daily_task();
        invalid_task.title = String::new(); // 不正な状態にする
        let today = Date::new(2026, 7, 7).unwrap();

        let result = invalid_task.create_task(&today);

        // エラーになることを検証
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid daily task.");
    }
}
