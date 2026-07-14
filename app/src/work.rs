use crate::page::Pages;
use core::Config;
use core::{DailyTask, MonthlyTask, Task, WeeklyTask};

pub struct Work {
    pub core: core::Core,
    pub outputs: Vec<core::CoreOutput>,
    pub config: Config,
    pub tasks: Option<Vec<Task>>,
    pub daily_tasks: Option<Vec<DailyTask>>,
    pub weekly_tasks: Option<Vec<WeeklyTask>>,
    pub monthly_tasks: Option<Vec<MonthlyTask>>,
    pub plot_data: Option<Vec<(i32, i32, i32, i32)>>,
    pub task: Task,
    pub daily_task: DailyTask,
    pub weekly_task: WeeklyTask,
    pub monthly_task: MonthlyTask,
    pub start_time: jiff::Zoned,

    pub next_page: Pages,
    pub last_page: Pages,
}

impl Work {
    pub fn new() -> Self {
        Self {
            core: core::Core::new().unwrap(),
            outputs: Vec::new(),
            config: Config::default(),
            tasks: None,
            daily_tasks: None,
            weekly_tasks: None,
            monthly_tasks: None,
            plot_data: None,
            task: Task::default(),
            daily_task: DailyTask::default(),
            weekly_task: WeeklyTask::default(),
            monthly_task: MonthlyTask::default(),
            start_time: jiff::Zoned::now(),
            next_page: Pages::Main,
            last_page: Pages::Init,
        }
    }
}
