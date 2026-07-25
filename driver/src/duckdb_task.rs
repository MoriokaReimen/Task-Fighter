use anyhow::Result;
use domain::{Task, TaskPriority, TaskStatus};
use duckdb::Row;
use duckdb::ToSql;
use jiff::civil::Date;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct DuckdbTask {
    pub uuid: Uuid,
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
        Self {
            uuid: task.uuid,
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
        let start_date = Date::from_str(&duckdb_task.start_date)?;
        let due_date = Date::from_str(&duckdb_task.due_date)?;
        let entry_date = Date::from_str(&duckdb_task.entry_date)?;

        let end_date = match duckdb_task.end_date {
            Some(ref s) => Some(Date::from_str(s)?),
            None => None,
        };

        let priority = TaskPriority::try_from(duckdb_task.priority)?;
        let status = TaskStatus::try_from(duckdb_task.status)?;

        Ok(Self {
            uuid: duckdb_task.uuid,
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
        Ok(Self {
            uuid: row.get("uuid")?,
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
            ("uuid", &self.uuid as &dyn ToSql),
            ("active", &self.active as &dyn ToSql),
            ("status", &self.status as &dyn ToSql),
            ("project", &self.project as &dyn ToSql),
            ("title", &self.title as &dyn ToSql),
            ("detail", &self.detail as &dyn ToSql),
            ("start_date", &self.start_date as &dyn ToSql),
            ("due_date", &self.due_date as &dyn ToSql),
            ("priority", &self.priority as &dyn ToSql),
            ("progress", &self.progress as &dyn ToSql),
            ("time_spent", &self.time_spent as &dyn ToSql),
            ("entry_date", &self.entry_date as &dyn ToSql),
            ("end_date", &self.end_date as &dyn ToSql),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use duckdb::Connection;
    use jiff::civil::Date;

    // テスト用のドメインモデルモックを生成するヘルパー関数
    fn create_dummy_task(id: i32, end_date: Option<Date>) -> Task {
        Task {
            id,
            active: true,
            status: TaskStatus::try_from(0).unwrap_or_default(),
            project: "TaskProject".to_string(),
            title: "TaskTitle".to_string(),
            detail: "TaskDetail".to_string(),
            start_date: Date::new(2026, 1, 1).unwrap(),
            due_date: Date::new(2026, 1, 31).unwrap(),
            priority: TaskPriority::try_from(1).unwrap_or_default(),
            progress: 50.0,
            time_spent: 12.5,
            entry_date: Date::new(2026, 1, 1).unwrap(),
            end_date,
        }
    }

    #[test]
    fn test_from_task_into_duckdb_task_with_end_date() {
        // 1. 正常系: end_date が Some の場合の相互変換テスト
        let target_date = Date::new(2026, 1, 15).unwrap();
        let domain_task = create_dummy_task(1, Some(target_date));

        // From トレイトの検証
        let duckdb_task = DuckdbTask::from(domain_task.clone());
        assert_eq!(duckdb_task.id, domain_task.id);
        assert_eq!(duckdb_task.start_date, "2026-01-01");
        assert_eq!(duckdb_task.due_date, "2026-01-31");
        assert_eq!(duckdb_task.end_date, Some("2026-01-15".to_string()));
        assert_eq!(duckdb_task.progress, 50.0);

        // TryFrom トレイトの検証 (逆変換)
        let converted_domain = Task::try_from(duckdb_task).unwrap();
        assert_eq!(converted_domain.id, domain_task.id);
        assert_eq!(converted_domain.end_date, Some(target_date));
    }

    #[test]
    fn test_from_task_into_duckdb_task_without_end_date() {
        // 2. 正常系: end_date が None の場合の相互変換テスト
        let domain_task = create_dummy_task(2, None);

        let duckdb_task = DuckdbTask::from(domain_task);
        assert_eq!(duckdb_task.end_date, None);

        let converted_domain = Task::try_from(duckdb_task).unwrap();
        assert_eq!(converted_domain.end_date, None);
    }

    #[test]
    fn test_try_from_invalid_date_format() {
        // 3. 異常系: 日付フォーマットが不正な場合、パースエラーになるか
        let invalid_task = DuckdbTask {
            id: 3,
            active: true,
            status: 0,
            project: "P".to_string(),
            title: "T".to_string(),
            detail: "D".to_string(),
            start_date: "invalid-date-format".to_string(), // 不正な日付
            due_date: "2026-01-31".to_string(),
            priority: 1,
            progress: 0.0,
            time_spent: 0.0,
            entry_date: "2026-01-01".to_string(),
            end_date: None,
        };

        let result = Task::try_from(invalid_task);
        assert!(
            result.is_err(),
            "Should fail when start_date format is invalid"
        );
    }

    #[test]
    fn test_try_from_invalid_enum_values() {
        // 4. 異常系: 不正な status や priority 値の場合、エラーになるか
        let invalid_task = DuckdbTask {
            id: 4,
            active: true,
            status: 999, // 不正なステータス
            project: "P".to_string(),
            title: "T".to_string(),
            detail: "D".to_string(),
            start_date: "2026-01-01".to_string(),
            due_date: "2026-01-31".to_string(),
            priority: -1, // 不正なプライオリティ
            progress: 0.0,
            time_spent: 0.0,
            entry_date: "2026-01-01".to_string(),
            end_date: None,
        };

        let result = Task::try_from(invalid_task);
        assert!(
            result.is_err(),
            "Should fail when status or priority is out of range"
        );
    }

    #[test]
    fn test_to_named_params() {
        // 5. to_named_params のテスト（HashMap の過不足チェックとキー削除）
        let duckdb_task = DuckdbTask {
            id: 10,
            active: true,
            status: 1,
            project: "P".to_string(),
            title: "T".to_string(),
            detail: "D".to_string(),
            start_date: "2026-01-01".to_string(),
            due_date: "2026-01-31".to_string(),
            priority: 1,
            progress: 10.0,
            time_spent: 5.0,
            entry_date: "2026-01-01".to_string(),
            end_date: Some("2026-01-10".to_string()),
        };

        let mut params = duckdb_task.to_named_params();
        assert_eq!(params.len(), 13); // フィールド全13件

        // 主要なキーの存在確認
        assert!(params.contains_key("id"));
        assert!(params.contains_key("progress"));
        assert!(params.contains_key("end_date"));

        // HashMap からキーを削除する挙動の確認
        let removed_progress = params.remove("progress");
        assert!(removed_progress.is_some());
        assert!(!params.contains_key("progress"));
        assert_eq!(params.len(), 12);
    }

    #[test]
    fn test_try_from_row() -> Result<()> {
        // 6. DBの Row からのパーステスト
        let conn = Connection::open_in_memory()?;

        conn.execute(
            "CREATE TABLE temp_tasks (
                id INT, active BOOL, status INT, project VARCHAR, title VARCHAR, detail VARCHAR, 
                start_date VARCHAR, due_date VARCHAR, priority INT, progress REAL, time_spent REAL, 
                entry_date VARCHAR, end_date VARCHAR
            );",
            [],
        )?;

        conn.execute(
            "INSERT INTO temp_tasks VALUES (55, true, 2, 'Proj', 'Title', 'Detail', '2026-02-01', '2026-02-28', 3, 75.0, 4.5, '2026-02-01', NULL);",
            [],
        )?;

        let mut stmt = conn.prepare("SELECT * FROM temp_tasks WHERE id = 55;")?;
        let mut rows = stmt.query([])?;

        let row = rows.next()?.unwrap();

        // 2重参照を避けるため、row をそのまま渡す
        let parsed_task = DuckdbTask::try_from(row)?;

        assert_eq!(parsed_task.id, 55);
        assert!(parsed_task.active);
        assert_eq!(parsed_task.status, 2);
        assert_eq!(parsed_task.project, "Proj");
        assert_eq!(parsed_task.start_date, "2026-02-01");
        assert_eq!(parsed_task.end_date, None); // NULL が None にマッピングされることの確認

        Ok(())
    }
}
