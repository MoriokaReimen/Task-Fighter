use anyhow::{Result, bail};
use jiff::Zoned;
use jiff::civil::Date;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub enum Priority {
    #[default]
    Low = 0,
    Medium = 1,
    High = 2,
}

impl TryFrom<i32> for Priority {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Priority::Low),
            1 => Ok(Priority::Medium),
            2 => Ok(Priority::High),
            _ => bail!("Invalid priority integer state: {}", value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TaskStatus {
    #[default]
    Pending = 0,
    WorkInProgress = 1,
    Complete = 2,
    Canceled = 3,
}

impl TryFrom<i32> for TaskStatus {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TaskStatus::Pending),
            1 => Ok(TaskStatus::WorkInProgress),
            2 => Ok(TaskStatus::Complete),
            3 => Ok(TaskStatus::Canceled),
            _ => bail!("Invalid task status integer state: {}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: i32,
    pub active: bool,
    pub status: TaskStatus,
    pub project: String,
    pub title: String,
    pub detail: String,
    pub start_date: Date,
    pub due_date: Date,
    pub priority: Priority,
    pub progress: f32,
    pub time_spent: f32,
    pub entry_date: Date,
    pub end_date: Option<Date>,
}

impl Task {
    pub fn is_saveable(&self) -> bool {
        !self.project.is_empty() && !self.title.is_empty()
    }

    pub fn accumulate_time(&mut self, seconds: i64) {
        let hours = (seconds as f32) / 3600.0;
        self.time_spent += hours;
        // 小数点第1位に安全に丸めるロジックをモデル側に隠蔽
        self.time_spent = (self.time_spent * 10.0).round() / 10.0;
    }
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: 0,
            active: true,
            status: TaskStatus::Pending,
            project: String::new(),
            title: String::new(),
            detail: String::new(),
            start_date: Zoned::now().date(),
            due_date: Zoned::now().date(),
            priority: Priority::Low,
            progress: 0.0,
            time_spent: 0.0,
            entry_date: Zoned::now().date(),
            end_date: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Priority の TryFrom テスト
    // =========================================================================
    #[test]
    fn test_priority_try_from_valid() {
        assert_eq!(Priority::try_from(0).unwrap(), Priority::Low);
        assert_eq!(Priority::try_from(1).unwrap(), Priority::Medium);
        assert_eq!(Priority::try_from(2).unwrap(), Priority::High);
    }

    #[test]
    fn test_priority_try_from_invalid() {
        let result = Priority::try_from(3);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid priority integer state: 3"
        );

        let result_neg = Priority::try_from(-1);
        assert!(result_neg.is_err());
    }

    // =========================================================================
    // TaskStatus の TryFrom テスト
    // =========================================================================
    #[test]
    fn test_task_status_try_from_valid() {
        assert_eq!(TaskStatus::try_from(0).unwrap(), TaskStatus::Pending);
        assert_eq!(TaskStatus::try_from(1).unwrap(), TaskStatus::WorkInProgress);
        assert_eq!(TaskStatus::try_from(2).unwrap(), TaskStatus::Complete);
        assert_eq!(TaskStatus::try_from(3).unwrap(), TaskStatus::Canceled);
    }

    #[test]
    fn test_task_status_try_from_invalid() {
        let result = TaskStatus::try_from(99);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid task status integer state: 99"
        );
    }

    // =========================================================================
    // Task の Default テスト
    // =========================================================================
    #[test]
    fn test_task_default_values() {
        let task = Task::default();

        // プリミティブ・列挙型のデフォルト値検証
        assert_eq!(task.id, 0);
        assert!(task.active); // 元コードで明示的に true 指定
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.project, "");
        assert_eq!(task.title, "");
        assert_eq!(task.detail, "");
        assert_eq!(task.priority, Priority::Low);
        assert_eq!(task.progress, 0.0);
        assert_eq!(task.time_spent, 0.0);
    }

    #[test]
    fn test_task_default_dates() {
        let task = Task::default();

        // start_date と due_date が両方とも Zoned::now().date() で
        // 同一の日付として初期化されているか検証
        assert_eq!(task.start_date, task.due_date);

        // 少なくとも不正な日付オブジェクトになっていないことの検証
        // (Jiff の Date 型が正常に生成されているか)
        assert!(task.start_date.year() >= 2026);
    }

    #[test]
    fn test_is_saveable() {
        let mut task = Task::default();

        assert!(!task.is_saveable(), "Default task is not saveable.");
        task.project = "Project".to_string();
        assert!(
            !task.is_saveable(),
            "task is lacking title and not saveable."
        );
        task.title = "Title".to_string();
        assert!(task.is_saveable(), "task is saveable.");
    }
}
