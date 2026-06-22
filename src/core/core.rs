use crate::driver::{self, Task};
use anyhow::Result;
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
    ScanTasksByFts(oneshot::Receiver<Result<Vec<Task>>>),
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

    pub fn scan_tasks_by_fts(&self, pattern: &String) -> CoreOutput {
        let conn = Arc::clone(&self.conn);
        let pattern = pattern.clone();
        let (tx, rx) = oneshot::channel();

        tokio::task::spawn_blocking(move || {
            let res = (|| {
                let conn_lock = conn
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Mutexのロックに失敗しました"))?;
                driver::scan_tasks_by_fts(&conn_lock, &pattern)
            })();
            let _ = tx.send(res);
        });

        CoreOutput::ScanTasksByFts(rx)
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
    use jiff::civil::date;

    // テスト用のダミータスクを生成するヘルパー関数
    fn create_test_task(id: i32, title: &str) -> Task {
        Task {
            id,
            active: true,
            status: TaskStatus::Pending,
            project: "TestProject".to_string(),
            title: title.to_string(),
            detail: "Test Detail".to_string(),
            start_date: date(2026, 6, 21),
            due_date: date(2026, 6, 30),
            priority: driver::Priority::Low,
            progress: 0.0,
            time_spent: 0.0,
        }
    }

    #[tokio::test]
    async fn test_core_insert_and_fetch_task() {
        // Coreの初期化 (driver::connect() が走り、DBが準備される)
        let core = Core::new().expect("Coreの初期化に失敗しました");

        // 1. タスクの挿入テスト
        let task = create_test_task(0, "非同期テストタスク");
        let output = core.insert_task(task);

        if let CoreOutput::InsertTask(rx) = output {
            let result = rx.await.expect("Channel が正常に閉じられませんでした");
            assert!(
                result.is_ok(),
                "タスクの挿入に失敗しました: {:?}",
                result.err()
            );
        } else {
            panic!("予期しない CoreOutput タイプです");
        }

        // 2. 挿入したタスクを全件取得して検証
        let output = core.fetch_all_tasks();
        if let CoreOutput::FetchAllTasks(rx) = output {
            let result = rx.await.expect("Channel が正常に閉じられませんでした");
            let tasks = result.expect("タスクの全件取得に失敗しました");

            // 少なくとも1件（今入れたタスク）が存在することを確認
            assert!(!tasks.is_empty());
            assert!(tasks.iter().any(|t| t.title == "非同期テストタスク"));
        } else {
            panic!("予期しない CoreOutput タイプです");
        }
    }

    #[tokio::test]
    async fn test_core_fetch_task_by_id_not_found() {
        let core = Core::new().expect("Coreの初期化に失敗しました");

        // 存在しないID (9999) を指定して取得を試みる
        let output = core.fetch_task_by_id(9999);
        if let CoreOutput::FetchTaskById(rx) = output {
            let result = rx.await.expect("Channel が正常に閉じられませんでした");
            // driver 側でエラー(bail!)を返すため、is_err() になるはず
            assert!(result.is_err());
        } else {
            panic!("予期しない CoreOutput タイプです");
        }
    }

    #[tokio::test]
    async fn test_core_update_task_not_found() {
        let core = Core::new().expect("Coreの初期化に失敗しました");
        let mut task = create_test_task(9999, "存在しないタスク");

        // 存在しないIDのタスクを更新
        let output = core.update_task(task);
        if let CoreOutput::UpdateTask(rx) = output {
            let result = rx.await.expect("Channel が正常に閉じられませんでした");
            // 該当レコードがないためエラーになるはず
            assert!(result.is_err());
        } else {
            panic!("予期しない CoreOutput タイプです");
        }
    }

    #[tokio::test]
    async fn test_core_scan_tasks_by_fts() {
        let core = Core::new().expect("Coreの初期化に失敗しました");

        // 検索パターン（不正な正規表現）によるエラーハンドリングのテスト
        let invalid_pattern = "[A-Z".to_string();
        let output = core.scan_tasks_by_fts(&invalid_pattern);

        if let CoreOutput::ScanTasksByFts(rx) = output {
            let result = rx.await.expect("Channel が正常に閉じられませんでした");
            assert!(result.is_err(), "不正な正規表現でエラーになりませんでした");
        } else {
            panic!("予期しない CoreOutput タイプです");
        }
    }
}
