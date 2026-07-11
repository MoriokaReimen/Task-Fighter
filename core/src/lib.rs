#[allow(clippy::module_inception)]
mod core;
pub use core::Core;
pub use core::CoreOutput;
pub use domain::TaskRecord;
pub use domain::{ColorScheme, Config};
pub use domain::{DailyTask, MonthlyTask, WeeklyTask};
pub use domain::{DailyTaskFilterFlags, DailyTaskOrderFlags, DailyTaskSearchFlags};
pub use domain::{Task, TaskPriority, TaskStatus};
pub use domain::{TaskFilterFlags, TaskOrderFlags, TaskSearchFlags};
pub use tokio::sync::oneshot::error::TryRecvError;

mod config_record;
mod daily_task_record;
mod monthly_task_record;
mod task_record;
mod weekly_task_record;
pub mod prelude {
    pub use domain::DailyTaskRecord;
    pub use domain::MonthlyTaskRecord;
    pub use domain::TaskRecord;
    pub use domain::WeeklyTaskRecord;
    pub use domain::config_record::ConfigRecord;
}
