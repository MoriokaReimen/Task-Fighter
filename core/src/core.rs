use anyhow::Result;
use domain::DailyTask;
use domain::MonthlyTask;
use domain::Task;
use domain::WeeklyTask;
use driver::Connection;
use std::path::PathBuf;

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::oneshot::Receiver;
use tracing::info;

pub struct Core {
    pub(crate) conn: Arc<Mutex<Connection>>,
    pub(crate) runtime: tokio::runtime::Runtime,
}

pub type PlotResult = Result<Vec<(i32, i32, i32, i32)>>;

#[derive(Debug)]
pub enum CoreOutput {
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

        let config_dir_path = get_config_dir_path()?;
        info!("Set config directory path to {}", config_dir_path.display());
        let path = driver::DuckdbPath::InDirectory(config_dir_path);

        let conn = driver::connect(&path)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            runtime,
        })
    }
}

pub fn get_config_dir_path() -> Result<PathBuf, std::io::Error> {
    let mut config_dir_path = dirs::config_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Failed to find the OS configuration directory",
        )
    })?;

    config_dir_path.push("task-fighter");

    Ok(config_dir_path)
}
