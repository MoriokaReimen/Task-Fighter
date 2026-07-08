use anyhow::Result;
use domain::{MonthlyTask, TaskPriority};
use duckdb::Row;
use duckdb::ToSql;
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

#[cfg(test)]
mod tests {
    use super::*;
    use duckdb::Connection;

    // テスト用のドメインモデルモックを生成するヘルパー関数
    fn create_dummy_monthly_task(id: i32, priority_value: u8) -> MonthlyTask {
        MonthlyTask {
            id,
            active: true,
            project: "MonthlyProject".to_string(),
            title: "MonthlyTitle".to_string(),
            detail: "MonthlyDetail".to_string(),
            priority: TaskPriority::try_from(priority_value as i32).unwrap_or_default(),
            start_day: 1,
            due_day: 25,
        }
    }

    #[test]
    fn test_from_monthly_task_into_duckdb_monthly_task() {
        // 1. MonthlyTask -> DuckdbMonthlyTask の相互変換（正常系）のテスト
        let domain_task = create_dummy_monthly_task(101, 1);

        // From トレイトの検証
        let duckdb_task = DuckdbMonthlyTask::from(domain_task.clone());

        assert_eq!(duckdb_task.id, domain_task.id);
        assert_eq!(duckdb_task.active, domain_task.active);
        assert_eq!(duckdb_task.project, domain_task.project);
        assert_eq!(duckdb_task.title, domain_task.title);
        assert_eq!(duckdb_task.detail, domain_task.detail);
        assert_eq!(duckdb_task.priority, 1);
        assert_eq!(duckdb_task.start_day, domain_task.start_day);
        assert_eq!(duckdb_task.due_day, domain_task.due_day);

        // TryFrom トレイトの検証 (逆変換)
        let converted_domain_task = MonthlyTask::try_from(duckdb_task).unwrap();
        assert_eq!(converted_domain_task.id, domain_task.id);
        assert_eq!(converted_domain_task.due_day, domain_task.due_day);
    }

    #[test]
    fn test_duckdb_monthly_task_try_from_invalid_priority() {
        // 2. 異常系: 不正な priority の値が入っていた場合に TryFrom が失敗するか
        let invalid_duckdb_task = DuckdbMonthlyTask {
            id: 1,
            active: true,
            project: "Proj".to_string(),
            title: "Title".to_string(),
            detail: "Detail".to_string(),
            priority: -5, // 定義外のマイナス値や不正値
            start_day: 1,
            due_day: 28,
        };

        let result = MonthlyTask::try_from(invalid_duckdb_task);
        assert!(
            result.is_err(),
            "Should fail when priority value is invalid"
        );
    }

    #[test]
    fn test_to_named_params() {
        // 3. to_named_params のテスト（HashMap のキーと値の保持チェック）
        let duckdb_task = DuckdbMonthlyTask {
            id: 500,
            active: true,
            project: "Billing".to_string(),
            title: "Invoice".to_string(),
            detail: "Send invoice".to_string(),
            priority: 2,
            start_day: 20,
            due_day: 30,
        };

        let mut params = duckdb_task.to_named_params();

        // 必要なキーがすべて存在することを確認
        assert_eq!(params.len(), 8);
        assert!(params.contains_key("id"));
        assert!(params.contains_key("active"));
        assert!(params.contains_key("project"));
        assert!(params.contains_key("title"));
        assert!(params.contains_key("detail"));
        assert!(params.contains_key("priority"));
        assert!(params.contains_key("start_day"));
        assert!(params.contains_key("due_day"));

        // 前回の応用：キーの削除も問題なく行えるか確認
        let removed_due_day = params.remove("due_day");
        assert!(removed_due_day.is_some());
        assert!(!params.contains_key("due_day"));
        assert_eq!(params.len(), 7);
    }

    #[test]
    fn test_try_from_row() -> Result<()> {
        // 4. DBの Row からのパーステスト
        let conn = Connection::open_in_memory()?;

        // DDLのカラム名（end_day）に合わせつつ、取得時に構造体のフィールド名（due_day）へとエイリアスを貼る
        conn.execute(
            "CREATE TABLE temp_monthly_tasks (id INT, active BOOL, project VARCHAR, title VARCHAR, detail VARCHAR, priority INT, start_day SMALLINT, end_day SMALLINT);",
            [],
        )?;
        conn.execute(
            "INSERT INTO temp_monthly_tasks VALUES (77, true, 'ProjX', 'TitleX', 'DetailX', 3, 5, 25);",
            [],
        )?;

        // row.get("due_day") を満たすため、SELECT 句で `end_day AS due_day` とする
        let mut stmt = conn.prepare(
            "SELECT id, active, project, title, detail, priority, start_day, end_day AS due_day FROM temp_monthly_tasks WHERE id = 77;"
        )?;
        let mut rows = stmt.query([])?;

        let row = rows.next()?.unwrap();

        // 【修正点】&row での2重参照を避け、型に適合するよう row をそのまま渡す
        let parsed_task = DuckdbMonthlyTask::try_from(row)?;

        assert_eq!(parsed_task.id, 77);
        assert!(parsed_task.active);
        assert_eq!(parsed_task.project, "ProjX");
        assert_eq!(parsed_task.title, "TitleX");
        assert_eq!(parsed_task.detail, "DetailX");
        assert_eq!(parsed_task.priority, 3);
        assert_eq!(parsed_task.start_day, 5);
        assert_eq!(parsed_task.due_day, 25);

        Ok(())
    }
}
