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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DuckdbPath;
    use crate::connect;
    use duckdb::Connection;
    use jiff::civil::Date;

    // テスト用のドメインモデルモックを生成するヘルパー関数
    fn create_dummy_work_time(id: i32, task_id: i32, date: Date) -> WorkTime {
        WorkTime {
            id,
            task_id,
            date,
            time_spent: 2.5,
        }
    }

    #[test]
    fn test_from_work_time_into_duckdb_work_time() {
        // 1. 正常系: WorkTime <-> DuckdbWorkTime の相互変換テスト
        let test_date = Date::new(2026, 7, 8).unwrap();
        let domain_work_time = create_dummy_work_time(10, 42, test_date);

        // From トレイトの検証
        let duckdb_work_time = DuckdbWorkTime::from(domain_work_time.clone());
        assert_eq!(duckdb_work_time.id, 10);
        assert_eq!(duckdb_work_time.task_id, 42);
        assert_eq!(duckdb_work_time.date, "2026-07-08");
        assert_eq!(duckdb_work_time.time_spent, 2.5);

        // TryFrom トレイトの検証 (逆変換)
        let converted_domain = WorkTime::try_from(duckdb_work_time).unwrap();
        assert_eq!(converted_domain.id, domain_work_time.id);
        assert_eq!(converted_domain.task_id, domain_work_time.task_id);
        assert_eq!(converted_domain.date, test_date);
        assert_eq!(converted_domain.time_spent, domain_work_time.time_spent);
    }

    #[test]
    fn test_try_from_invalid_date_format() {
        // 2. 異常系: 日付フォーマットが不正な場合にパースエラーを返すか検証
        let invalid_work_time = DuckdbWorkTime {
            id: 1,
            task_id: 100,
            date: "invalid-date".to_string(), // 壊れた日付データ
            time_spent: 1.0,
        };

        let result = WorkTime::try_from(invalid_work_time);
        assert!(
            result.is_err(),
            "Should fail when date string fails in parse"
        );
    }

    #[test]
    fn test_to_named_params() {
        // 3. to_named_params のテスト（HashMap の検証とキー削除確認）
        let duckdb_work_time = DuckdbWorkTime {
            id: 55,
            task_id: 99,
            date: "2026-12-31".to_string(),
            time_spent: 8.0,
        };

        let mut params = duckdb_work_time.to_named_params();
        assert_eq!(params.len(), 4);

        assert!(params.contains_key("id"));
        assert!(params.contains_key("task_id"));
        assert!(params.contains_key("date"));
        assert!(params.contains_key("time_spent"));

        // 先ほどの HashMap 削除操作の応用
        let removed_time = params.remove("time_spent");
        assert!(removed_time.is_some());
        assert!(!params.contains_key("time_spent"));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_try_from_row() -> Result<()> {
        let path = DuckdbPath::InMemory;
        let conn = connect(&path);
        let conn = conn.unwrap();

        conn.execute(
            "INSERT INTO work_time (task_id, date, time_spent) VALUES (101, '2026-05-10', 4.5);",
            [],
        )?;

        let mut stmt = conn.prepare("SELECT id, task_id, CAST(date AS VARCHAR) AS date, time_spent FROM work_time WHERE id = 1;")?;
        let mut rows = stmt.query([])?;

        let row = rows.next()?.unwrap();

        // unsatisfied trait bound を回避するため、&row ではなく row を直接渡す
        let parsed_work_time = DuckdbWorkTime::try_from(row)?;

        assert_eq!(parsed_work_time.id, 1);
        assert_eq!(parsed_work_time.task_id, 101);
        // DuckDB の DATE 型は SELECT 時点で標準の文字列等として row.get で回収可能
        assert_eq!(parsed_work_time.date, "2026-05-10");
        assert_eq!(parsed_work_time.time_spent, 4.5);

        Ok(())
    }
}
