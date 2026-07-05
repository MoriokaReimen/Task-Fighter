mod task;
pub use task::{Task, TaskPriority, TaskStatus};

mod daily_task;
pub use daily_task::DailyTask;

mod weekly_task;
pub use weekly_task::WeeklyTask;

mod monthly_task;
pub use monthly_task::MonthlyTask;

mod task_record;
pub use task_record::{TaskFilterFlags, TaskOrderFlags, TaskRecord, TaskSearchFlags, PlotResult};

mod monthly_task_record;
pub use monthly_task_record::{
    MonthlyTaskFilterFlags, MonthlyTaskOrderFlags, MonthlyTaskRecord, MonthlyTaskSearchFlags,
};

mod daily_task_record;
pub use daily_task_record::{
    DailyTaskFilterFlags, DailyTaskOrderFlags, DailyTaskRecord, DailyTaskSearchFlags,
};

mod weekly_task_record;
pub use weekly_task_record::{
    WeeklyTaskFilterFlags, WeeklyTaskOrderFlags, WeeklyTaskRecord, WeeklyTaskSearchFlags,
};
pub mod prelude {
    pub use crate::daily_task_record::DailyTaskRecord;
    pub use crate::monthly_task_record::MonthlyTaskRecord;
    pub use crate::task_record::TaskRecord;
    pub use crate::weekly_task_record::WeeklyTaskRecord;
}
