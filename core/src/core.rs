use anyhow::Result;
use domain::DailyTask;
use domain::MonthlyTask;
use domain::Task;
use domain::WeeklyTask;
use driver::Connection;

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::oneshot::Receiver;

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
    /* Elements for DailyTask  */
    DeleteDailyTask(Receiver<Result<()>>),
    FetchAllDailyTask(Receiver<Result<Vec<DailyTask>>>),
    FetchOneDailyTask(Receiver<Result<DailyTask>>),
    InsertDailyTask(Receiver<Result<()>>),
    SearchDailyTask(Receiver<Result<Vec<DailyTask>>>),
    SyncAllDailyTask(Receiver<Result<()>>),
    UpdateDailyTask(Receiver<Result<()>>),
    UpsertDailyTask(Receiver<Result<()>>),
    /* Elements for WeeklyTask  */
    DeleteWeeklyTask(Receiver<Result<()>>),
    FetchAllWeeklyTask(Receiver<Result<Vec<WeeklyTask>>>),
    FetchOneWeeklyTask(Receiver<Result<WeeklyTask>>),
    InsertWeeklyTask(Receiver<Result<()>>),
    SearchWeeklyTask(Receiver<Result<Vec<WeeklyTask>>>),
    SyncAllWeeklyTask(Receiver<Result<()>>),
    UpdateWeeklyTask(Receiver<Result<()>>),
    UpsertWeeklyTask(Receiver<Result<()>>),
    /* Elements for MonthlyTask  */
    DeleteMonthlyTask(Receiver<Result<()>>),
    FetchAllMonthlyTask(Receiver<Result<Vec<MonthlyTask>>>),
    FetchOneMonthlyTask(Receiver<Result<MonthlyTask>>),
    InsertMonthlyTask(Receiver<Result<()>>),
    SearchMonthlyTask(Receiver<Result<Vec<MonthlyTask>>>),
    SyncAllMonthlyTask(Receiver<Result<()>>),
    UpdateMonthlyTask(Receiver<Result<()>>),
    UpsertMonthlyTask(Receiver<Result<()>>),
}

impl Core {
    /// Initializes a new Core instance with an established database connection.
    pub fn new() -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        let path = driver::DuckdbPath::InDirectory("./runtime".into());

        let conn = driver::connect(&path)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            runtime,
        })
    }
}
