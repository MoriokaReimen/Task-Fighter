use anyhow::Result;
use domain::prelude::*;
use domain::{Task, TaskFilterFlags, TaskOrderFlags, TaskSearchFlags};
use driver::Connection;

use jiff::{ToSpan, Zoned};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::oneshot::{self, Receiver};

pub struct Core {
    pub(crate) conn: Arc<Mutex<Connection>>,
    pub(crate) runtime: tokio::runtime::Runtime,
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

