use crate::task::{Task, TaskPriority};
use anyhow::{Result, bail};
use jiff::civil::Date;

pub struct DailyTask {
    pub id: i32,
    pub project: String,
    pub title: String,
    pub detail: String,
    pub priority: TaskPriority,
}

impl Default for DailyTask {
    fn default() -> Self {
        Self {
            id: 0,
            project: String::new(),
            title: String::new(),
            detail: String::new(),
            priority: TaskPriority::Low,
        }
    }
}

impl DailyTask {
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
