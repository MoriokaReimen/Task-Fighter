use anyhow::Result;
use driver::{Connection, Task, TaskFilterFlags, TaskOrderFlags};
use jiff::{ToSpan, Zoned};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::oneshot::{self, Receiver};

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

/// tokio::sync::Mutex をブロッキングタスク内で安全に取得・処理するマクロ
macro_rules! spawn_async_db {
    ($self:expr, $output_variant:ident, |$conn:ident| $action:expr) => {{
        let conn = Arc::clone(&$self.conn);
        let (tx, rx) = oneshot::channel();

        // spawn_blocking 内から非同期Mutexにアクセスするためのハンドルを取得
        let handle = $self.runtime.handle().clone();

        $self.runtime.spawn_blocking(move || {
            // ブロッキングスレッド内で非同期Mutexをロックする
            let conn_guard = handle.block_on(async { conn.lock().await });
            let $conn = &*conn_guard;
            let result = $action;

            // スレッドを抜ける前に確実にガードをドロップ（解放）
            drop(conn_guard);

            let _ = tx.send(result);
        });

        CoreOutput::$output_variant(rx)
    }};
}

impl Core {
    /// Initializes a new Core instance with an established database connection.
    pub fn new() -> Result<Core> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        let path = driver::DuckdbPath::InDirectory("./runtime".into());

        let conn = driver::connect(&path)?;
        driver::initialize_periodic_tasks(&conn)?;
        Ok(Core {
            conn: Arc::new(Mutex::new(conn)),
            runtime,
        })
    }

    pub fn get_next_id(&self) -> Result<i32> {
        self.runtime.block_on(async {
            let conn = self.conn.lock().await;
            driver::get_next_id(&conn)
        })
    }

    pub fn upsert_task(&self, task: Task) -> CoreOutput {
        spawn_async_db!(self, UpsertTask, |c| driver::upsert_task(c, &task))
    }

    pub fn fetch_active_tasks(&self) -> CoreOutput {
        spawn_async_db!(self, FetchActiveTasks, |c| {
            let filter_flag = TaskFilterFlags::Active;
            let order_flag = TaskOrderFlags::OrderByPriority | TaskOrderFlags::OrderByDueDate | TaskOrderFlags::Reversed;
            driver::fetch_all_task(c, filter_flag, order_flag)
        })
    }

    pub fn update_task(&self, task: Task) -> CoreOutput {
        spawn_async_db!(self, UpdateTask, |c| driver::update_task(c, &task))
    }

    pub fn scan_tasks(&self, pattern: &str, only_active: bool) -> CoreOutput {
        let pattern = pattern.to_string();
        spawn_async_db!(self, ScanTasks, |c| driver::scan_tasks(
            c,
            &pattern,
            only_active
        ))
    }

    pub fn mail_daily(&self, tasks: &[Task]) -> CoreOutput {
        let conn = Arc::clone(&self.conn);
        let (tx, rx) = oneshot::channel();
        let tasks = tasks.to_vec();
        let handle = self.runtime.handle().clone();

        self.runtime.spawn_blocking(move || {
            let result = (|| -> Result<()> {
                let today = Zoned::now().date();
                let start_date = today - 99.days();

                // ここで安全にブロックしてロックを取得
                let conn_guard = handle.block_on(async { conn.lock().await });
                let data = driver::get_plot_data(&conn_guard, start_date, today)?;
                drop(conn_guard); // 不要になったらすぐ解放

                let image_data = driver::export_to_base64(&data)?;
                driver::launch_system_mailer(&tasks, &image_data)?;
                Ok(())
            })();
            let _ = tx.send(result);
        });

        CoreOutput::MailDaily(rx)
    }

    pub fn get_plot_data(&self) -> CoreOutput {
        spawn_async_db!(self, PlotData, |c| {
            let today = Zoned::now().date();
            let start_date = today - 99.days();
            driver::get_plot_data(c, start_date, today)
        })
    }



}
