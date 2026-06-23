use anyhow::{Context, Result, bail};
use jiff::Zoned;
use jiff::civil::Date;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub enum Priority {
    #[default]
    Low = 0,
    Medium = 1,
    High = 2,
}

impl TryFrom<i32> for Priority {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Priority::Low),
            1 => Ok(Priority::Medium),
            2 => Ok(Priority::High),
            _ => bail!("Invalid priority integer state: {}", value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TaskStatus {
    #[default]
    Pending = 0,
    WorkInProgress = 1,
    Complete = 2,
}

impl TryFrom<i32> for TaskStatus {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TaskStatus::Pending),
            1 => Ok(TaskStatus::WorkInProgress),
            2 => Ok(TaskStatus::Complete),
            _ => bail!("Invalid task status integer state: {}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: i32,
    pub active: bool,
    pub status: TaskStatus,
    pub project: String,
    pub title: String,
    pub detail: String,
    pub start_date: Date,
    pub due_date: Date,
    pub priority: Priority,
    pub progress: f32,
    pub time_spent: f32,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: 0,
            active: true,
            status: TaskStatus::Pending,
            project: String::new(),
            title: String::new(),
            detail: String::new(),
            start_date: Zoned::now().date(),
            due_date: Zoned::now().date(),
            priority: Priority::Low,
            progress: 0.0,
            time_spent: 0.0,
        }
    }
}

/// Centralized mapper to convert a database row slice into a Task token instance,
/// significantly flattening nesting inside fetch functions.
impl<'a> TryFrom<&'a rusqlite::Row<'a>> for Task {
    type Error = rusqlite::Error;

    fn try_from(row: &'a rusqlite::Row<'a>) -> Result<Self, Self::Error> {
        let active_raw: i32 = row.get(1)?;
        let status_raw: i32 = row.get(2)?;
        let priority_raw: i32 = row.get(8)?;

        let status = TaskStatus::try_from(status_raw).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Integer, e.into())
        })?;
        let priority = Priority::try_from(priority_raw).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(8, rusqlite::types::Type::Integer, e.into())
        })?;

        Ok(Task {
            id: row.get(0)?,
            active: active_raw != 0,
            status,
            project: row.get(3)?,
            title: row.get(4)?,
            detail: row.get(5)?,
            start_date: row.get(6)?,
            due_date: row.get(7)?,
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
        .context("Failed to establish SQLite database file handle stream connection")?;

    // Create tasks master schema structure
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            active      INTEGER NOT NULL DEFAULT 1,
            status      INTEGER NOT NULL DEFAULT 0,
            project     TEXT NOT NULL,
            title       TEXT NOT NULL,
            detail      TEXT NOT NULL,
            start_date  DATETIME NOT NULL,
            due_date    DATETIME NOT NULL,
            priority    INTEGER NOT NULL DEFAULT 1,
            progress    REAL NOT NULL DEFAULT 0.0 CHECK(progress >= 0.0 AND progress <= 100.0),
            time_spent  REAL NOT NULL DEFAULT 0.0
        );",
        [],
    )
    .context("Failed executing target master initialization schema table creation migrations")?;

    // Create tasks full text search indices structures
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS tasks_fts USING fts5(
            title, 
            project,
            detail, 
            content='tasks', 
            content_rowid='id',
            tokenize='trigram' 
        );",
        [],
    )
    .context("Failed executing full-text search layout indices extensions migrations setup")?;

    // FTS Integration triggers hooks definitions pipelines
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS tasks_ai AFTER INSERT ON tasks BEGIN
            INSERT INTO tasks_fts(rowid, title, project, detail) VALUES (new.id, new.title, new.project, new.detail);
        END;",
        [],
    )
    .context("Failed to attach continuous data synchronization hooks for insertion bounds")?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS tasks_au AFTER UPDATE OF title, project, detail ON tasks BEGIN
            INSERT INTO tasks_fts(tasks_fts, rowid, title, project, detail) VALUES('delete', old.id, old.title, old.project, old.detail);
            INSERT INTO tasks_fts(rowid, title, project, detail) VALUES (new.id, new.title, new.project, new.detail);
        END;",
        [],
    ).context("Failed to attach continuous data synchronization hooks for modification bounds")?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS tasks_ad AFTER DELETE ON tasks BEGIN
            INSERT INTO tasks_fts(tasks_fts, rowid, title, project, detail) VALUES('delete', old.id, old.title, old.project, old.detail);
        END;",
        [],
    ).context("Failed to attach continuous data synchronization hooks for removal bounds")?;

    info!("Database and target system schemas synchronized cleanly.");

    Ok(conn)
}

pub fn insert_task(conn: &Connection, task: &Task) -> Result<()> {
    info!("Inserting task record token: {:?}", task);
    conn.execute(
        "INSERT INTO tasks (active, status, project, title, detail, start_date, due_date, priority, progress, time_spent) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            task.active as i32,
            task.status as i32,
            task.project,
            task.title,
            task.detail,
            task.start_date,
            task.due_date,
            task.priority as i32,
            task.progress,
            task.time_spent
        ],
    )
    .context("Failed to commit novel dataset item to relational datastore row bounds")?;
    Ok(())
}

pub fn fetch_all_tasks(conn: &Connection) -> Result<Vec<Task>> {
    info!("Querying all existing task entry items dataset");
    let mut stmt = conn.prepare(
        "SELECT id, active, status, project, title, detail, start_date, due_date, priority, progress, time_spent FROM tasks ORDER BY priority DESC",
    )?;

    let tasks = stmt
        .query_map([], |row| Task::try_from(row))?
        .collect::<Result<Vec<Task>, _>>()
        .context("Failed to extract database mapping parameters sequence loops")?;

    Ok(tasks)
}

pub fn fetch_task_by_id(conn: &Connection, id: i32) -> Result<Task> {
    info!("Querying unique task entry via identifier: {}", id);
    let mut stmt = conn.prepare(
        "SELECT id, active, status, project, title, detail, start_date, due_date, priority, progress, time_spent FROM tasks WHERE id = ?1"
    ).context("Failed compiling relational parameter statements validations queries")?;

    let task = stmt
        .query_row(params![id], |row| Task::try_from(row))
        .with_context(|| format!("Requested unique primary index token record not found or lookup failed: Identity [{}]", id))?;

    Ok(task)
}

pub fn fetch_active_tasks(conn: &Connection) -> Result<Vec<Task>> {
    info!("Querying active tasks sequence state contexts");
    let mut stmt = conn.prepare(
        "SELECT id, active, status, project, title, detail, start_date, due_date, priority, progress, time_spent FROM tasks WHERE active = 1 ORDER BY priority DESC"
    ).context("Failed compiling relational parameter statements validations queries")?;

    let tasks = stmt
        .query_map([], |row| Task::try_from(row))?
        .collect::<Result<Vec<Task>, _>>()
        .context("Failed parsing query sequences lists mapping constraints rows")?;

    Ok(tasks)
}

pub fn fetch_incomplete_tasks(conn: &Connection) -> Result<Vec<Task>> {
    info!("Querying pending items sequence state contexts");
    let mut stmt = conn.prepare(
        "SELECT id, active, status, project, title, detail, start_date, due_date, priority, progress, time_spent FROM tasks WHERE status = 0 OR status = 1  ORDER BY priority DESC"
    ).context("Failed compiling relational parameter statements validations queries")?;

    let tasks = stmt
        .query_map([], |row| Task::try_from(row))?
        .collect::<Result<Vec<Task>, _>>()
        .context("Failed parsing query sequences lists mapping constraints rows")?;

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
            task.active as i32,
            task.status as i32,
            task.project,
            task.title,
            task.detail,
            task.start_date,
            task.due_date,
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

    info!(
        "Database fields mutated cleanly for token constraint identifier: {}",
        task.id
    );
    Ok(())
}

#[allow(unused)]
pub fn scan_tasks_by_fts(conn: &Connection, pattern: &str) -> Result<Vec<Task>> {
    info!(
        "Executing full text query token matches indexing sequence lookup: {}",
        pattern
    );

    let trimmed = pattern.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let mut ret = Vec::new();
    if trimmed.chars().count() >= 3 {
        let mut stmt = conn
            .prepare(
                "SELECT t.id 
             FROM tasks t
             JOIN tasks_fts f ON t.id = f.rowid
             WHERE tasks_fts MATCH ?1
             ORDER BY rank;",
            )
            .context("Failed initializing internal full-text search extensions queries contexts")?;

        let matched_ids = stmt
            .query_map([pattern], |row| row.get::<_, i32>(0))
            .context("Failed processing full text indexing tokens queries execution layers")?
            .collect::<Result<Vec<i32>, rusqlite::Error>>()
            .context("Failed reading tokenized data sequences blocks paths")?;

        for id in matched_ids {
            ret.push(fetch_task_by_id(conn, id)?);
        }
    } else {
        let mut stmt = conn.prepare(
            "SELECT id FROM tasks WHERE title LIKE ?1 OR detail LIKE ?1 OR project LIKE ?1 ORDER BY id DESC;"
        ).context("Failed initializing fallbacks substring patterns comparisons matching templates queries")?;

        let like_pattern = format!("%{}%", trimmed);
        let mut rows = stmt
            .query([like_pattern])
            .context("Failed executing fallback patterns comparison scans query sets")?;

        while let Some(row) = rows.next()? {
            let id = row.get(0)?;
            ret.push(fetch_task_by_id(conn, id)?);
        }
    }

    Ok(ret)
}

pub fn scan_tasks(conn: &Connection, pattern: &str, only_active: bool) -> Result<Vec<Task>> {
    info!("Executing text query partial match lookup: {}", pattern);

    let trimmed = pattern.trim();
    // 検索ワードが空文字の場合は即座に空のベクターを返す
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    // 💡 すべての文字数において、標準の LIKE 演算子による部分一致検索を行う
    let sql = if only_active {
        "SELECT id, active, status, project, title, detail, start_date, due_date, priority, progress, time_spent \
         FROM tasks \
         WHERE (title LIKE ?1 OR detail LIKE ?1 OR project LIKE ?1) AND active = 1 \
         ORDER BY priority DESC;"
    } else {
        "SELECT id, active, status, project, title, detail, start_date, due_date, priority, progress, time_spent \
         FROM tasks \
         WHERE (title LIKE ?1 OR detail LIKE ?1 OR project LIKE ?1) \
         ORDER BY priority DESC;"
    };
    let mut stmt = conn
        .prepare(sql)
        .context("Failed to prepare partial match database query statement")?;

    // 前後に % を付与して部分一致のワイルドカードパターンを作成
    let like_pattern = format!("%{}%", trimmed);

    // 💡 1回のクエリで必要なタスク情報を一括取得し、Task構造体にマッピング
    let tasks = stmt
        .query_map([like_pattern], |row| Task::try_from(row))?
        .collect::<Result<Vec<Task>, _>>()
        .context("Failed parsing query sequences lists mapping constraints rows")?;

    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;
    use rand::RngExt;
    use rand::prelude::IndexedRandom;

    fn setup_in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE tasks (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                active      INTEGER NOT NULL DEFAULT 1,
                status      INTEGER NOT NULL DEFAULT 0,
                project     TEXT NOT NULL,
                title       TEXT NOT NULL,
                detail      TEXT NOT NULL,
                start_date  DATETIME NOT NULL,
                due_date    DATETIME NOT NULL,
                priority    INTEGER NOT NULL DEFAULT 1,
                progress    REAL NOT NULL DEFAULT 0.0 CHECK(progress >= 0.0 AND progress <= 100.0),
                time_spent  REAL NOT NULL DEFAULT 0.0
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS tasks_fts USING fts5(
                title, 
                detail, 
                content='tasks', 
                content_rowid='id',
                tokenize='trigram'
            );",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS tasks_ai AFTER INSERT ON tasks BEGIN
                INSERT INTO tasks_fts(rowid, title, detail) VALUES (new.id, new.title, new.detail);
            END;",
            [],
        )
        .unwrap();

        conn.execute(
        "CREATE TRIGGER IF NOT EXISTS tasks_au AFTER UPDATE OF title, detail ON tasks BEGIN
            INSERT INTO tasks_fts(tasks_fts, rowid, title, detail) VALUES('delete', old.id, old.title, old.detail);
            INSERT INTO tasks_fts(rowid, title, detail) VALUES (new.id, new.title, new.detail);
        END;",
        [],
        ).unwrap();

        conn.execute(
        "CREATE TRIGGER IF NOT EXISTS tasks_ad AFTER DELETE ON tasks BEGIN
            INSERT INTO tasks_fts(tasks_fts, rowid, title, detail) VALUES('delete', old.id, old.title, old.detail);
        END;",
        [],
        ).unwrap();

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

    fn generate_random_tasks(conn: &mut Connection, count: usize) -> Result<()> {
        info!(
            "Generating {} random test mock task entries dataset tokens...",
            count
        );

        let projects = vec!["Core", "UI", "Bugs", "Marketing", "Infra"];
        let nouns = vec![
            "Server", "View", "API", "Button", "Database", "Docs", "Auth", "Login",
        ];
        let verbs = vec![
            " Implementation",
            " Refactoring",
            " Testing",
            " Optimization",
            " Debugging",
        ];
        let details = vec![
            "Requires immediate processing validation channels.",
            "Align implementation with specs constraints. Unit tests are mandatory.",
            "Report status inside weekly synchronous alignment checkpoints.",
            "Analyse tracking error metrics patterns to discover root structural bugs.",
        ];

        let mut rng = rand::rng();
        let tx = conn
            .transaction()
            .context("Failed initialization boundary transactions closures")?;

        // Scope statement constraints separately to prevent continuous lifetime borrowing locks
        {
            let mut stmt = tx.prepare(
                "INSERT INTO tasks (
                    project, title, detail, start_date, due_date, priority, progress, time_spent
                ) VALUES (
                    :project, :title, :detail, :start_date, :due_date, :priority, :progress, :time_spent
                )",
            ).context("Failed preparing transaction batch loops items insertions tokens queries statements")?;

            for i in 0..count {
                let project = projects.choose(&mut rng).unwrap().to_string();
                let title = format!(
                    "{}{cached} #{i}",
                    nouns.choose(&mut rng).unwrap(),
                    cached = verbs.choose(&mut rng).unwrap()
                );
                let detail = details.choose(&mut rng).unwrap().to_string();

                let priority = rng.random_range(1..=3);
                let progress = rng.random_range(0.0..=100.0);
                let time_spent = rng.random_range(0.0..=40.0);

                let start_month = rng.random_range(1..=5);
                let start_day = rng.random_range(1..=25);
                let start_date = format!("2026-0{start_month}-{start_day:02} 09:00:00");
                let due_date = format!(
                    "2026-0{cached}-{start_day:02} 18:00:00",
                    cached = start_month + 1
                );

                stmt.execute(rusqlite::named_params! {
                    ":project": project,
                    ":title": title,
                    ":detail": detail,
                    ":start_date": start_date,
                    ":due_date": due_date,
                    ":priority": priority,
                    ":progress": progress,
                    ":time_spent": time_spent,
                })
                .context("Failed to safely flush iteration token item during loop pipeline")?;
            }
        }

        tx.commit().context(
            "Failed finalizing atomic dataset initialization batch transactions updates",
        )?;
        info!(
            "Completed random benchmark mocking configuration state allocation rows creation safely."
        );
        Ok(())
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

    #[test]
    fn test_scan_tasks_by_fts() {
        let mut conn = setup_in_memory_db();
        let _ = generate_random_tasks(&mut conn, 100000);

        let task1 = create_test_task(
            "Rust Study Milestone Tasks",
            "Continuous verification sequences checkouts loops items.",
        );
        let task2 = create_test_task(
            "Python Automated Scraping Automation Engine Setup",
            "Asynchronous metrics collector targets systems components context pipelines execution.",
        );
        let task3 = create_test_task(
            "Grocery Shopping Routines",
            "Purchase organic validation tokens and secondary structural resources manuals textbook item for Rust Study Milestone Tasks workflows.",
        );

        insert_task(&conn, &task1).unwrap();
        insert_task(&conn, &task2).unwrap();
        insert_task(&conn, &task3).unwrap();

        let matched = scan_tasks_by_fts(&conn, "Study").unwrap();
        assert_eq!(matched.len(), 2);
        assert!(
            matched
                .iter()
                .any(|t| t.title == "Rust Study Milestone Tasks")
        );
        assert!(
            matched
                .iter()
                .any(|t| t.title == "Grocery Shopping Routines")
        );

        let matched = scan_tasks_by_fts(&conn, "python").unwrap();
        assert_eq!(matched.len(), 1);
        assert!(
            matched
                .iter()
                .any(|t| t.title == "Python Automated Scraping Automation Engine Setup")
        );

        let matched = scan_tasks_by_fts(&conn, "Grocery").unwrap();
        assert_eq!(matched.len(), 1);
        assert!(
            matched
                .iter()
                .any(|t| t.title == "Grocery Shopping Routines")
        );
    }

    // テスト用のインメモリ接続とテーブル初期化を行うヘルパー関数
    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE tasks (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                active       INTEGER NOT NULL DEFAULT 1,
                status       INTEGER NOT NULL DEFAULT 0,
                project      TEXT NOT NULL,
                title        TEXT NOT NULL,
                detail       TEXT NOT NULL,
                start_date   DATETIME NOT NULL,
                due_date     DATETIME NOT NULL,
                priority     INTEGER NOT NULL DEFAULT 1,
                progress     REAL NOT NULL DEFAULT 0.0,
                time_spent   REAL NOT NULL DEFAULT 0.0
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
