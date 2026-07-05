use crate::task::{Task, TaskPriority, TaskStatus};
use anyhow::Result;
use duckdb::Row;
use duckdb::ToSql;
use jiff::civil::Date;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DuckdbTask {
    pub id: i32,
    pub active: bool,
    pub status: i32,
    pub project: String,
    pub title: String,
    pub detail: String,
    pub start_date: String,
    pub due_date: String,
    pub priority: i32,
    pub progress: f32,
    pub time_spent: f32,
    pub entry_date: String,
    pub end_date: Option<String>,
}

impl From<Task> for DuckdbTask {
    fn from(task: Task) -> Self {
        DuckdbTask {
            id: task.id,
            active: task.active,
            status: task.status as i32,
            project: task.project,
            title: task.title,
            detail: task.detail,
            start_date: task.start_date.to_string(),
            due_date: task.due_date.to_string(),
            priority: task.priority as i32,
            progress: task.progress,
            time_spent: task.time_spent,
            entry_date: task.entry_date.to_string(),
            end_date: task.end_date.map(|d| d.to_string()),
        }
    }
}

impl TryFrom<DuckdbTask> for Task {
    type Error = anyhow::Error;

    fn try_from(duckdb_task: DuckdbTask) -> Result<Self> {
        // jiff::Error を DomainError::InvalidDate にマッピング
        let start_date = Date::from_str(&duckdb_task.start_date)?;
        let due_date = Date::from_str(&duckdb_task.due_date)?;
        let entry_date = Date::from_str(&duckdb_task.entry_date)?;

        let end_date = match duckdb_task.end_date {
            Some(ref s) => Some(Date::from_str(s)?),
            None => None,
        };

        // 各数値を Enum に変換する際のエラーをマッピング
        let priority = TaskPriority::try_from(duckdb_task.priority)?;
        let status = TaskStatus::try_from(duckdb_task.status)?;

        Ok(Task {
            id: duckdb_task.id,
            active: duckdb_task.active,
            status,
            project: duckdb_task.project,
            title: duckdb_task.title,
            detail: duckdb_task.detail,
            start_date,
            due_date,
            priority,
            progress: duckdb_task.progress,
            time_spent: duckdb_task.time_spent,
            entry_date,
            end_date,
        })
    }
}

impl TryFrom<&Row<'_>> for DuckdbTask {
    type Error = duckdb::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(DuckdbTask {
            id: row.get("id")?,
            active: row.get("active")?,
            status: row.get("status")?,
            project: row.get("project")?,
            title: row.get("title")?,
            detail: row.get("detail")?,
            start_date: row.get("start_date")?,
            due_date: row.get("due_date")?,
            priority: row.get("priority")?,
            progress: row.get("progress")?,
            time_spent: row.get("time_spent")?,
            entry_date: row.get("entry_date")?,
            end_date: row.get("end_date")?,
        })
    }
}

impl DuckdbTask {
    pub fn to_named_params(&self) -> HashMap<&str, &dyn ToSql> {
        HashMap::from_iter([
            (":id", &self.id as &dyn ToSql),
            (":active", &self.active as &dyn ToSql),
            (":status", &self.status as &dyn ToSql),
            (":project", &self.project as &dyn ToSql),
            (":title", &self.title as &dyn ToSql),
            (":detail", &self.detail as &dyn ToSql),
            (":start_date", &self.start_date as &dyn ToSql),
            (":due_date", &self.due_date as &dyn ToSql),
            (":priority", &self.priority as &dyn ToSql),
            (":progress", &self.progress as &dyn ToSql),
            (":time_spent", &self.time_spent as &dyn ToSql),
            (":entry_date", &self.entry_date as &dyn ToSql),
            (":end_date", &self.end_date as &dyn ToSql),
        ])
    }
}
