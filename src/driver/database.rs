use crate::driver::Task;
use crate::driver::{Priority, TaskStatus};
use anyhow::{Context, Result, bail};
use duckdb::{Connection, params};
use jiff::Zoned;
use jiff::civil::Date;
use std::fs;
use std::path::Path;
use tracing::info;

/// Centralized mapper to convert a database row slice into a Task token instance,
/// significantly flattening nesting inside fetch functions.
impl<'a> TryFrom<&'a duckdb::Row<'a>> for Task {
    type Error = duckdb::Error;

    fn try_from(row: &'a duckdb::Row<'a>) -> Result<Self, Self::Error> {
        let status_raw: i32 = row.get(2)?;
        let priority_raw: i32 = row.get(8)?;

        let status = TaskStatus::try_from(status_raw).map_err(|e| {
            duckdb::Error::FromSqlConversionFailure(2, duckdb::types::Type::Int, e.into())
        })?;
        let priority = Priority::try_from(priority_raw).map_err(|e| {
            duckdb::Error::FromSqlConversionFailure(8, duckdb::types::Type::Int, e.into())
        })?;

        let start_date_str: String = row.get(6)?;
        let due_date_str: String = row.get(7)?;

        let start_date = start_date_str.parse::<Date>().map_err(|e| {
            duckdb::Error::FromSqlConversionFailure(6, duckdb::types::Type::Text, e.into())
        })?;
        let due_date = due_date_str.parse::<Date>().map_err(|e| {
            duckdb::Error::FromSqlConversionFailure(7, duckdb::types::Type::Text, e.into())
        })?;

        Ok(Task {
            id: row.get(0)?,
            active: row.get(1)?,
            status,
            project: row.get(3)?,
            title: row.get(4)?,
            detail: row.get(5)?,
            start_date,
            due_date,
            priority,
            progress: row.get(9)?,
            time_spent: row.get(10)?,
        })
    }
}

pub fn connect() -> Result<Connection> {
    let path = Path::new("runtime");
    if path.exists() && !path.is_dir() {
        bail!("'runtime' exists but is a file. Expected a directory context path target.");
    }
    fs::create_dir_all(path).context("Failed to safely initialize target 'runtime' directory")?;

    let conn = Connection::open("./runtime/task_fighter.db")
        .context("Failed to establish DuckDB database file handle stream connection")?;

    // 💡 1. 自動インクリメント用のシーケンス（SEQUENCE）を作成
    conn.execute("CREATE SEQUENCE IF NOT EXISTS tasks_id_seq START 1;", [])
        .context("Failed to create sequence for tasks id")?;

    // 💡 2. テーブル作成時に DEFAULT nextval('tasks_id_seq') を指定
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id          INTEGER PRIMARY KEY DEFAULT nextval('tasks_id_seq'),
            active      BOOL NOT NULL DEFAULT true,
            status      UTINYINT NOT NULL DEFAULT 0,
            project     VARCHAR NOT NULL,
            title       VARCHAR NOT NULL,
            detail      VARCHAR NOT NULL,
            start_date  DATE NOT NULL,
            due_date    DATE NOT NULL,
            entry_date  DATE NOT NULL,
            end_date    DATE,
            priority    UTINYINT NOT NULL DEFAULT 1,
            progress    REAL NOT NULL DEFAULT 0.0 CHECK(progress >= 0.0 AND progress <= 100.0),
            time_spent  REAL NOT NULL DEFAULT 0.0
        );",
        [],
    )
    .context("Failed executing target master initialization schema table creation migrations")?;
    info!("Database and target system schemas synchronized cleanly.");

    Ok(conn)
}

pub fn insert_task(conn: &Connection, task: &Task) -> Result<()> {
    info!("Inserting task record token: {:?}", task);

    let sql = "INSERT INTO tasks (active, status, project, title, detail, start_date, due_date, priority, progress, time_spent, entry_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)";
    let entry_date = Zoned::now().date();

    conn.execute(
        sql,
        params![
            task.active,
            task.status as i32,
            task.project,
            task.title,
            task.detail,
            task.start_date.to_string(),
            task.due_date.to_string(),
            task.priority as i32,
            task.progress,
            task.time_spent,
            entry_date.to_string()
        ],
    )
    .context("Failed to commit novel dataset item to relational datastore row bounds")?;

    Ok(())
}

fn exists_id(conn: &Connection, id: i32) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT 1 FROM tasks WHERE id = ?;")?;
    let exists = stmt
        .exists(duckdb::params![id])
        .context("Failed to check if task ID exists")?;

    Ok(exists)
}

pub fn upsert_task(conn: &Connection, task: &Task) -> Result<()> {
    let exists = exists_id(conn, task.id)?;
    if exists {
        info!("ID {} exists. Update task.", task.id);
        update_task(conn, task)?;
    } else {
        info!("ID {} not exists. Create new task.", task.id);
        insert_task(conn, task)?;
    }

    Ok(())
}

pub fn get_next_id(conn: &Connection) -> Result<i32> {
    let query = "SELECT last_value FROM duckdb_sequences() WHERE sequence_name = 'tasks_id_seq';";
    let last_value: Option<i64> = conn
        .query_row(query, [], |row| row.get(0))
        .context("Failed to query next sequence value from DuckDB catalogs")?;
    let next_id = last_value.map(|val| (val + 1) as i32).unwrap_or(1);
    info!("Next task id is {}.", next_id);

    Ok(next_id)
}

pub fn fetch_active_tasks(conn: &Connection) -> Result<Vec<Task>> {
    info!("Querying active tasks");
    let mut stmt = conn.prepare(
        "SELECT id, active, status, project, title, detail, start_date::VARCHAR, due_date::VARCHAR, priority, progress, time_spent FROM tasks WHERE active = true ORDER BY priority DESC"
    ).context("Failed compiling relational parameter statements validations queries")?;

    let tasks = stmt
        .query_map([], |row| Task::try_from(row))?
        .collect::<Result<Vec<Task>, _>>()
        .context("Failed parsing query sequences lists mapping constraints rows")?;
    info!("{} tasks queried.", tasks.len());

    Ok(tasks)
}

pub fn update_task(conn: &Connection, task: &Task) -> Result<()> {
    let mut stmt = conn.prepare(
        "UPDATE tasks 
         SET active = ?1, status = ?2, project = ?3, title = ?4, detail = ?5, start_date = ?6, due_date = ?7, priority = ?8, progress = ?9, time_spent = ?10 
         WHERE id = ?11",
    ).context("Failed compiling database structural mutations modification pipelines statements")?;

    let rows_affected = stmt
        .execute(params![
            task.active,
            task.status as i32,
            task.project,
            task.title,
            task.detail,
            task.start_date.to_string(),
            task.due_date.to_string(),
            task.priority as i32,
            task.progress,
            task.time_spent,
            task.id
        ])
        .context("Failed executing datastore entity mutations pipelines updates states")?;

    if rows_affected == 0 {
        bail!(
            "Target modification bounds target identity token is non-existent: Identity [{}]",
            task.id
        );
    }

    if task.status == TaskStatus::Complete || task.status == TaskStatus::Canceled {
        let end_date = Zoned::now().date();
        let mut stmt = conn
            .prepare(
                "UPDATE tasks 
            SET end_date = ?2
            WHERE id = ?1",
            )
            .context(
                "Failed compiling database structural mutations modification pipelines statements",
            )?;

        let rows_affected = stmt
            .execute(params![task.id, end_date.to_string()])
            .context("Failed executing datastore entity mutations pipelines updates states")?;

        if rows_affected == 0 {
            bail!(
                "Target modification bounds target identity token is non-existent: Identity [{}]",
                task.id
            );
        }
    }

    info!("Task {} update success.", task.id);
    Ok(())
}

pub fn count_tasks_by_date(conn: &Connection, target_date: Date) -> Result<(i32, i32, i32, i32)> {
    let sql = r#"
            SELECT COUNT(*) 
            FROM tasks
            WHERE start_date <= ?1 AND (end_date => ?1 OR end_date IS NULL) AND status = ?2
        "#;

    let pending_count: i32 =
        conn.query_row(sql, params![target_date.to_string(), 0], |row| row.get(0))?;
    let work_in_progress_count: i32 =
        conn.query_row(sql, params![target_date.to_string(), 1], |row| row.get(0))?;
    let complete_count: i32 =
        conn.query_row(sql, params![target_date.to_string(), 2], |row| row.get(0))?;
    let canceled_count: i32 =
        conn.query_row(sql, params![target_date.to_string(), 3], |row| row.get(0))?;

    Ok((
        pending_count,
        work_in_progress_count,
        complete_count,
        canceled_count,
    ))
}

pub fn scan_tasks(conn: &Connection, pattern: &str, only_active: bool) -> Result<Vec<Task>> {
    info!("Scanning tasks with regex pattern: {}", pattern);

    let trimmed = pattern.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    // 💡 LIKE の代わりに regexp_matches(対象, パターン, オプション) を使用
    // 'i' オプションを付与することで、大文字・小文字を区別せずに正規表現マッチングを行います
    let sql = if only_active {
        "SELECT id, active, status, project, title, detail, start_date::TEXT, due_date::TEXT, priority, progress, time_spent \
         FROM tasks \
         WHERE (regexp_matches(title, ?1, 'i') OR regexp_matches(detail, ?1, 'i') OR regexp_matches(project, ?1, 'i')) AND active = true \
         ORDER BY priority DESC;"
    } else {
        "SELECT id, active, status, project, title, detail, start_date::TEXT, due_date::TEXT, priority, progress, time_spent \
         FROM tasks \
         WHERE (regexp_matches(title, ?1, 'i') OR regexp_matches(detail, ?1, 'i') OR regexp_matches(project, ?1, 'i')) \
         ORDER BY priority DESC;"
    };

    let mut stmt = conn
        .prepare(sql)
        .context("Failed to prepare regex match database query statement")?;

    // 💡 LIKE の時のような前後の `%` は不要になり、入力された正規表現をそのまま渡します
    let tasks = stmt
        .query_map([trimmed], |row| Task::try_from(row))?
        .collect::<Result<Vec<Task>, _>>()
        .context("Failed parsing query sequences lists mapping constraints rows")?;

    info!("{} tasks queried.", tasks.len());
    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::Date;
    use jiff::civil::date;
    use rand::RngExt;
    use rand::prelude::IndexedRandom;

    fn setup_in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        // 💡 1. 自動インクリメント用のシーケンス（SEQUENCE）を作成
        conn.execute("CREATE SEQUENCE IF NOT EXISTS tasks_id_seq START 1;", [])
            .unwrap();

        // 💡 2. テーブル作成時に DEFAULT nextval('tasks_id_seq') を指定
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
            id          INTEGER PRIMARY KEY DEFAULT nextval('tasks_id_seq'),
            active      BOOL NOT NULL DEFAULT 1,
            status      INTEGER NOT NULL DEFAULT 0,
            project     VARCHAR NOT NULL,
            title       VARCHAR NOT NULL,
            detail      VARCHAR NOT NULL,
            start_date  DATE NOT NULL,
            due_date    DATE NOT NULL,
            priority    INTEGER NOT NULL DEFAULT 1,
            progress    FLOAT NOT NULL DEFAULT 0.0 CHECK(progress >= 0.0 AND progress <= 100.0),
            time_spent  FLOAT NOT NULL DEFAULT 0.0
        );",
            [],
        )
        .unwrap();

        conn
    }

    fn create_test_task(title: &str, detail: &str) -> Task {
        Task {
            id: 0,
            active: true,
            status: TaskStatus::WorkInProgress,
            project: "TestProject".to_string(),
            title: title.to_string(),
            detail: detail.to_string(),
            start_date: date(2026, 1, 1),
            due_date: date(2026, 1, 10),
            priority: Priority::Medium,
            progress: 50.0,
            time_spent: 2.5,
        }
    }

    #[test]
    fn test_priority_try_from_valid() {
        assert_eq!(Priority::try_from(0).unwrap(), Priority::Low);
        assert_eq!(Priority::try_from(1).unwrap(), Priority::Medium);
        assert_eq!(Priority::try_from(2).unwrap(), Priority::High);
    }

    #[test]
    fn test_priority_try_from_invalid() {
        assert!(Priority::try_from(-1).is_err());
        assert!(Priority::try_from(3).is_err());
    }

    #[test]
    fn test_task_default() {
        let task = Task::default();
        assert_eq!(task.id, 0);
        assert!(task.active);
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.priority, Priority::Low);
        assert_eq!(task.progress, 0.0);
    }

    #[test]
    fn test_insert_and_fetch_task_by_id() {
        let conn = setup_in_memory_db();
        let task = create_test_task("New Task Item", "Detailed Specs Context");

        insert_task(&conn, &task).unwrap();

        let fetched = fetch_task_by_id(&conn, 1).unwrap();
        assert_eq!(fetched.id, 1);
        assert_eq!(fetched.title, "New Task Item");
        assert_eq!(fetched.detail, "Detailed Specs Context");
        assert_eq!(fetched.progress, 50.0);
    }

    #[test]
    fn test_fetch_task_by_id_not_found() {
        let conn = setup_in_memory_db();
        let result = fetch_task_by_id(&conn, 999);
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_active_and_incomplete_tasks() {
        let conn = setup_in_memory_db();

        let task1 = Task {
            active: true,
            status: TaskStatus::WorkInProgress,
            ..create_test_task("T1", "D1")
        };
        let task2 = Task {
            active: false,
            status: TaskStatus::Pending,
            ..create_test_task("T2", "D2")
        };
        let task3 = Task {
            active: true,
            status: TaskStatus::Complete,
            ..create_test_task("T3", "D3")
        };

        insert_task(&conn, &task1).unwrap();
        insert_task(&conn, &task2).unwrap();
        insert_task(&conn, &task3).unwrap();

        let active_tasks = fetch_active_tasks(&conn).unwrap();
        assert_eq!(active_tasks.len(), 2);
        assert!(active_tasks.iter().any(|t| t.title == "T1"));
        assert!(active_tasks.iter().any(|t| t.title == "T3"));

        let incomplete_tasks = fetch_incomplete_tasks(&conn).unwrap();
        assert_eq!(incomplete_tasks.len(), 2);
        assert!(incomplete_tasks.iter().any(|t| t.title == "T1"));
        assert!(incomplete_tasks.iter().any(|t| t.title == "T2"));
    }

    #[test]
    fn test_update_task() {
        let conn = setup_in_memory_db();
        let task = create_test_task("Before Mutation State", "Initial Context Document");
        insert_task(&conn, &task).unwrap();

        let mut to_update = fetch_task_by_id(&conn, 1).unwrap();
        to_update.title = "Mutated State Token".to_string();
        to_update.status = TaskStatus::Complete;
        to_update.progress = 100.0;

        update_task(&conn, &to_update).unwrap();

        let updated = fetch_task_by_id(&conn, 1).unwrap();
        assert_eq!(updated.title, "Mutated State Token");
        assert_eq!(updated.progress, 100.0);
    }

    #[test]
    fn test_update_task_not_found() {
        let conn = setup_in_memory_db();
        let mut task = create_test_task(
            "Missing Context ID Boundaries Verification",
            "Data Payload Mock",
        );
        task.id = 999;

        let result = update_task(&conn, &task);
        assert!(result.is_err());
    }

    // テスト用のインメモリ接続とテーブル初期化を行うヘルパー関数
    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        // 💡 1. 自動インクリメント用のシーケンス（SEQUENCE）を作成
        conn.execute("CREATE SEQUENCE IF NOT EXISTS tasks_id_seq START 1;", [])
            .unwrap();

        // 💡 2. テーブル作成時に DEFAULT nextval('tasks_id_seq') を指定
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
            id          INTEGER PRIMARY KEY DEFAULT nextval('tasks_id_seq'),
            active      INTEGER NOT NULL DEFAULT 1,
            status      INTEGER NOT NULL DEFAULT 0,
            project     VARCHAR NOT NULL,
            title       VARCHAR NOT NULL,
            detail      VARCHAR NOT NULL,
            start_date  DATE NOT NULL,
            due_date    DATE NOT NULL,
            priority    INTEGER NOT NULL DEFAULT 1,
            progress    FLOAT NOT NULL DEFAULT 0.0 CHECK(progress >= 0.0 AND progress <= 100.0),
            time_spent  FLOAT NOT NULL DEFAULT 0.0
        );",
            [],
        )
        .unwrap();

        conn
    }

    // テスト用のダミーデータ挿入ヘルパー
    fn insert_dummy_task(conn: &Connection, project: &str, title: &str, detail: &str) {
        let task = Task {
            project: project.to_string(),
            title: title.to_string(),
            detail: detail.to_string(),
            start_date: Date::new(2026, 1, 1).unwrap(),
            due_date: Date::new(2026, 1, 7).unwrap(),
            ..Default::default()
        };
        insert_task(conn, &task).unwrap();
    }

    #[test]
    fn test_scan_tasks_empty_pattern() {
        let conn = setup_test_db();
        insert_dummy_task(&conn, "Rust", "Fix a bug", "Rust memory leak");

        // 💡 空文字、または空白のみの場合は空のベクターが返るか
        let results = scan_tasks(&conn, "", false).unwrap();
        assert!(results.is_empty());

        let results = scan_tasks(&conn, "   ", false).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_scan_tasks_match_title() {
        let conn = setup_test_db();
        insert_dummy_task(&conn, "Rust", "Fix a memory leak", "Critical issue");
        insert_dummy_task(&conn, "Go", "Refactor code", "Clean up");

        // 💡 タイトル（title）の部分一致で正しくヒットするか
        let results = scan_tasks(&conn, "memory", false).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Fix a memory leak");
    }

    #[test]
    fn test_scan_tasks_match_project_and_detail() {
        let conn = setup_test_db();
        insert_dummy_task(&conn, "Frontend", "UI Design", "Use egui for components");
        insert_dummy_task(&conn, "Backend", "API Setup", "Database integration");

        // 💡 プロジェクト名（project）での検索
        let results = scan_tasks(&conn, "Front", false).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].project, "Frontend");

        // 💡 詳細欄（detail）での検索
        let results = scan_tasks(&conn, "egui", false).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "UI Design");
    }

    #[test]
    fn test_scan_tasks_no_match() {
        let conn = setup_test_db();
        insert_dummy_task(&conn, "Rust", "Fix a bug", "Detail");

        // 💡 どこにもヒットしないキーワードの場合、空のベクターが返るか
        let results = scan_tasks(&conn, "Python", false).unwrap();
        assert!(results.is_empty());
    }
}
