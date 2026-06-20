use anyhow::{Context, Result};
use jiff::Zoned;
use jiff::civil::Date;
use regex::Regex;
use rusqlite::{Connection, params};
use tracing::info;

// 優先度を表す Enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
            _ => anyhow::bail!("不正なプライオリティ値です: {}", value),
        }
    }
}

// データベースのレコードに対応する構造体
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: i32,
    pub active: bool,
    pub done: bool,
    pub project: String,
    pub title: String,
    pub detail: String,
    pub start_date: Date, // String から Dateに変更
    pub due_date: Date,   // String から Dateに変更
    pub priority: Priority,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: 0,
            active: true,
            done: false,
            project: String::new(),
            title: String::new(),
            detail: String::new(),
            start_date: Zoned::now().date(),
            due_date: Zoned::now().date(),
            priority: Priority::Low,
        }
    }
}

impl Task {
    pub fn new(
        id: i32,
        active: bool,
        done: bool,
        project: String,
        title: String,
        detail: String,
        start_date: Date,
        due_date: Date,
        priority: Priority,
    ) -> Self {
        Self {
            id,
            active,
            done,
            project,
            title,
            detail,
            start_date,
            due_date,
            priority,
        }
    }
}

pub fn connect() -> Result<Connection> {
    let conn = Connection::open("./runtime/task_fighter.db")
        .context("データベースファイルのオープンに失敗しました")?;

    // start_date, due_date は SQLite の DATETIME 型（テキスト表現、またはISO 8601互換形式）として保存します
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            active      INTEGER NOT NULL DEFAULT 1,
            done        INTEGER NOT NULL DEFAULT 0,
            project     TEXT NOT NULL,
            title       TEXT NOT NULL,
            detail      TEXT NOT NULL,
            start_date  DATETIME NOT NULL,
            due_date    DATETIME NOT NULL,
            priority    INTEGER NOT NULL DEFAULT 1
        )",
        [],
    )
    .context("テーブルの作成に失敗しました")?;
    info!("データベースとtasksテーブルが正常に準備されました。");
    Ok(conn)
}

// データ挿入用のヘルパー関数
pub fn insert_task(conn: &Connection, task: &Task) -> Result<()> {
    info!("Insert task: {:?}", task);
    conn.execute(
        "INSERT INTO tasks (active, done, project, title, detail, start_date, due_date, priority) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            task.active as i32,
            task.done as i32,
            task.project,
            task.title,
            task.detail,
            task.start_date, // rusqliteのchrono機能により、Dateをそのまま渡せます
            task.due_date,
            task.priority as i32
        ],
    )
    .context("データの挿入に失敗しました")?;
    Ok(())
}

// データ全件取得用のヘルパー関数
pub fn fetch_all_tasks(conn: &Connection) -> Result<Vec<Task>> {
    info!("Fetch all tasks");
    let mut stmt = conn.prepare(
        "SELECT id, active, done, project, title, detail, start_date, due_date, priority FROM tasks",
    )?;

    let task_iter = stmt.query_map([], |row| {
        let active_raw: i32 = row.get(1)?;
        let done_raw: i32 = row.get(2)?;
        let priority_raw: i32 = row.get(8)?;

        Ok((
            row.get::<_, i32>(0)?,
            active_raw != 0,
            done_raw != 0,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, Date>(6)?, // SQLから直接Dateとして取得
            row.get::<_, Date>(7)?, // SQLから直接Dateとして取得
            priority_raw,
        ))
    })?;

    let mut tasks = Vec::new();
    for item in task_iter {
        let (id, active, done, project, title, detail, start_date, due_date, p_raw) = item?;
        tasks.push(Task {
            id,
            active,
            done,
            project,
            title,
            detail,
            start_date,
            due_date,
            priority: Priority::try_from(p_raw)?,
        });
    }

    Ok(tasks)
}

pub fn fetch_task_by_id(conn: &Connection, id: i32) -> Result<Task> {
    info!("Fetch task by id : {}", id);
    let mut stmt = conn
        .prepare("SELECT id, active, done, project, title, detail, start_date, due_date, priority FROM tasks WHERE id = ?1")
        .context("クエリの準備に失敗しました")?;

    let row_result = stmt.query_row(params![id], |row| {
        let active_raw: i32 = row.get(1)?;
        let done_raw: i32 = row.get(2)?;
        let priority_raw: i32 = row.get(8)?;
        Ok((
            row.get::<_, i32>(0)?,
            active_raw != 0,
            done_raw != 0,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, Date>(6)?,
            row.get::<_, Date>(7)?,
            priority_raw,
        ))
    });

    match row_result {
        Ok(tup) => {
            let (id, active, done, project, title, detail, start_date, due_date, p_raw) = tup;
            Ok(Task {
                id,
                active,
                done,
                project,
                title,
                detail,
                start_date,
                due_date,
                priority: Priority::try_from(p_raw)?,
            })
        }
        Err(e) => {
            anyhow::bail!(
                "指定されたID ({}) のタスクが見つからなかったか、取得に失敗しました: {}",
                id,
                e
            );
        }
    }
}

pub fn fetch_active_tasks(conn: &Connection) -> Result<Vec<Task>> {
    info!("Fetch active tasks");
    let mut stmt = conn
        .prepare("SELECT id, active, done, project, title, detail, start_date, due_date, priority FROM tasks WHERE active = 1")
        .context("クエリの準備に失敗しました")?;

    let task_iter = stmt.query_map([], |row| {
        let active_raw: i32 = row.get(1)?;
        let done_raw: i32 = row.get(2)?;
        let priority_raw: i32 = row.get(8)?;
        Ok((
            row.get::<_, i32>(0)?,
            active_raw != 0,
            done_raw != 0,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, Date>(6)?,
            row.get::<_, Date>(7)?,
            priority_raw,
        ))
    })?;

    let mut tasks = Vec::new();
    for item in task_iter {
        let (id, active, done, project, title, detail, start_date, due_date, p_raw) =
            item.context("レコードの読み込みに失敗しました")?;
        tasks.push(Task {
            id,
            active,
            done,
            project,
            title,
            detail,
            start_date,
            due_date,
            priority: Priority::try_from(p_raw)?,
        });
    }

    Ok(tasks)
}

// doneがfalse（0）のタスクのみを取得する関数
pub fn fetch_incomplete_tasks(conn: &Connection) -> Result<Vec<Task>> {
    info!("Fetch incomplete tasks");
    let mut stmt = conn
        .prepare("SELECT id, active, done, project, title, detail, start_date, due_date, priority FROM tasks WHERE done = 0")
        .context("クエリの準備に失敗しました")?;

    let task_iter = stmt.query_map([], |row| {
        let active_raw: i32 = row.get(1)?;
        let done_raw: i32 = row.get(2)?;
        let priority_raw: i32 = row.get(8)?;
        Ok((
            row.get::<_, i32>(0)?,
            active_raw != 0,
            done_raw != 0,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, Date>(6)?,
            row.get::<_, Date>(7)?,
            priority_raw,
        ))
    })?;

    let mut tasks = Vec::new();
    for item in task_iter {
        let (id, active, done, project, title, detail, start_date, due_date, p_raw) =
            item.context("レコードの読み込みに失敗しました")?;
        tasks.push(Task {
            id,
            active,
            done,
            project,
            title,
            detail,
            start_date,
            due_date,
            priority: Priority::try_from(p_raw)?,
        });
    }

    Ok(tasks)
}

// 指定した id のタスク内容を更新する関数
pub fn update_task(conn: &Connection, task: &Task) -> Result<()> {
    let mut stmt = conn
        .prepare(
            "UPDATE tasks 
             SET active = ?1, done = ?2, project = ?3, title = ?4, detail = ?5, start_date = ?6, due_date = ?7, priority = ?8 
             WHERE id = ?9",
        )
        .context("クエリの準備に失敗しました")?;

    let rows_affected = stmt
        .execute(params![
            task.active as i32,
            task.done as i32,
            task.project,
            task.title,
            task.detail,
            task.start_date, // Dateを直接バインド
            task.due_date,   // Dateを直接バインド
            task.priority as i32,
            task.id
        ])
        .context("データの更新に失敗しました")?;

    if rows_affected == 0 {
        anyhow::bail!("指定されたID ({}) のタスクが見つかりませんでした", task.id);
    }

    info!("ID: {} のタスクを正常に更新しました。", task.id);
    Ok(())
}

// 指定された正規表現パターンで title または detail をスキャンし、マッチした id のリストを返す
pub fn scan_tasks_by_regex(conn: &Connection, pattern: &str) -> Result<Vec<Task>> {
    info!("Scan tasks with pattern : {}", pattern);
    let re = Regex::new(pattern).context(format!("不正な正規表現パターンです: {}", pattern))?;

    let mut stmt = conn
        .prepare("SELECT id, title, detail FROM tasks")
        .context("クエリの準備に失敗しました")?;

    let rows_iter = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;

    let mut matched_ids = Vec::new();

    for row in rows_iter {
        let (id, title, detail) = row.context("レコードの読み込みに失敗しました")?;

        if re.is_match(&title) || re.is_match(&detail) {
            matched_ids.push(id);
        }
    }

    let mut ret = Vec::new();
    for id in matched_ids {
        let task = fetch_task_by_id(conn, id)?;
        ret.push(task)
    }

    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use rusqlite::Connection;

    // テスト用のメモリ内DBをセットアップするヘルパー関数
    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE tasks (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                active        INTEGER NOT NULL DEFAULT 1,
                done        INTEGER NOT NULL DEFAULT 0,
                project     TEXT NOT NULL,
                title       TEXT NOT NULL,
                detail      TEXT NOT NULL,
                start_date  DATETIME NOT NULL,
                due_date    DATETIME NOT NULL,
                priority    INTEGER NOT NULL DEFAULT 1
            )",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_priority_try_from() {
        assert_eq!(Priority::try_from(0).unwrap(), Priority::Low);
        assert_eq!(Priority::try_from(1).unwrap(), Priority::Medium);
        assert_eq!(Priority::try_from(2).unwrap(), Priority::High);
        assert!(Priority::try_from(3).is_err());
        assert!(Priority::try_from(-1).is_err());
    }

    #[test]
    fn test_insert_and_fetch_all_tasks() -> Result<()> {
        let conn = setup_test_db();

        // 基準となる日時を生成
        let start = Utc.with_ymd_and_hms(2026, 6, 1, 9, 0, 0).unwrap();
        let due = Utc.with_ymd_and_hms(2026, 6, 10, 18, 0, 0).unwrap();

        let task = Task::new(
            0,
            true,
            false,
            "Project A".to_string(),
            "Task Title".to_string(),
            "Task Detail".to_string(),
            start,
            due,
            Priority::High,
        );

        insert_task(&conn, &task)?;

        let tasks = fetch_all_tasks(&conn)?;
        assert_eq!(tasks.len(), 1);

        let fetched = &tasks[0];
        assert_eq!(fetched.id, 1); // AUTOINCREMENTにより1から始まる
        assert_eq!(fetched.active, true);
        assert_eq!(fetched.done, false);
        assert_eq!(fetched.project, "Project A");
        assert_eq!(fetched.title, "Task Title");
        assert_eq!(fetched.detail, "Task Detail");
        assert_eq!(fetched.start_date, start);
        assert_eq!(fetched.due_date, due);
        assert_eq!(fetched.priority, Priority::High);

        Ok(())
    }

    #[test]
    fn test_fetch_task_by_id() -> Result<()> {
        let conn = setup_test_db();
        let now = Utc::now();

        let task = Task::new(
            0,
            true,
            false,
            "Proj".to_string(),
            "Title".to_string(),
            "Detail".to_string(),
            now,
            now,
            Priority::Low,
        );
        insert_task(&conn, &task)?;

        // 存在するIDの取得
        let fetched = fetch_task_by_id(&conn, 1)?;
        assert_eq!(fetched.id, 1);

        // 存在しないIDの取得はエラーになるか確認
        let missing = fetch_task_by_id(&conn, 999);
        assert!(missing.is_err());

        Ok(())
    }

    #[test]
    fn test_fetch_active_tasks() -> Result<()> {
        let conn = setup_test_db();
        let now = Utc::now();

        // 未完了タスク
        let task1 = Task::new(
            0,
            false,
            false,
            "P".to_string(),
            "T1".to_string(),
            "D".to_string(),
            now,
            now,
            Priority::Medium,
        );
        // 完了済みタスク
        let task2 = Task::new(
            0,
            true,
            true,
            "P".to_string(),
            "T2".to_string(),
            "D".to_string(),
            now,
            now,
            Priority::Medium,
        );

        // 完了済みタスク
        let task3 = Task::new(
            0,
            false,
            true,
            "P".to_string(),
            "T3".to_string(),
            "D".to_string(),
            now,
            now,
            Priority::Medium,
        );

        // 完了済みタスク
        let task4 = Task::new(
            0,
            true,
            true,
            "P".to_string(),
            "T4".to_string(),
            "D".to_string(),
            now,
            now,
            Priority::Medium,
        );

        insert_task(&conn, &task1)?;
        insert_task(&conn, &task2)?;
        insert_task(&conn, &task3)?;
        insert_task(&conn, &task4)?;

        let active_tasks = fetch_active_tasks(&conn)?;
        assert_eq!(active_tasks.len(), 2);
        assert_eq!(active_tasks[0].title, "T2");
        assert_eq!(active_tasks[1].title, "T4");

        Ok(())
    }

    #[test]
    fn test_fetch_incomplete_tasks() -> Result<()> {
        let conn = setup_test_db();
        let now = Utc::now();

        // 未完了タスク
        let task1 = Task::new(
            0,
            true,
            false,
            "P".to_string(),
            "T1".to_string(),
            "D".to_string(),
            now,
            now,
            Priority::Medium,
        );
        // 完了済みタスク
        let task2 = Task::new(
            0,
            true,
            true,
            "P".to_string(),
            "T2".to_string(),
            "D".to_string(),
            now,
            now,
            Priority::Medium,
        );

        insert_task(&conn, &task1)?;
        insert_task(&conn, &task2)?;

        let incomplete_tasks = fetch_incomplete_tasks(&conn)?;
        assert_eq!(incomplete_tasks.len(), 1);
        assert_eq!(incomplete_tasks[0].title, "T1");

        Ok(())
    }

    #[test]
    fn test_update_task() -> Result<()> {
        let conn = setup_test_db();
        let now = Utc::now();

        let task = Task::new(
            0,
            true,
            false,
            "Proj".to_string(),
            "Old Title".to_string(),
            "Detail".to_string(),
            now,
            now,
            Priority::Low,
        );
        insert_task(&conn, &task)?;

        // ID: 1のタスクを更新するデータを作成
        let updated_time = Utc.with_ymd_and_hms(2026, 12, 31, 23, 59, 59).unwrap();
        let updated_task = Task::new(
            1, // 既存のIDを指定
            false,
            true,
            "New Proj".to_string(),
            "New Title".to_string(),
            "New Detail".to_string(),
            now,
            updated_time,
            Priority::High,
        );

        update_task(&conn, &updated_task)?;

        // 反映確認
        let fetched = fetch_task_by_id(&conn, 1)?;
        assert_eq!(fetched.active, false);
        assert_eq!(fetched.done, true);
        assert_eq!(fetched.project, "New Proj");
        assert_eq!(fetched.title, "New Title");
        assert_eq!(fetched.due_date, updated_time);
        assert_eq!(fetched.priority, Priority::High);

        // 存在しないIDの更新はエラーになるか確認
        let mut invalid_task = fetched.clone();
        invalid_task.id = 999;
        assert!(update_task(&conn, &invalid_task).is_err());

        Ok(())
    }

    #[test]
    fn test_scan_tasks_by_regex() -> Result<()> {
        let conn = setup_test_db();
        let now = Utc::now();

        let task1 = Task::new(
            0,
            true,
            false,
            "P".to_string(),
            "Rustを勉強する".to_string(),
            "あいうえお".to_string(),
            now,
            now,
            Priority::Medium,
        );
        let task2 = Task::new(
            0,
            true,
            false,
            "P".to_string(),
            "料理をする".to_string(),
            "SQLiteの設定".to_string(),
            now,
            now,
            Priority::Medium,
        );

        insert_task(&conn, &task1)?;
        insert_task(&conn, &task2)?;

        // タイトルにマッチ
        let matched_rust = scan_tasks_by_regex(&conn, r"Rust")?;
        assert_eq!(matched_rust, vec![1]);

        // 詳細にマッチ
        let matched_sql = scan_tasks_by_regex(&conn, r"SQLite")?;
        assert_eq!(matched_sql, vec![2]);

        // どちらにもマッチしない
        let matched_none = scan_tasks_by_regex(&conn, r"Python")?;
        assert!(matched_none.is_empty());

        // 不正な正規表現のときはエラーが返るか確認
        assert!(scan_tasks_by_regex(&conn, r"[Unclosed-bracket").is_err());

        Ok(())
    }
}
