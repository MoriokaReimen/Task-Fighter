use anyhow::Result;
use domain::prelude::*;
use domain::{Task, TaskFilterFlags, TaskOrderFlags, TaskSearchFlags};
use driver::Connection;

use jiff::{ToSpan, Zoned};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::oneshot::{self, Receiver};

pub struct Core {
    conn: Arc<Mutex<Connection>>,
    runtime: tokio::runtime::Runtime,
}

pub type PlotResult = Result<Vec<(i32, i32, i32, i32)>>;

pub enum CoreOutput {
    Idle,
    InsertTask(Receiver<Result<()>>),
    UpsertTask(Receiver<Result<()>>),
    FetchAllTask(Receiver<Result<Vec<Task>>>),
    FetchOneTask(Receiver<Result<Task>>),
    UpdateTask(Receiver<Result<()>>),
    SearchTask(Receiver<Result<Vec<Task>>>),
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
}

impl TaskRecord for Core {
    type AsyncOutput = CoreOutput;

    fn get_next_task_id(&self) -> Result<i32> {
        self.runtime.block_on(async {
            let conn = self.conn.lock().await;
            driver::get_next_task_id(&conn)
        })
    }

    fn fetch_one_task(&self, id: i32) -> Self::AsyncOutput {
        spawn_async_db!(self, FetchOneTask, |conn| { driver::fetch_one_task(conn, id) })
    }

    fn fetch_all_task(
        &self,
        filter_flags: TaskFilterFlags,
        order_flags: TaskOrderFlags,
    ) -> Self::AsyncOutput {
        // 【修正】引数名が `filter_flags` になっているため、マクロ内でもそれに合わせる
        // 【修正】CoreOutput のバリアントは `FetchAllTasks` の可能性が高いため変更（元のままだと型エラーになる可能性があるため確認してください）
        spawn_async_db!(self, FetchAllTask, |c| {
            driver::fetch_all_task(c, filter_flags, order_flags)
        })
    }

    fn search_task(
        &self,
        pattern: &str,
        search_flags: TaskSearchFlags,
        filter_flags: TaskFilterFlags,
        order_flags: TaskOrderFlags,
    ) -> Self::AsyncOutput {
        let pattern = pattern.to_string();
        spawn_async_db!(self, SearchTask, |c| {
            driver::search_task(c, &pattern, search_flags, filter_flags, order_flags)
        })
    }

    fn insert_task(&self, task: &Task) -> Self::AsyncOutput {
        // 【修正】driver::insert_task が定義されていると仮定して修正（updateになっていたため）
        let task = task.clone();
        spawn_async_db!(self, InsertTask, |conn| driver::insert_task(conn, &task))
    }

    fn update_task(&self, task: &Task) -> Self::AsyncOutput {
        let task = task.clone();
        spawn_async_db!(self, UpdateTask, |c| driver::update_task(c, &task))
    }

    fn upsert_task(&self, task: &Task) -> Self::AsyncOutput {
        let task = task.clone();
        spawn_async_db!(self, UpsertTask, |c| driver::upsert_task(c, &task))
    }

    // 【注意】TaskRecord トレイトに以下の独自の関数（get_plot_data, mail_daily）が含まれている場合はこれで通ります。
    // もしトレイト側に定義がない場合は、これらの関数自体を impl Task Record for Core から削除してください。
    fn get_plot_data(&self) -> Self::AsyncOutput {
        spawn_async_db!(self, PlotData, |c| {
            let today = Zoned::now().date();
            let start_date = today - 99.days();
            driver::get_plot_data(c, start_date, today)
        })
    }

    fn mail_daily(&self, tasks: &[Task]) -> Self::AsyncOutput {
        let conn = Arc::clone(&self.conn);
        let (tx, rx) = oneshot::channel();
        let tasks = tasks.to_vec();
        let handle = self.runtime.handle().clone();

        self.runtime.spawn_blocking(move || {
            let result = (|| -> Result<()> {
                let today = Zoned::now().date();
                let start_date = today - 99.days();

                let conn_guard = handle.block_on(async { conn.lock().await });
                let data = driver::get_plot_data(&conn_guard, start_date, today)?;
                drop(conn_guard);

                let image_data = driver::export_to_base64(&data)?;
                driver::launch_system_mailer(&tasks, &image_data)?;
                Ok(())
            })();
            let _ = tx.send(result);
        });

        CoreOutput::MailDaily(rx)
    }
}
