use super::main_app::{App, AppState};
use crate::app::task_table::TaskTable;
use crate::core::CoreOutput;
use eframe::egui::{self, Align, Button, Color32, Layout, ScrollArea, Ui, vec2};
use tracing::info;

impl App {
    /// Renders the default task list dashboard view.
    pub fn default_view(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        // Trigger automatic tasks fetch if system is idle and no tasks are stored yet
        if matches!(self.output, CoreOutput::Idle) && self.displayed_tasks.is_none() {
            self.output = self.core.fetch_active_tasks();
        }

        // --- Bottom Action Panel ---
        egui::Panel::bottom("bottom_panel").show_inside(ui, |ui: &mut Ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(Button::new("➕ Create New").min_size(vec2(110.0, 28.0)))
                    .clicked()
                {
                    self.state = AppState::Create;
                }

                if ui
                    .add(Button::new("📧 Email Report").min_size(vec2(120.0, 28.0)))
                    .clicked()
                {
                    info!("Email Report Button Pressed");
                    if let Some(ref tasks) = self.displayed_tasks {
                        self.output = self.core.mail_daily(tasks.clone());
                    }
                }
            });
        });

        // --- Central Dashboard Content ---
        egui::CentralPanel::default().show_inside(ui, |ui: &mut Ui| {
            ui.heading("📋 Task List");

            // Search Control Bar Layout
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                if ui
                    .add(Button::new("↩ Reset").min_size(vec2(80.0, 28.0)))
                    .clicked()
                {
                    info!("Reset Button Pressed");
                    self.output = self.core.fetch_active_tasks();
                }

                if ui
                    .add(Button::new("🔍 Search").min_size(vec2(80.0, 28.0)))
                    .clicked()
                {
                    info!("Search Button Pressed");
                    self.output = self.core.scan_tasks(&self.scan_pattern, self.only_active);
                }
                ui.checkbox(&mut self.only_active, "");
                ui.label("Only Active");

                ui.add(
                    egui::TextEdit::singleline(&mut self.scan_pattern)
                        .desired_width(ui.available_width()),
                );
            });

            // Scrollable Workspace Panels
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                        self.render_task_list_content(ui);
                    });
                });
        });
    }

    /// Extracted helper to process and render list entries or empty state placeholders.
    fn render_task_list_content(&mut self, ui: &mut Ui) {
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

        let Some(tasks) = self.displayed_tasks.clone() else {
            return;
        };

        // Guard 2: Display informational placeholder if dataset is zero-length
        if tasks.is_empty() {
            ui.colored_label(Color32::GRAY, "No active tasks found.");
            return;
        }

        ui.separator();
        let mut task_table = TaskTable::new(&tasks);
        task_table.show(ui);
        if task_table.clicked() {
            self.temp_task = task_table.clicked_task().clone().unwrap();
            self.state = AppState::Edit;
            info!("Edit Button Pressed: {:?}", self.temp_task);
        }
    }
}
