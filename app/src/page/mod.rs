mod create_task_page;
pub use create_task_page::*;
mod main_page;
pub use main_page::*;
mod edit_task_page;
pub use edit_task_page::*;
mod graph_page;
pub use graph_page::*;
mod timer_page;
pub use timer_page::*;

mod daily_main_page;
pub use daily_main_page::*;
mod edit_daily_task_page;
pub use edit_daily_task_page::*;
mod create_daily_task_page;
pub use create_daily_task_page::*;

mod weekly_main_page;
pub use weekly_main_page::*;
mod edit_weekly_task_page;
pub use edit_weekly_task_page::*;
mod create_weekly_task_page;
pub use create_weekly_task_page::*;

mod monthly_main_page;
pub use monthly_main_page::*;

mod edit_monthly_task_page;
pub use edit_monthly_task_page::*;

mod create_monthly_task_page;
pub use create_monthly_task_page::*;

mod kanban_page;
pub use kanban_page::*;

pub trait Page {
    fn on_entry(&mut self, work: &mut crate::work::Work);
    fn show(&mut self, ui: &mut egui::Ui, work: &mut crate::work::Work);
    fn on_exit(&mut self, work: &mut crate::work::Work);
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pages {
    #[default]
    Init,

    Main,
    EditTask,
    CreateTask,

    DailyMain,
    EditDailyTask,
    CreateDailyTask,

    WeeklyMain,
    EditWeeklyTask,
    CreateWeeklyTask,

    MonthlyMain,
    EditMonthlyTask,
    CreateMonthlyTask,

    Kanban,
    Graph,
    Timer,
}
