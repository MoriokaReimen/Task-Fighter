use core::Config;
use core::Task;

pub struct Work {
    pub core: core::Core,
    pub output: core::CoreOutput,
    pub config: Config,
    pub tasks: Option<Vec<Task>>,
    pub plot_data: Option<Vec<(i32, i32, i32, i32)>>,
    pub task: Task,
    pub start_time: jiff::Zoned,
}

impl Work {
    pub fn new() -> Self {
        Self {
            core: core::Core::new().unwrap(),
            output: core::CoreOutput::Idle,
            config: Config::default(),
            tasks: None,
            plot_data: None,
            task: Task::default(),
            start_time: jiff::Zoned::now(),
        }
    }
}
