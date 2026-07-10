use core::{CoreOutput, Task};

pub(crate) struct Work {
    pub output: core::CoreOutput,
    pub displayed_tasks: Option<Vec<Task>>,
    pub plot_data: Option<Vec<(i32, i32, i32, i32)>>,
    pub task: Task,
    pub start_time: jiff::Zoned,
}

impl Work {
    fn new() -> Self {
        Self {
            output: core::CoreOutput::Idle,
            displayed_tasks: None,
            plot_data: None,
            task: Task::default(),
            start_time: jiff::Zoned::now(),
        }
    }
}
