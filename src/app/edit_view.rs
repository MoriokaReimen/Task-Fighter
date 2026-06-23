use super::main_app::{App, AppState};
use crate::app::task_edit::TaskEdit;
use crate::driver::Task;
use eframe::egui::{self, Align, Button, Layout, Ui, vec2};
use tracing::info;

impl App {
    /// Renders the task editing view inside a dedicated panel setup.
    pub fn edit_view(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        // --- Bottom Action Bar ---
        egui::Panel::bottom("bottom_panel").show_inside(ui, |ui: &mut Ui| {
            // Right-to-left layout places buttons from rightmost to leftmost
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Cancel Button Action
                if ui
                    .add(Button::new("❌ Cancel").min_size(vec2(90.0, 28.0)))
                    .clicked()
                {
                    info!("Cancel Button Pressed");
                    self.temp_task = Task::default();
                    self.state = AppState::Default;
                    self.displayed_tasks = None;
                }

                // Save Button Action
                if ui
                    .add(Button::new("💾 Save").min_size(vec2(90.0, 28.0)))
                    .clicked()
                {
                    info!("Save Button Pressed");
                    self.output = self.core.update_task(self.temp_task.clone());
                    self.state = AppState::Default;
                    self.displayed_tasks = None;
                    self.temp_task = Task::default();
                }
            });
        });

        // --- Main Form Content ---
        egui::CentralPanel::default().show_inside(ui, |ui: &mut Ui| {
            ui.heading("✏ Edit Task");
            let mut task_edit = TaskEdit::new(&mut self.temp_task);
            task_edit.show(ui);
        });
    }
}
