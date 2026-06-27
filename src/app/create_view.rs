use super::main_app::{App, AppState};
use crate::app::task_edit::TaskEdit;
use crate::app::yes_no_popup::PopupResult;
use crate::core::CoreOutput;
use crate::driver::Task;
use crate::fl;
use eframe::egui::{self, Align, Button, Layout, Ui, vec2};
use tracing::info;

impl App {
    /// Renders the task creation view inside separate action and workspace panels.
    pub fn create_view(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        // --- Bottom Action Bar ---
        egui::Panel::bottom("bottom_panel").show_inside(ui, |ui: &mut Ui| {
            // Right-to-left layout automatically places items horizontally without nested horizontal blocks
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Cancel Action
                if ui
                    .add(Button::new(fl!("close")).min_size(vec2(90.0, 28.0)))
                    .clicked()
                {
                    info!("Cancel Button Pressed");
                    self.temp_task = Task::default();
                    self.state = AppState::Default;
                    self.displayed_tasks = None;
                }

                // Save Action
                if ui
                    .add(Button::new(fl!("save")).min_size(vec2(90.0, 28.0)))
                    .clicked()
                {
                    info!("Save Button Pressed");
                    self.yes_no_popup
                        .open(fl!("save-task"), fl!("save-task-message"));
                }
                if self.yes_no_popup.show(ui) == PopupResult::Yes {
                    if self.temp_task.is_saveable() {
                        let task_to_insert = self.temp_task.clone();
                        self.output = self.core.upsert_task(task_to_insert);
                    } else {
                        let message = if self.temp_task.project.is_empty() {
                            fl!("project-empty")
                        } else {
                            fl!("title-empty")
                        };
                        self.warning_popup.open(fl!("save-error"), message);
                    }
                }
                self.warning_popup.show(ui);
            });
        });

        // --- Main Form Panel ---
        egui::CentralPanel::default().show_inside(ui, |ui: &mut Ui| {
            ui.heading(fl!("create-task"));
            if !matches!(self.output, CoreOutput::Idle) {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        // サイズを大きく設定（例: 64.0 ポイント）して表示
                        ui.add(egui::Spinner::new().size(64.0));
                    },
                );
                return;
            }
            // Render input form with minimized layout nesting depth
            let mut task_edit = TaskEdit::new(&mut self.temp_task);
            task_edit.show(ui);
        });
    }
}
