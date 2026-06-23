use crate::driver::{self, Task};
use anyhow::Result;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

pub struct Core {
    conn: Arc<Mutex<Connection>>,
}

#[allow(dead_code)]
pub enum CoreOutput {
    Idle,
    InsertTask(oneshot::Receiver<Result<()>>),
    FetchAllTasks(oneshot::Receiver<Result<Vec<Task>>>),
    FetchTaskById(oneshot::Receiver<Result<Task>>),
    FetchActiveTasks(oneshot::Receiver<Result<Vec<Task>>>),
    FetchIncompleteTasks(oneshot::Receiver<Result<Vec<Task>>>),
    UpdateTask(oneshot::Receiver<Result<()>>),
    ScanTasksByFts(oneshot::Receiver<Result<Vec<Task>>>),
    ScanTasks(oneshot::Receiver<Result<Vec<Task>>>),
    MailDaily(oneshot::Receiver<Result<()>>),
}

/// Helper macro to reduce async database locking boilerplate and flatten nesting depth.
macro_rules! execute_blocking {
    ($self:expr, $tx:expr, $action:expr) => {{
        let conn = Arc::clone(&$self.conn);
        tokio::task::spawn_blocking(move || {
            let res = conn
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to acquire database mutex lock"))
                .and_then(|conn_lock| $action(&conn_lock));
            let _ = $tx.send(res);
        });
    }};
}

impl Core {
    /// Initializes a new Core instance with an established database connection.
    pub fn new() -> Result<Core> {
        let conn = driver::connect()?;
        driver::initialize_periodic_tasks(&conn)?;
        Ok(Core {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn insert_task(&self, task: Task) -> CoreOutput {
        let (tx, rx) = oneshot::channel();
        execute_blocking!(self, tx, |conn_lock| driver::insert_task(conn_lock, &task));
        CoreOutput::InsertTask(rx)
    }

    #[allow(dead_code)]
    pub fn fetch_all_tasks(&self) -> CoreOutput {
        let (tx, rx) = oneshot::channel();
        execute_blocking!(self, tx, driver::fetch_all_tasks);
        CoreOutput::FetchAllTasks(rx)
    }

    #[allow(dead_code)]
    pub fn fetch_task_by_id(&self, id: i32) -> CoreOutput {
        let (tx, rx) = oneshot::channel();
        execute_blocking!(self, tx, |conn_lock| driver::fetch_task_by_id(
            conn_lock, id
        ));
        CoreOutput::FetchTaskById(rx)
    }

    pub fn fetch_active_tasks(&self) -> CoreOutput {
        let (tx, rx) = oneshot::channel();
        execute_blocking!(self, tx, driver::fetch_active_tasks);
        CoreOutput::FetchActiveTasks(rx)
    }

    #[allow(dead_code)]
    pub fn fetch_incomplete_tasks(&self) -> CoreOutput {
        let (tx, rx) = oneshot::channel();
        execute_blocking!(self, tx, driver::fetch_incomplete_tasks);
        CoreOutput::FetchIncompleteTasks(rx)
    }

    pub fn update_task(&self, task: Task) -> CoreOutput {
        let (tx, rx) = oneshot::channel();
        execute_blocking!(self, tx, |conn_lock| driver::update_task(conn_lock, &task));
        CoreOutput::UpdateTask(rx)
    }

    #[allow(dead_code)]
    pub fn scan_tasks_by_fts(&self, pattern: &str) -> CoreOutput {
        let (tx, rx) = oneshot::channel();
        let pattern = pattern.to_string();
        execute_blocking!(self, tx, |conn_lock| driver::scan_tasks_by_fts(
            conn_lock, &pattern
        ));
        CoreOutput::ScanTasksByFts(rx)
    }

    pub fn scan_tasks(&self, pattern: &str, only_active: bool) -> CoreOutput {
        let (tx, rx) = oneshot::channel();
        let pattern = pattern.to_string();
        execute_blocking!(self, tx, |conn_lock| driver::scan_tasks(
            conn_lock, &pattern, only_active
        ));
        CoreOutput::ScanTasks(rx)
    }

    pub fn mail_daily(&self, tasks: Vec<Task>) -> CoreOutput {
        let (tx, rx) = oneshot::channel();
        execute_blocking!(self, tx, |_conn_lock| driver::launch_system_mailer(&tasks));
        CoreOutput::MailDaily(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::TaskStatus;
    use jiff::civil::date;

    /// Helper function to construct dummy task tokens for test state evaluations.
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
        // Initialize Core instance (automatically boots SQLite connection)
        let core = Core::new().expect("Failed to initialize Core context");

        // 1. Evaluate Task Insertion logic
        let task = create_test_task(0, "Async Test Task");
        let output = core.insert_task(task);

        let CoreOutput::InsertTask(rx) = output else {
            panic!("Unexpected CoreOutput variant encountered");
        };

        let result = rx
            .await
            .expect("Oneshot communication channel severed prematurely");
        assert!(
            result.is_ok(),
            "Task insertion failed execution pipeline: {:?}",
            result.err()
        );

        // 2. Query inserted items and validate database entry existence
        let output = core.fetch_all_tasks();
        let CoreOutput::FetchAllTasks(rx) = output else {
            panic!("Unexpected CoreOutput variant encountered");
        };

        let result = rx
            .await
            .expect("Oneshot communication channel severed prematurely");
        let tasks = result.expect("Failed to safely resolve all database tasks");

        assert!(
            !tasks.is_empty(),
            "Database dataset returned empty after item insertion"
        );
        assert!(
            tasks.iter().any(|t| t.title == "Async Test Task"),
            "Target mock task token not found"
        );
    }

    #[tokio::test]
    async fn test_core_fetch_task_by_id_not_found() {
        let core = Core::new().expect("Failed to initialize Core context");

        // Request a explicitly non-existent row entry identifier (9999)
        let output = core.fetch_task_by_id(9999);
        let CoreOutput::FetchTaskById(rx) = output else {
            panic!("Unexpected CoreOutput variant encountered");
        };

        let result = rx
            .await
            .expect("Oneshot communication channel severed prematurely");
        assert!(
            result.is_err(),
            "Expected lookup failure for non-existent primary key identifier"
        );
    }

    #[tokio::test]
    async fn test_core_update_task_not_found() {
        let core = Core::new().expect("Failed to initialize Core context");
        let task = create_test_task(9999, "Non-existent Task Entry");

        // Evaluate modification handling constraints against a missing identity record
        let output = core.update_task(task);
        let CoreOutput::UpdateTask(rx) = output else {
            panic!("Unexpected CoreOutput variant encountered");
        };

        let result = rx
            .await
            .expect("Oneshot communication channel severed prematurely");
        assert!(
            result.is_err(),
            "Expected execution update constraints violation fallback error"
        );
    }

    #[tokio::test]
    async fn test_core_scan_tasks_by_fts() {
        let core = Core::new().expect("Failed to initialize Core context");

        // Evaluate operational errors handling with malformed regex constraints inputs
        let invalid_pattern = "[A-Z".to_string();
        let output = core.scan_tasks_by_fts(&invalid_pattern);

        let CoreOutput::ScanTasksByFts(rx) = output else {
            panic!("Unexpected CoreOutput variant encountered");
        };

        let result = rx
            .await
            .expect("Oneshot communication channel severed prematurely");
        assert!(
            result.is_err(),
            "Malformed execution query syntax did not yield anticipated panic fallback bounds"
        );
    }
}
