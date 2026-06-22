use anyhow::{Context, Result, bail};
use jiff::Zoned;
use jiff::civil::Date;
use rusqlite::{Connection, params};
use std::fs;
use std::path::Path;
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

// 優先度を表す Enum
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
            _ => anyhow::bail!("不正なTaskStatus値です: {}", value),
        }
    }
}

// データベースのレコードに対応する構造体
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: i32,
    pub active: bool,
    pub status: TaskStatus,
    pub project: String,
    pub title: String,
    pub detail: String,
    pub start_date: Date, // String から Dateに変更
    pub due_date: Date,   // String から Dateに変更
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
            progress: 0.0f32,
            time_spent: 0.0f32,
        }
    }
}

impl Task {
    pub fn new(
        id: i32,
        active: bool,
        status: TaskStatus,
        project: String,
        title: String,
        detail: String,
        start_date: Date,
        due_date: Date,
        priority: Priority,
        progress: f32,
        time_spent: f32,
    ) -> Self {
        Self {
            id,
            active,
            status,
            project,
            title,
            detail,
            start_date,
            due_date,
            priority,
            progress,
            time_spent,
        }
    }
}

pub fn connect() -> Result<Connection> {
    let path = Path::new("runtime");
    if path.exists() && !path.is_dir() {
        bail!("'runtime' はディレクトリではなく、同名のファイルとして既に存在しています。");
    } else {
        fs::create_dir_all(path).context("runtime ディレクトリの作成に失敗しました。")?;
    }

    let conn = Connection::open("./runtime/task_fighter.db")
        .context("データベースファイルのオープンに失敗しました")?;

    // tasks テーブルの作成
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
    .context("テーブルの作成に失敗しました")?;

    // tasks_fts 仮想テーブルの作成
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
    .context("FTS5テーブルの作成に失敗しました")?;

    // 3. データ追加時のトリガー
    // 【修正】カラム指定に rowid を追加し、new.id をバインド
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS tasks_ai AFTER INSERT ON tasks BEGIN
            INSERT INTO tasks_fts(rowid, title, project, detail) VALUES (new.id, new.title, new.project, new.detail);
        END;",
        [],
    )
    .context("INSERTトリガーの作成に失敗しました")?;

    // 4. データ更新時のトリガー
    // 【修正】UPDATE構文の「OR OF」を「OF」に修正
    // 【修正】後半のINSERTで rowid カラムの指定と new.id のバインドを追加
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS tasks_au AFTER UPDATE OF title, detail ON tasks BEGIN
            INSERT INTO tasks_fts(tasks_fts, rowid, title, project, detail) VALUES('delete', old.id, old.title, old.project, old.detail);
            INSERT INTO tasks_fts(rowid, title, project, detail) VALUES (new.id, new.title, new.project, new.detail);
        END;",
        [],
    ).context("UPDATEトリガーの作成に失敗しました")?;

    // 5. データ削除時のトリガー
    // 【修正】最初のカラム指定を `title` ではなく `tasks_fts`（テーブル名と同名の制御カラム）に修正
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS tasks_ad AFTER DELETE ON tasks BEGIN
            INSERT INTO tasks_fts(tasks_fts, rowid, title, project, detail) VALUES('delete', old.id, old.title, old.project, old.detail);
        END;",
        [],
    ).context("DELETEトリガーの作成に失敗しました")?;

    info!("データベースとtasksテーブルが正常に準備されました。");
    Ok(conn)
}

// データ挿入用のヘルパー関数
pub fn insert_task(conn: &Connection, task: &Task) -> Result<()> {
    info!("Insert task: {:?}", task);
    conn.execute(
        "INSERT INTO tasks (active, status, project, title, detail, start_date, due_date, priority, progress, time_spent) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            task.active as i32,
            task.status as i32,
            task.project,
            task.title,
            task.detail,
            task.start_date, // rusqliteのchrono機能により、Dateをそのまま渡せます
            task.due_date,
            task.priority as i32,
            task.progress,
            task.time_spent
        ],
    )
    .context("データの挿入に失敗しました")?;
    Ok(())
}

// データ全件取得用のヘルパー関数
pub fn fetch_all_tasks(conn: &Connection) -> Result<Vec<Task>> {
    info!("Fetch all tasks");
    let mut stmt = conn.prepare(
        "SELECT id, active, status, project, title, detail, start_date, due_date, priority, progress, time_spent FROM tasks",
    )?;

    let task_iter = stmt.query_map([], |row| {
        let active_raw: i32 = row.get(1)?;
        let status_raw: i32 = row.get(2)?;
        let priority_raw: i32 = row.get(8)?;

        Ok((
            row.get::<_, i32>(0)?,
            active_raw != 0,
            status_raw,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, Date>(6)?, // SQLから直接Dateとして取得
            row.get::<_, Date>(7)?, // SQLから直接Dateとして取得
            priority_raw,
            row.get(9)?,
            row.get(10)?,
        ))
    })?;

    let mut tasks = Vec::new();
    for item in task_iter {
        let (
            id,
            active,
            status_raw,
            project,
            title,
            detail,
            start_date,
            due_date,
            p_raw,
            progress,
            time_spent,
        ) = item?;
        tasks.push(Task {
            id,
            active,
            status: TaskStatus::try_from(status_raw)?,
            project,
            title,
            detail,
            start_date,
            due_date,
            priority: Priority::try_from(p_raw)?,
            progress,
            time_spent,
        });
    }

    Ok(tasks)
}

pub fn fetch_task_by_id(conn: &Connection, id: i32) -> Result<Task> {
    info!("Fetch task by id : {}", id);
    let mut stmt = conn
        .prepare("SELECT id, active, status, project, title, detail, start_date, due_date, priority, progress, time_spent FROM tasks WHERE id = ?1")
        .context("クエリの準備に失敗しました")?;

    let row_result = stmt.query_row(params![id], |row| {
        let active_raw: i32 = row.get(1)?;
        let status_raw: i32 = row.get(2)?;
        let priority_raw: i32 = row.get(8)?;
        Ok((
            row.get::<_, i32>(0)?,
            active_raw != 0,
            status_raw,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, Date>(6)?,
            row.get::<_, Date>(7)?,
            priority_raw,
            row.get(9)?,
            row.get(10)?,
        ))
    });

    match row_result {
        Ok(tup) => {
            let (
                id,
                active,
                status_raw,
                project,
                title,
                detail,
                start_date,
                due_date,
                p_raw,
                progress,
                time_spent,
            ) = tup;
            Ok(Task {
                id,
                active,
                status: TaskStatus::try_from(status_raw)?,
                project,
                title,
                detail,
                start_date,
                due_date,
                priority: Priority::try_from(p_raw)?,
                progress,
                time_spent,
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
        .prepare("SELECT id, active, status, project, title, detail, start_date, due_date, priority, progress, time_spent FROM tasks WHERE active = 1")
        .context("クエリの準備に失敗しました")?;

    let task_iter = stmt.query_map([], |row| {
        let active_raw: i32 = row.get(1)?;
        let status_raw: i32 = row.get(2)?;
        let priority_raw: i32 = row.get(8)?;
        Ok((
            row.get::<_, i32>(0)?,
            active_raw != 0,
            status_raw,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, Date>(6)?,
            row.get::<_, Date>(7)?,
            priority_raw,
            row.get(9)?,
            row.get(10)?,
        ))
    })?;

    let mut tasks = Vec::new();
    for item in task_iter {
        let (
            id,
            active,
            status_raw,
            project,
            title,
            detail,
            start_date,
            due_date,
            p_raw,
            progress,
            time_spent,
        ) = item.context("レコードの読み込みに失敗しました")?;
        tasks.push(Task {
            id,
            active,
            status: TaskStatus::try_from(status_raw)?,
            project,
            title,
            detail,
            start_date,
            due_date,
            priority: Priority::try_from(p_raw)?,
            progress,
            time_spent,
        });
    }

    Ok(tasks)
}

// statusがfalse（0）のタスクのみを取得する関数
pub fn fetch_incomplete_tasks(conn: &Connection) -> Result<Vec<Task>> {
    info!("Fetch incomplete tasks");
    let mut stmt = conn
        .prepare("SELECT id, active, status, project, title, detail, start_date, due_date, priority, progress, time_spent FROM tasks WHERE status = 0 OR status = 1")
        .context("クエリの準備に失敗しました")?;

    let task_iter = stmt.query_map([], |row| {
        let active_raw: i32 = row.get(1)?;
        let status_raw: i32 = row.get(2)?;
        let priority_raw: i32 = row.get(8)?;
        Ok((
            row.get::<_, i32>(0)?,
            active_raw != 0,
            status_raw,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, Date>(6)?,
            row.get::<_, Date>(7)?,
            priority_raw,
            row.get(9)?,
            row.get(10)?,
        ))
    })?;

    let mut tasks = Vec::new();
    for item in task_iter {
        let (
            id,
            active,
            status_raw,
            project,
            title,
            detail,
            start_date,
            due_date,
            p_raw,
            progress,
            time_spent,
        ) = item.context("レコードの読み込みに失敗しました")?;
        tasks.push(Task {
            id,
            active,
            status: TaskStatus::try_from(status_raw)?,
            project,
            title,
            detail,
            start_date,
            due_date,
            priority: Priority::try_from(p_raw)?,
            progress,
            time_spent,
        });
    }

    Ok(tasks)
}

// 指定した id のタスク内容を更新する関数
pub fn update_task(conn: &Connection, task: &Task) -> Result<()> {
    let mut stmt = conn
        .prepare(
            "UPDATE tasks 
             SET active = ?1, status = ?2, project = ?3, title = ?4, detail = ?5, start_date = ?6, due_date = ?7, priority = ?8, progress = ?9, time_spent = ?10 
             WHERE id = ?11",
        )
        .context("クエリの準備に失敗しました")?;

    let rows_affected = stmt
        .execute(params![
            task.active as i32,
            task.status as i32,
            task.project,
            task.title,
            task.detail,
            task.start_date, // Dateを直接バインド
            task.due_date,   // Dateを直接バインド
            task.priority as i32,
            task.progress,
            task.time_spent,
            task.id
        ])
        .context("データの更新に失敗しました")?;

    if rows_affected == 0 {
        anyhow::bail!("指定されたID ({}) のタスクが見つかりませんでした", task.id);
    }

    info!("ID: {} のタスクを正常に更新しました。", task.id);
    Ok(())
}

pub fn scan_tasks_by_fts(conn: &Connection, pattern: &str) -> Result<Vec<Task>> {
    info!("Scan tasks with FTS5 pattern : {}", pattern);

    // pattern が空文字列の場合は、エラーを避けるため空の配列を返すか、
    // もしくは全件取得などの処理に分岐させてください
    let trimmed = pattern.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let char_count = trimmed.chars().count();
    let mut ret = Vec::new();
    if char_count >= 3 {
        // tasks と tasks_fts を結合（JOIN）し、MATCH 演算子で高速検索します。
        // rank でソートすることで、関連度が高い順（マッチ数が多い等）に並び替えます。
        let mut stmt = conn
            .prepare(
                "SELECT t.id 
                FROM tasks t
                JOIN tasks_fts f ON t.id = f.rowid
                WHERE tasks_fts MATCH ?1
                ORDER BY rank;",
            )
            .context("FTS5クエリの準備に失敗しました")?;

        // マッチしたIDのリストを一括で取得
        let matched_ids = stmt
            .query_map([pattern], |row| row.get::<_, i32>(0))
            .context("FTS5クエリの実行に失敗しました")?
            .collect::<Result<Vec<i32>, rusqlite::Error>>()
            .context("レコードの読み込みに失敗しました")?;

        // 各IDに対応する詳細なTaskオブジェクトを取得
        for id in matched_ids {
            let task = fetch_task_by_id(conn, id)?;
            ret.push(task);
        }
    } else {
        // ==========================================
        // 【1〜2文字】通常の LIKE 句で部分一致検索
        // ==========================================
        let mut stmt = conn
            .prepare(
                "SELECT id, title, project, detail
                 FROM tasks
                 WHERE title LIKE ?1 OR detail LIKE ?1 OR project LIKE ?1
                 ORDER BY id DESC;",
            )
            .context("LIKEクエリの準備に失敗しました")?;

        let like_pattern = format!("%{}%", trimmed);

        let mut rows = stmt
            .query([like_pattern])
            .context("LIKEクエリの実行に失敗しました")?;
        while let Some(row) = rows.next()? {
            let id = row.get(0)?;
            let task = fetch_task_by_id(conn, id)?;
            ret.push(task);
        }
    }

    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;
    use rand::RngExt;

    // 共通で利用するインメモリDB初期化ヘルパー
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
        .unwrap();

        // tasks_fts 仮想テーブルの作成
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

        // 3. データ追加時のトリガー
        // 【修正】カラム指定に rowid を追加し、new.id をバインド
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS tasks_ai AFTER INSERT ON tasks BEGIN
            INSERT INTO tasks_fts(rowid, title, detail) VALUES (new.id, new.title, new.detail);
        END;",
            [],
        )
        .unwrap();

        // 4. データ更新時のトリガー
        // 【修正】UPDATE構文の「OR OF」を「OF」に修正
        // 【修正】後半のINSERTで rowid カラムの指定と new.id のバインドを追加
        conn.execute(
        "CREATE TRIGGER IF NOT EXISTS tasks_au AFTER UPDATE OF title, detail ON tasks BEGIN
            INSERT INTO tasks_fts(tasks_fts, rowid, title, detail) VALUES('delete', old.id, old.title, old.detail);
            INSERT INTO tasks_fts(rowid, title, detail) VALUES (new.id, new.title, new.detail);
        END;",
        [],
        ).unwrap();

        // 5. データ削除時のトリガー
        // 【修正】最初のカラム指定を `title` ではなく `tasks_fts`（テーブル名と同名の制御カラム）に修正
        conn.execute(
        "CREATE TRIGGER IF NOT EXISTS tasks_ad AFTER DELETE ON tasks BEGIN
            INSERT INTO tasks_fts(tasks_fts, rowid, title, detail) VALUES('delete', old.id, old.title, old.detail);
        END;",
        [],
        ).unwrap();

        conn
    }

    // テスト用のダミータスクを生成するヘルパー
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
        info!("{} 個のランダムなタスクデータを生成中...", count);

        // テスト用のダミー単語リスト（日本語検索テスト用）
        let projects = vec![
            "基盤開発",
            "UI改善",
            "バグ修正",
            "マーケティング",
            "インフラ構築",
        ];
        let nouns = vec![
            "サーバー",
            "画面",
            "API",
            "ボタン",
            "データベース",
            "ドキュメント",
            "認証",
            "ログイン",
        ];
        let verbs = vec![
            "の実装",
            "のリファクタリング",
            "のテスト",
            "の見直し",
            "の最適化",
            "のデバッグ",
        ];
        let details = vec![
            "至急対応する必要があります。進捗が遅れているため要確認。",
            "要件定義書に沿って実装を進めてください。テストコードも必須です。",
            "週次の定例ミーティングで進捗を報告してください。進捗率は高めを維持。",
            "不具合の報告が上がっているため、ログを解析して原因を特定すること。",
            "ドキュメントの作成も並行して行ってください。完了条件を満たすこと。",
        ];

        let mut rng = rand::rng();

        // 💡 重要: 高速化のためにトランザクションを開始
        let tx = conn
            .transaction()
            .context("トランザクションの開始に失敗しました")?;

        {
            // ループ内で再利用するプリペアドステートメントを準備
            let mut stmt = tx
                .prepare(
                    "INSERT INTO tasks (
                project, title, detail, start_date, due_date, priority, progress, time_spent
            ) VALUES (
                :project, :title, :detail, :start_date, :due_date, :priority, :progress, :time_spent
            )",
                )
                .context("INSERTステートメントの準備に失敗しました")?;

            for i in 0..count {
                // ランダムな組み合わせでタイトルを生成（例: "サーバーのリファクタリング #452"）
                let project = projects.choose(&mut rng).unwrap();
                let title = format!(
                    "{}{} #{}",
                    nouns.choose(&mut rng).unwrap(),
                    verbs.choose(&mut rng).unwrap(),
                    i
                );
                let detail = details.choose(&mut rng).unwrap().to_string();

                // ランダムな進捗と優先度
                let priority = rng.random_range(1..=5); // 1〜5
                let progress = rng.random_range(0.0..=100.0);
                let time_spent = rng.random_range(0.0..=40.0);

                // 簡易的な日付文字列の生成（2026年の適当な日付）
                let start_month = rng.random_range(1..=6);
                let start_day = rng.random_range(1..=28);
                let start_date = format!("2026-0{:01}-{:02} 09:00:00", start_month, start_day);
                let due_date = format!("2026-0{:01}-{:02} 18:00:00", start_month + 1, start_day);

                // パラメータをバインドして実行
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
                .context(format!("{} 件目のデータ挿入に失敗しました", i))?;
            }
        } // stmt のスコープをここで終わらせて借用を解除する

        // 💡 最後にコミットして一気にディスクへ書き込む
        tx.commit()
            .context("トランザクションのコミットに失敗しました")?;

        info!("{} 個のタスクデータの生成が正常に完了しました。", count);
        Ok(())
    }

    // --- Priority Enum のテスト ---

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

    // --- Task 構造体のテスト ---

    #[test]
    fn test_task_default() {
        let task = Task::default();
        assert_eq!(task.id, 0);
        assert!(task.active);
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.priority, Priority::Low);
        assert_eq!(task.progress, 0.0);
    }

    // --- データベース操作（CRUD）のテスト ---

    #[test]
    fn test_insert_and_fetch_task_by_id() {
        let conn = setup_in_memory_db();
        let mut task = create_test_task("新規タスク", "詳細文");

        // 挿入テスト
        insert_task(&conn, &task).unwrap();

        // 1件取得テスト (AUTOINCREMENTによりIDは1になる)
        let fetched = fetch_task_by_id(&conn, 1).unwrap();
        assert_eq!(fetched.id, 1);
        assert_eq!(fetched.title, "新規タスク");
        assert_eq!(fetched.detail, "詳細文");
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

        // アクティブなタスクの検証 (task1, task3)
        let active_tasks = fetch_active_tasks(&conn).unwrap();
        assert_eq!(active_tasks.len(), 2);
        assert!(active_tasks.iter().any(|t| t.title == "T1"));
        assert!(active_tasks.iter().any(|t| t.title == "T3"));

        // 未完了のタスクの検証 (task1, task2)
        let incomplete_tasks = fetch_incomplete_tasks(&conn).unwrap();
        assert_eq!(incomplete_tasks.len(), 2);
        assert!(incomplete_tasks.iter().any(|t| t.title == "T1"));
        assert!(incomplete_tasks.iter().any(|t| t.title == "T2"));
    }

    #[test]
    fn test_update_task() {
        let conn = setup_in_memory_db();
        let task = create_test_task("更新前", "詳細");
        insert_task(&conn, &task).unwrap();

        // 既存タスクを取得して書き換え
        let mut to_update = fetch_task_by_id(&conn, 1).unwrap();
        to_update.title = "更新後".to_string();
        to_update.status = TaskStatus::Complete;
        to_update.progress = 100.0;

        update_task(&conn, &to_update).unwrap();

        // 反映されているか検証
        let updated = fetch_task_by_id(&conn, 1).unwrap();
        assert_eq!(updated.title, "更新後");
        assert_eq!(task.status, TaskStatus::WorkInProgress);
        assert_eq!(updated.progress, 100.0);
    }

    #[test]
    fn test_update_task_not_found() {
        let conn = setup_in_memory_db();
        let mut task = create_test_task("存在しない", "タスク");
        task.id = 999; // 存在しないID

        let result = update_task(&conn, &task);
        assert!(result.is_err());
    }

    // --- 正規表現検索のテスト ---

    #[test]
    fn test_scan_tasks_by_fts() {
        let mut conn = setup_in_memory_db();
        generate_random_tasks(&mut conn, 100000);
        let task1 = create_test_task("Rustの勉強", "毎日コミットする");
        let task2 = create_test_task("Pythonスクリプト作成", "自動化ツールの開発");
        let task3 = create_test_task("お買い物", "牛乳とRustの勉強の本を買う");

        insert_task(&conn, &task1).unwrap();
        insert_task(&conn, &task2).unwrap();
        insert_task(&conn, &task3).unwrap();

        // "Rust" を含むタスクを検索 (境界や大文字小文字を考慮)
        let matched = scan_tasks_by_fts(&conn, "の勉強").unwrap();
        assert_eq!(matched.len(), 2);
        assert!(matched.iter().any(|t| t.title == "Rustの勉強"));
        assert!(matched.iter().any(|t| t.title == "お買い物"));

        let matched = scan_tasks_by_fts(&conn, "python").unwrap();
        assert_eq!(matched.len(), 1);
        assert!(matched.iter().any(|t| t.title == "Pythonスクリプト作成"));

        let matched = scan_tasks_by_fts(&conn, "牛乳").unwrap();
        assert_eq!(matched.len(), 1);
        assert!(matched.iter().any(|t| t.title == "お買い物"));
    }
}
