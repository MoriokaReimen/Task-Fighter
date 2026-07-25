use anyhow::Result;
use domain::{DailyTask, TaskPriority};
use duckdb::Row;
use duckdb::ToSql;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuckdbDailyTask {
    pub uuid: Uuid,
    pub active: bool,
    pub project: String,
    pub title: String,
    pub detail: String,
    pub priority: i32,
}

impl From<DailyTask> for DuckdbDailyTask {
    fn from(task: DailyTask) -> Self {
        Self {
            uuid: task.uuid,
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

        Ok(Self {
            uuid: duckdb_daily_task.uuid,
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
        Ok(Self {
            uuid: row.get("uuid")?,
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
            ("uuid", &self.uuid as &dyn ToSql),
            ("active", &self.active as &dyn ToSql),
            ("project", &self.project as &dyn ToSql),
            ("title", &self.title as &dyn ToSql),
            ("detail", &self.detail as &dyn ToSql),
            ("priority", &self.priority as &dyn ToSql),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use duckdb::Connection;

    // テスト用のドメインモデルのモック
    fn create_dummy_daily_task(uuid: i32, priority_value: u8) -> DailyTask {
        // ※実際の TaskPriority::try_from などの挙動に合わせてダミーを作ります
        // ここでは、環境に合わせて適宜マッピングを想定してください
        DailyTask {
            uuid,
            active: true,
            project: "TestProject".to_string(),
            title: "TestTitle".to_string(),
            detail: "TestDetail".to_string(),
            // 実際のTaskPriorityのファクトリ、あるいは既存インスタンスを指定
            priority: TaskPriority::try_from(i32::from(priority_value)).unwrap_or_default(),
        }
    }

    #[test]
    fn test_from_daily_task_into_duckdb_daily_task() {
        // 1. DailyTask -> DuckdbDailyTask の相互変換（正常系）のテスト
        let domain_task = create_dummy_daily_task(42, 1);

        // From トレイトの検証
        let duckdb_task = DuckdbDailyTask::from(domain_task.clone());

        assert_eq!(duckdb_task.uuid, domain_task.uuid);
        assert_eq!(duckdb_task.active, domain_task.active);
        assert_eq!(duckdb_task.project, domain_task.project);
        assert_eq!(duckdb_task.title, domain_task.title);
        assert_eq!(duckdb_task.detail, domain_task.detail);
        // priority が正しく i32 にキャストされているか
        assert_eq!(duckdb_task.priority, 1);

        // TryFrom トレイトの検証 (逆変換)
        let converted_domain_task = DailyTask::try_from(duckdb_task).unwrap();
        assert_eq!(converted_domain_task.uuid, domain_task.uuid);
        assert_eq!(converted_domain_task.title, domain_task.title);
    }

    #[test]
    fn test_duckdb_daily_task_try_from_invalid_priority() {
        // 2. 異常系: 不正な priority の値が入っていた場合に TryFrom が失敗するか
        let invalid_duckdb_task = DuckdbDailyTask {
            uuid: 1,
            active: true,
            project: "Proj".to_string(),
            title: "Title".to_string(),
            detail: "Detail".to_string(),
            priority: 999, // 定義外の不正な値
        };

        let result = DailyTask::try_from(invalid_duckdb_task);
        assert!(result.is_err(), "Should fail when priority is invalid");
    }

    #[test]
    fn test_to_named_params_and_map_manipulation() {
        // 3. to_named_params のテスト（HashMap の中身と、キー削除の応用）
        let duckdb_task = DuckdbDailyTask {
            uuid: 100,
            active: false,
            project: "Secret".to_string(),
            title: "Task".to_string(),
            detail: "Desc".to_string(),
            priority: 2,
        };

        let mut params = duckdb_task.to_named_params();

        // 正しくすべてのキーが登録されているか
        assert!(params.contains_key("uuid"));
        assert!(params.contains_key("active"));
        assert!(params.contains_key("project"));
        assert!(params.contains_key("title"));
        assert!(params.contains_key("detail"));
        assert!(params.contains_key("priority"));
        assert_eq!(params.len(), 6);

        // 先ほどの応用：特定のキー（例: uuid）を削除して、残りのパラメータだけを使うようなケースのシミュレート
        let removed_id_param = params.remove("uuid");
        assert!(removed_id_param.is_some());
        assert_eq!(params.len(), 5); // uuid が抜けて5件になっている
        assert!(!params.contains_key("uuid"));
    }

    #[test]
    fn test_try_from_row() -> Result<()> {
        // 4. DBの Row からのパーステスト
        let conn = Connection::open_in_memory()?;

        // ダミーのデータを一件だけ SELECT できるテーブルを作成
        conn.execute(
            "CREATE TABLE temp_tasks (uuid INT, active BOOL, project VARCHAR, title VARCHAR, detail VARCHAR, priority INT);",
            [],
        )?;
        conn.execute(
            "INSERT INTO temp_tasks VALUES (7, true, 'P', 'T', 'D', 1);",
            [],
        )?;

        let mut stmt = conn.prepare(
            "SELECT uuid, active, project, title, detail, priority FROM temp_tasks WHERE uuid = 7;",
        )?;
        let mut rows = stmt.query([])?;

        let row = rows.next()?.unwrap();

        // &Row からの TryFrom を実行
        let parsed_task = DuckdbDailyTask::try_from(row)?;

        assert_eq!(parsed_task.uuid, 7);
        assert!(parsed_task.active);
        assert_eq!(parsed_task.project, "P");
        assert_eq!(parsed_task.title, "T");
        assert_eq!(parsed_task.detail, "D");
        assert_eq!(parsed_task.priority, 1);

        Ok(())
    }
}
