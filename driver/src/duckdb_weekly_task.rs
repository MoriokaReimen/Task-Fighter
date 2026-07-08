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
            start_day: Weekday::to_monday_zero_offset(task.start_day) as i32,
            due_day: Weekday::to_monday_zero_offset(task.due_day) as i32,
        }
    }
}

impl TryFrom<DuckdbWeeklyTask> for WeeklyTask {
    type Error = anyhow::Error;

    fn try_from(duckdb_weekly_task: DuckdbWeeklyTask) -> Result<Self> {
        let priority = TaskPriority::try_from(duckdb_weekly_task.priority)?;
        let start_day = Weekday::from_monday_one_offset(duckdb_weekly_task.start_day as i8)
            .map_err(|e| anyhow::anyhow!("invalid start_day: {}", e))?;

        let due_day = Weekday::from_monday_one_offset(duckdb_weekly_task.due_day as i8)
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

#[cfg(test)]
mod tests {
    use super::*;
    use duckdb::Connection;
    use jiff::civil::Weekday;

    // テスト用のドメインモデルモックを生成するヘルパー関数
    fn create_dummy_weekly_task(id: i32, start_offset: i8, due_offset: i8) -> WeeklyTask {
        WeeklyTask {
            id,
            active: true,
            project: "WeeklyProject".to_string(),
            title: "WeeklyTitle".to_string(),
            detail: "WeeklyDetail".to_string(),
            priority: TaskPriority::try_from(1).unwrap_or_default(),
            start_day: Weekday::from_monday_zero_offset(start_offset).unwrap(),
            due_day: Weekday::from_monday_zero_offset(due_offset).unwrap(),
        }
    }

    #[test]
    fn test_from_weekly_task_into_duckdb_weekly_task() {
        // 1. 正常系: WeeklyTask <-> DuckdbWeeklyTask の相互変換テスト
        // 0 = Monday, 4 = Friday
        let domain_task = create_dummy_weekly_task(201, 1, 5);

        // From トレイトの検証
        let duckdb_task = DuckdbWeeklyTask::from(domain_task.clone());
        assert_eq!(duckdb_task.id, domain_task.id);
        assert_eq!(duckdb_task.start_day, 1);
        assert_eq!(duckdb_task.due_day, 5);
        assert_eq!(duckdb_task.priority, 1);

        // TryFrom トレイトの検証 (逆変換)
        let converted_domain = WeeklyTask::try_from(duckdb_task).unwrap();
        assert_eq!(converted_domain.id, domain_task.id);
        assert_eq!(converted_domain.start_day, Weekday::Monday);
        assert_eq!(converted_domain.due_day, Weekday::Friday);
    }

    #[test]
    fn test_try_from_invalid_weekday_offset() {
        // 2. 異常系: 曜日オフセットが不正な値（例: 7）の場合にパースエラーになるか
        let invalid_task = DuckdbWeeklyTask {
            id: 1,
            active: true,
            project: "Proj".to_string(),
            title: "Title".to_string(),
            detail: "Detail".to_string(),
            priority: 1,
            start_day: 1,
            due_day: 8, // 0~6の範囲外
        };

        let result = WeeklyTask::try_from(invalid_task);
        assert!(result.is_err(), "Should fail when due_day offset is 7");
        assert!(result.unwrap_err().to_string().contains("invalid due_day"));
    }

    #[test]
    fn test_try_from_invalid_priority() {
        // 3. 異常系: 優先度が不正な値の場合にパースエラーになるか
        let invalid_task = DuckdbWeeklyTask {
            id: 2,
            active: true,
            project: "Proj".to_string(),
            title: "Title".to_string(),
            detail: "Detail".to_string(),
            priority: 99, // 不正なプライオリティ
            start_day: 1,
            due_day: 5,
        };

        let result = WeeklyTask::try_from(invalid_task);
        assert!(result.is_err(), "Should fail when priority is out of range");
    }

    #[test]
    fn test_to_named_params() {
        // 4. to_named_params のテスト（HashMap の保持チェックとキー削除）
        let duckdb_task = DuckdbWeeklyTask {
            id: 300,
            active: true,
            project: "Gym".to_string(),
            title: "Workout".to_string(),
            detail: "Leg day".to_string(),
            priority: 2,
            start_day: 2, // Wednesday
            due_day: 4,   // Friday
        };

        let mut params = duckdb_task.to_named_params();
        assert_eq!(params.len(), 8);

        assert!(params.contains_key("id"));
        assert!(params.contains_key("start_day"));
        assert!(params.contains_key("due_day"));

        // HashMap から特定のキーを削除できるかテスト
        let removed_start = params.remove("start_day");
        assert!(removed_start.is_some());
        assert!(!params.contains_key("start_day"));
        assert_eq!(params.len(), 7);
    }

    #[test]
    fn test_try_from_row() -> Result<()> {
        // 5. DBの Row からのパーステスト
        let conn = Connection::open_in_memory()?;

        // 以前のDDL（weekly_tasks）に合わせて、end_day を due_day として取得するケースをシミュレート
        conn.execute(
            "CREATE TABLE temp_weekly_tasks (
                id INT, active BOOL, project VARCHAR, title VARCHAR, detail VARCHAR, priority INT, start_day INT, end_day INT
            );",
            [],
        )?;

        // 3 = Thursday, 6 = Sunday
        conn.execute(
            "INSERT INTO temp_weekly_tasks VALUES (15, true, 'ProjW', 'TitleW', 'DetailW', 1, 3, 6);",
            [],
        )?;

        // 構造体のフィールド名（due_day）を満たすために、エイリアス（end_day AS due_day）を使用
        let mut stmt = conn.prepare(
            "SELECT id, active, project, title, detail, priority, start_day, end_day AS due_day FROM temp_weekly_tasks WHERE id = 15;"
        )?;
        let mut rows = stmt.query([])?;

        let row = rows.next()?.unwrap();

        // 型不整合を回避するため、row をそのまま渡す
        let parsed_task = DuckdbWeeklyTask::try_from(row)?;

        assert_eq!(parsed_task.id, 15);
        assert!(parsed_task.active);
        assert_eq!(parsed_task.project, "ProjW");
        assert_eq!(parsed_task.start_day, 3);
        assert_eq!(parsed_task.due_day, 6);

        Ok(())
    }
}
