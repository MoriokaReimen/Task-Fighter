use anyhow::Result;
use domain::{TaskPriority, WeeklyTask};
use duckdb::Row;
use duckdb::ToSql;
use jiff::civil::Weekday;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DuckdbWeeklyTask {
    pub id: i32,
    pub active: bool,
    pub project: String,
    pub title: String,
    pub detail: String,
    pub priority: i32,
    pub start_day: i32,
    pub due_day: i32,
}

impl From<WeeklyTask> for DuckdbWeeklyTask {
    fn from(task: WeeklyTask) -> Self {
        DuckdbWeeklyTask {
            id: task.id,
            active: task.active,
            project: task.project,
            title: task.title,
            detail: task.detail,
            priority: task.priority as i32,
            start_day: task.start_day as i32,
            due_day: task.due_day as i32,
        }
    }
}

impl TryFrom<DuckdbWeeklyTask> for WeeklyTask {
    type Error = anyhow::Error;

    fn try_from(duckdb_weekly_task: DuckdbWeeklyTask) -> Result<Self> {
        let priority = TaskPriority::try_from(duckdb_weekly_task.priority)?;
        let start_day = Weekday::from_monday_zero_offset(duckdb_weekly_task.start_day as i8)
            .map_err(|e| anyhow::anyhow!("invalid start_day: {}", e))?;

        let due_day = Weekday::from_monday_zero_offset(duckdb_weekly_task.due_day as i8)
            .map_err(|e| anyhow::anyhow!("invalid due_day: {}", e))?;

        Ok(WeeklyTask {
            id: duckdb_weekly_task.id,
            active: duckdb_weekly_task.active,
            project: duckdb_weekly_task.project,
            title: duckdb_weekly_task.title,
            detail: duckdb_weekly_task.detail,
            priority,
            start_day,
            due_day,
        })
    }
}

impl TryFrom<&Row<'_>> for DuckdbWeeklyTask {
    type Error = duckdb::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(DuckdbWeeklyTask {
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

impl DuckdbWeeklyTask {
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
