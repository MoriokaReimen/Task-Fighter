use anyhow::Result;
use domain::WorkTime;
use duckdb::Row;
use duckdb::ToSql;
use jiff::civil::Date;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DuckdbWorkTime {
    pub id: i32,
    pub task_id: i32,
    pub date: String,
    pub time_spent: f32,
}

impl From<WorkTime> for DuckdbWorkTime {
    fn from(work_time: WorkTime) -> Self {
        DuckdbWorkTime {
            id: work_time.id,
            task_id: work_time.task_id,
            date: work_time.date.to_string(),
            time_spent: work_time.time_spent,
        }
    }
}

impl TryFrom<DuckdbWorkTime> for WorkTime {
    type Error = anyhow::Error;

    fn try_from(duckdb_work_time: DuckdbWorkTime) -> Result<Self> {
        let date = Date::from_str(&duckdb_work_time.date)?;
        Ok(WorkTime {
            id: duckdb_work_time.id,
            task_id: duckdb_work_time.task_id,
            date,
            time_spent: duckdb_work_time.time_spent,
        })
    }
}

impl TryFrom<&Row<'_>> for DuckdbWorkTime {
    type Error = duckdb::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(DuckdbWorkTime {
            id: row.get("id")?,
            task_id: row.get("task_id")?,
            date: row.get("date")?,
            time_spent: row.get("time_spent")?,
        })
    }
}

impl DuckdbWorkTime {
    pub fn to_named_params(&self) -> HashMap<&str, &dyn ToSql> {
        HashMap::from_iter([
            ("id", &self.id as &dyn ToSql),
            ("task_id", &self.task_id as &dyn ToSql),
            ("date", &self.date as &dyn ToSql),
            ("time_spent", &self.time_spent as &dyn ToSql),
        ])
    }
}
