use anyhow::Result;
use domain::{TaskPriority, MonthlyTask};
use duckdb::Row;
use duckdb::ToSql;
use jiff::civil::Weekday;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DuckdbMonthlyTask {
    pub id: i32,
    pub active: bool,
    pub project: String,
    pub title: String,
    pub detail: String,
    pub priority: i32,
    pub start_day: i16,
    pub due_day: i16,
}

impl From<MonthlyTask> for DuckdbMonthlyTask {
    fn from(task: MonthlyTask) -> Self {
        DuckdbMonthlyTask {
            id: task.id,
            active: task.active,
            project: task.project,
            title: task.title,
            detail: task.detail,
            priority: task.priority as i32,
            start_day: task.start_day,
            due_day: task.due_day,
        }
    }
}

impl TryFrom<DuckdbMonthlyTask> for MonthlyTask {
    type Error = anyhow::Error;

    fn try_from(duckdb_monthly_task: DuckdbMonthlyTask) -> Result<Self> {
        let priority = TaskPriority::try_from(duckdb_monthly_task.priority)?;
        Ok(MonthlyTask {
            id: duckdb_monthly_task.id,
            active: duckdb_monthly_task.active,
            project: duckdb_monthly_task.project,
            title: duckdb_monthly_task.title,
            detail: duckdb_monthly_task.detail,
            priority,
            start_day: duckdb_monthly_task.start_day,
            due_day: duckdb_monthly_task.due_day,
        })
    }
}

impl TryFrom<&Row<'_>> for DuckdbMonthlyTask {
    type Error = duckdb::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(DuckdbMonthlyTask {
            id: row.get("id")?,
            active: row.get("active")?,
            project: row.get("project")?,
            title: row.get("title")?,
            detail: row.get("detail")?,
            priority: row.get("priority")?,
            start_day: row.get("start_day")?,
            due_day: row.get("due_day")?,
        })
    }
}

impl DuckdbMonthlyTask {
    pub fn to_named_params(&self) -> HashMap<&str, &dyn ToSql> {
        HashMap::from_iter([
            ("id", &self.id as &dyn ToSql),
            ("active", &self.active as &dyn ToSql),
            ("project", &self.project as &dyn ToSql),
            ("title", &self.title as &dyn ToSql),
            ("detail", &self.detail as &dyn ToSql),
            ("priority", &self.priority as &dyn ToSql),
            ("start_day", &self.start_day as &dyn ToSql),
            ("due_day", &self.due_day as &dyn ToSql),
        ])
    }
}
