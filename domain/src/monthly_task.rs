use crate::task::{TaskPriority, Task};
use jiff::civil::Date;
use anyhow::{Result, bail};

pub struct MonthlyTask {
    pub id: i32,
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
