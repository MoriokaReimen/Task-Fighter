use anyhow::Result;
use domain::{DailyTask, TaskPriority};
use duckdb::Row;
use duckdb::ToSql;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DuckdbDailyTask {
    pub id: i32,
    pub active: bool,
    pub project: String,
    pub title: String,
    pub detail: String,
    pub priority: i32,
}

impl From<DailyTask> for DuckdbDailyTask {
    fn from(task: DailyTask) -> Self {
        DuckdbDailyTask {
            id: task.id,
            active: task.active,
            project: task.project,
            title: task.title,
            detail: task.detail,
            priority: task.priority as i32,
        }
    }
}

impl TryFrom<DuckdbDailyTask> for DailyTask {
    type Error = anyhow::Error;

    fn try_from(duckdb_daily_task: DuckdbDailyTask) -> Result<Self> {
        let priority = TaskPriority::try_from(duckdb_daily_task.priority)?;

        Ok(DailyTask {
            id: duckdb_daily_task.id,
            active: duckdb_daily_task.active,
            project: duckdb_daily_task.project,
            title: duckdb_daily_task.title,
            detail: duckdb_daily_task.detail,
            priority,
        })
    }
}

impl TryFrom<&Row<'_>> for DuckdbDailyTask {
    type Error = duckdb::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(DuckdbDailyTask {
            id: row.get("id")?,
            active: row.get("active")?,
            project: row.get("project")?,
            title: row.get("title")?,
            detail: row.get("detail")?,
            priority: row.get("priority")?,
        })
    }
}

impl DuckdbDailyTask {
    pub fn to_named_params(&self) -> HashMap<&str, &dyn ToSql> {
        HashMap::from_iter([
            ("id", &self.id as &dyn ToSql),
            ("active", &self.active as &dyn ToSql),
            ("project", &self.project as &dyn ToSql),
            ("title", &self.title as &dyn ToSql),
            ("detail", &self.detail as &dyn ToSql),
            ("priority", &self.priority as &dyn ToSql),
        ])
    }
}
