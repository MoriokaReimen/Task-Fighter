#[allow(clippy::module_inception)]
mod core;
pub use core::Core;
pub use core::CoreOutput;
pub use domain::TaskRecord;
pub use domain::{Task, TaskPriority, TaskStatus};
pub use tokio::sync::oneshot::error::TryRecvError;

pub mod prelude {
    pub use domain::DailyTaskRecord;
    pub use domain::MonthlyTaskRecord;
    pub use domain::TaskRecord;
    pub use domain::WeeklyTaskRecord;
}
