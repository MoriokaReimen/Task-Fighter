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

pub trait Page {
    fn show(&mut self, ui: &mut egui::Ui, work: &mut crate::work::Work) -> Pages;
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pages {
    #[default]
    Main,
    EditTask,
    CreateTask,
    Graph,
    Timer,
}
