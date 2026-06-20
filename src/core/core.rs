use super::super::driver;
use anyhow::Result;
use driver::Task;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

pub struct Core {
    conn: Arc<Mutex<Connection>>,
}

pub enum CoreOutput {
    Idle,
    InsertTask(oneshot::Receiver<Result<()>>),
    FetchAllTasks(oneshot::Receiver<Result<Vec<Task>>>),
    FetchTaskById(oneshot::Receiver<Result<Task>>),
    FetchActiveTasks(oneshot::Receiver<Result<Vec<Task>>>),
    FetchIncompleteTasks(oneshot::Receiver<Result<Vec<Task>>>),
    UpdateTask(oneshot::Receiver<Result<()>>),
    ScanTasksByRegex(oneshot::Receiver<Result<Vec<Task>>>),
    MailDaily(oneshot::Receiver<Result<()>>),
}

impl Core {
    pub fn new() -> Result<Core> {
        let conn = driver::connect()?;
        Ok(Core {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn insert_task(&self, task: Task) -> CoreOutput {
        let conn = Arc::clone(&self.conn);
        let (tx, rx) = oneshot::channel();
        tokio::task::spawn_blocking(move || {
            let res = (|| {
                let conn_lock = conn
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Mutexのロックに失敗しました"))?;
                driver::insert_task(&conn_lock, &task)
            })();
            let _ = tx.send(res);
        });

        CoreOutput::InsertTask(rx)
    }

    pub fn fetch_all_tasks(&self) -> CoreOutput {
        let conn = Arc::clone(&self.conn);
        let (tx, rx) = oneshot::channel();

        tokio::task::spawn_blocking(move || {
            let res = (|| {
                let conn_lock = conn
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Mutexのロックに失敗しました"))?;
                driver::fetch_all_tasks(&conn_lock)
            })();
            let _ = tx.send(res);
        });

        CoreOutput::FetchAllTasks(rx)
    }

    pub fn fetch_task_by_id(&self, id: i32) -> CoreOutput {
        let conn = Arc::clone(&self.conn);
        let (tx, rx) = oneshot::channel();

        tokio::task::spawn_blocking(move || {
            let res = (|| {
                let conn_lock = conn
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Mutexのロックに失敗しました"))?;
                driver::fetch_task_by_id(&conn_lock, id)
            })();
            let _ = tx.send(res);
        });

        CoreOutput::FetchTaskById(rx)
    }

    pub fn fetch_active_tasks(&self) -> CoreOutput {
        let conn = Arc::clone(&self.conn);
        let (tx, rx) = oneshot::channel();

        tokio::task::spawn_blocking(move || {
            let res = (|| {
                let conn_lock = conn
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Mutexのロックに失敗しました"))?;
                driver::fetch_active_tasks(&conn_lock)
            })();
            let _ = tx.send(res);
        });

        CoreOutput::FetchActiveTasks(rx)
    }

    pub fn fetch_incomplete_tasks(&self) -> CoreOutput {
        let conn = Arc::clone(&self.conn);
        let (tx, rx) = oneshot::channel();

        tokio::task::spawn_blocking(move || {
            let res = (|| {
                let conn_lock = conn
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Mutexのロックに失敗しました"))?;
                driver::fetch_incomplete_tasks(&conn_lock)
            })();
            let _ = tx.send(res);
        });

        CoreOutput::FetchIncompleteTasks(rx)
    }

    pub fn update_task(&self, task: Task) -> CoreOutput {
        let conn = Arc::clone(&self.conn);
        let (tx, rx) = oneshot::channel();

        tokio::task::spawn_blocking(move || {
            let res = (|| {
                let conn_lock = conn
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Mutexのロックに失敗しました"))?;
                driver::update_task(&conn_lock, &task)
            })();
            let _ = tx.send(res);
        });

        CoreOutput::UpdateTask(rx)
    }

    pub fn scan_tasks_by_regex(&self, pattern: &String) -> CoreOutput {
        let conn = Arc::clone(&self.conn);
        let pattern = pattern.clone();
        let (tx, rx) = oneshot::channel();

        tokio::task::spawn_blocking(move || {
            let res = (|| {
                let conn_lock = conn
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Mutexのロックに失敗しました"))?;
                driver::scan_tasks_by_regex(&conn_lock, &pattern)
            })();
            let _ = tx.send(res);
        });

        CoreOutput::ScanTasksByRegex(rx)
    }

    pub fn mail_daily(&self, tasks: Vec<Task>) -> CoreOutput {
        let conn = Arc::clone(&self.conn);
        let (tx, rx) = oneshot::channel();

        tokio::task::spawn_blocking(move || {
            let res = (|| {
                let _conn_lock = conn
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Mutexのロックに失敗しました"))?;
                driver::launch_system_mailer(&tasks)?;
                Ok(())
            })();
            let _ = tx.send(res);
        });

        CoreOutput::MailDaily(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::{Priority, Task};
    use crate::*;
    use chrono::{TimeZone, Utc};

    // テスト用のダミータスクを生成するヘルパー
    fn create_test_task(id: i32, title: &str, done: bool, active: bool) -> Task {
        let now = Utc.with_ymd_and_hms(2026, 6, 20, 9, 0, 0).unwrap();
        Task::new(
            id,
            active,
            done,
            "TestProject".to_string(),
            title.to_string(),
            "Detail contents".to_string(),
            now,
            now,
            Priority::Medium,
        )
    }

    // メモリ内DBを使ってCoreインスタンスを作成するヘルパー
    // driver::connect() が外部ファイルを固定で開く仕様である場合、
    // テスト実行時の排他ロック競合を防ぐため、テスト用DBコネクションを自前でセットアップします。
    fn setup_test_core() -> Core {
        let conn = Connection::open_in_memory().unwrap();
        // driver側と同じテーブル定義をテスト用メモリ内DBに作成
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
        .unwrap();

        Core {
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    #[tokio::test]
    async fn test_core_insert_and_fetch_all() -> Result<()> {
        let core = setup_test_core();
        let task = create_test_task(0, "Learn Rust Async", false, true);

        // 1. タスクを挿入
        if let CoreOutput::InsertTask(rx) = core.insert_task(task) {
            // oneshotチャネル受信のエラー(?)と、DB操作自体のエラー(?)を二重にアンラップ
            rx.await??;
        } else {
            panic!("不正なCoreOutputバリアントが返されました");
        }

        // 2. 全件取得
        if let CoreOutput::FetchAllTasks(rx) = core.fetch_all_tasks() {
            let tasks = rx.await??;
            assert_eq!(tasks.len(), 1);
            assert_eq!(tasks[0].title, "Learn Rust Async");
            assert_eq!(tasks[0].id, 1); // AUTOINCREMENTによるインデックス
        } else {
            panic!("不正なCoreOutputバリアントが返されました");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_core_fetch_task_by_id() -> Result<()> {
        let core = setup_test_core();
        let task = create_test_task(0, "Target Task", false, true);

        if let CoreOutput::InsertTask(rx) = core.insert_task(task) {
            rx.await??;
        }

        // ID: 1 のタスクを指定して取得
        if let CoreOutput::FetchTaskById(rx) = core.fetch_task_by_id(1) {
            let fetched = rx.await??;
            assert_eq!(fetched.title, "Target Task");
        } else {
            panic!("不正なCoreOutputバリアントが返されました");
        }

        // 存在しないIDを指定した場合の検証
        if let CoreOutput::FetchTaskById(rx) = core.fetch_task_by_id(999) {
            let result = rx.await?; // チャネル自体の受信は成功するが
            assert!(result.is_err()); // 中身のDB処理はエラーになる
        } else {
            panic!("不正なCoreOutputバリアントが返されました");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_core_fetch_active_and_incomplete() -> Result<()> {
        let core = setup_test_core();

        // 未完了/アクティブなタスク
        let task1 = create_test_task(0, "Active Incomplete", false, true);
        // 完了済みタスク
        let task2 = create_test_task(0, "Completed", true, true);

        if let CoreOutput::InsertTask(rx) = core.insert_task(task1) {
            rx.await??;
        }
        if let CoreOutput::InsertTask(rx) = core.insert_task(task2) {
            rx.await??;
        }

        // 未完了タスクのみの取得を検証
        if let CoreOutput::FetchIncompleteTasks(rx) = core.fetch_incomplete_tasks() {
            let incomplete = rx.await??;
            assert_eq!(incomplete.len(), 1);
            assert_eq!(incomplete[0].title, "Active Incomplete");
        } else {
            panic!("不正なCoreOutputバリアントが返されました");
        }

        // アクティブタスクのみの取得を検証
        if let CoreOutput::FetchActiveTasks(rx) = core.fetch_active_tasks() {
            let active = rx.await??;
            assert_eq!(active.len(), 2);
        } else {
            panic!("不正なCoreOutputバリアントが返されました");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_core_update_task() -> Result<()> {
        let core = setup_test_core();
        let task = create_test_task(0, "Before Update", false, true);

        if let CoreOutput::InsertTask(rx) = core.insert_task(task) {
            rx.await??;
        }

        // ID: 1 のタスクを書き換えるデータを用意
        let updated_task = create_test_task(1, "After Update", true, false);

        if let CoreOutput::UpdateTask(rx) = core.update_task(updated_task) {
            rx.await??;
        } else {
            panic!("不正なCoreOutputバリアントが返されました");
        }

        // 実際に書き換わっているかロードして検証
        if let CoreOutput::FetchTaskById(rx) = core.fetch_task_by_id(1) {
            let fetched = rx.await??;
            assert_eq!(fetched.title, "After Update");
            assert_eq!(fetched.done, true);
            assert_eq!(fetched.active, false);
        } else {
            panic!("不正なCoreOutputバリアントが返されました");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_core_scan_tasks_by_regex() -> Result<()> {
        let core = setup_test_core();
        let task1 = create_test_task(0, "Rust Programming", false, true);
        let task2 = create_test_task(0, "Python Scripting", false, true);

        if let CoreOutput::InsertTask(rx) = core.insert_task(task1) {
            rx.await??;
        }
        if let CoreOutput::InsertTask(rx) = core.insert_task(task2) {
            rx.await??;
        }

        // 正規表現スキャンのテスト
        if let CoreOutput::ScanTasksByRegex(rx) = core.scan_tasks_by_regex("Rust".to_string()) {
            let matched_ids = rx.await??;
            assert_eq!(matched_ids, vec![1]);
        } else {
            panic!("不正なCoreOutputバリアントが返されました");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_core_mail_daily_flow() -> Result<()> {
        let core = setup_test_core();
        let task = create_test_task(0, "Daily Report Task", false, true);
        if let CoreOutput::InsertTask(rx) = core.insert_task(task) {
            rx.await??;
        }

        // mail_daily の呼び出しとクラッシュしないことの検証
        // ※システムメーラー起動は環境に依存するため、Result成否にかかわらずパニックを起こさないかをチェックします。
        if let CoreOutput::MailDaily(rx) = core.mail_daily() {
            let _ = rx.await?;
        } else {
            panic!("不正なCoreOutputバリアントが返されました");
        }

        Ok(())
    }
}
