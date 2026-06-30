use anyhow::Result;
use driver::{self, Connection, Task};
use jiff::{ToSpan, Zoned};
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Mutex};

pub struct Core {
    conn: Arc<Mutex<Connection>>,
    runtime: tokio::runtime::Runtime,
}

pub type PlotResult = Result<Vec<(i32, i32, i32, i32)>>;

#[allow(dead_code)]
pub enum CoreOutput {
    Idle,
    InsertTask(Receiver<Result<()>>),
    UpsertTask(Receiver<Result<()>>),
    FetchAllTasks(Receiver<Result<Vec<Task>>>),
    FetchTaskById(Receiver<Result<Task>>),
    FetchActiveTasks(Receiver<Result<Vec<Task>>>),
    FetchIncompleteTasks(Receiver<Result<Vec<Task>>>),
    UpdateTask(Receiver<Result<()>>),
    ScanTasks(Receiver<Result<Vec<Task>>>),
    MailDaily(Receiver<Result<()>>),
    PlotData(Receiver<PlotResult>),
}

impl Core {
    /// Initializes a new Core instance with an established database connection.
    pub fn new() -> Result<Core> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let conn = driver::connect()?;
        driver::initialize_periodic_tasks(&conn)?;
        Ok(Core {
            conn: Arc::new(Mutex::new(conn)),
            runtime,
        })
    }

    pub fn get_next_id(&self) -> Result<i32> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("Database mutex lock was poisoned"))?;
        driver::get_next_id(&conn)
    }

    pub fn upsert_task(&self, task: Task) -> CoreOutput {
        let conn = Arc::clone(&self.conn);
        let (tx, rx) = channel();
        self.runtime.spawn(async move {
            let result = conn
                .lock()
                .map_err(|_| anyhow::anyhow!("Database mutex lock was poisoned"))
                .and_then(|conn_lock| driver::upsert_task(&conn_lock, &task)); // &conn_lock を渡す
            let _ = tx.send(result);
        });

        CoreOutput::UpsertTask(rx)
    }

    pub fn fetch_active_tasks(&self) -> CoreOutput {
        let conn = Arc::clone(&self.conn);
        let (tx, rx) = channel();
        self.runtime.spawn(async move {
            let result = conn
                .lock()
                .map_err(|_| anyhow::anyhow!("Database mutex lock was poisoned"))
                .and_then(|conn_lock| driver::fetch_active_tasks(&conn_lock)); // &conn_lock を渡す
            let _ = tx.send(result);
        });
        CoreOutput::FetchActiveTasks(rx)
    }

    pub fn update_task(&self, task: Task) -> CoreOutput {
        let conn = Arc::clone(&self.conn);
        let (tx, rx) = channel();
        self.runtime.spawn(async move {
            let result = conn
                .lock()
                .map_err(|_| anyhow::anyhow!("Database mutex lock was poisoned"))
                .and_then(|conn_lock| driver::update_task(&conn_lock, &task)); // &conn_lock を渡す
            let _ = tx.send(result);
        });

        CoreOutput::UpdateTask(rx)
    }

    pub fn scan_tasks(&self, pattern: &str, only_active: bool) -> CoreOutput {
        let conn = Arc::clone(&self.conn);
        let (tx, rx) = channel();
        let pattern = pattern.to_string();
        self.runtime.spawn(async move {
            let result = conn
                .lock()
                .map_err(|_| anyhow::anyhow!("Database mutex lock was poisoned"))
                .and_then(|conn_lock| driver::scan_tasks(&conn_lock, &pattern, only_active)); // &conn_lock を渡す
            let _ = tx.send(result);
        });

        CoreOutput::ScanTasks(rx)
    }

    pub fn mail_daily(&self, tasks: &[Task]) -> CoreOutput {
        let conn = Arc::clone(&self.conn);
        let (tx, rx) = channel();
        let tasks = tasks.to_vec();
        self.runtime.spawn(async move {
            let today = Zoned::now().date();
            let start_date = today - 99.days();
            let result = (|| -> Result<()> {
                let conn_lock = conn
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Database mutex lock was poisoned"))?;
                let data = driver::count_tasks_by_date(&conn_lock, start_date, today)?; // &conn_lock を渡す
                let image_data = driver::export_to_base64(&data)?;
                driver::launch_system_mailer(&tasks, &image_data)?;
                Ok(())
            })();
            let _ = tx.send(result);
        });
        CoreOutput::MailDaily(rx)
    }

    pub fn get_plot_data(&self) -> CoreOutput {
        let conn = Arc::clone(&self.conn);
        let (tx, rx) = channel();

        self.runtime.spawn(async move {
            let today = Zoned::now().date();
            let start_date = today - 99.days();
            let result = conn
                .lock()
                .map_err(|_| anyhow::anyhow!("Database mutex lock was poisoned"))
                .and_then(|conn_lock| driver::count_tasks_by_date(&conn_lock, start_date, today)); // &conn_lock を渡す
            let _ = tx.send(result);
        });

        CoreOutput::PlotData(rx)
    }
}
