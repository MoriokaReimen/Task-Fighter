use super::main_app::{App, AppState};
use crate::core::CoreOutput;
use crate::driver::{Priority, Task, TaskStatus};
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

                ui.add_space(8.0);

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
                    self.scan_pattern.clear();
                    self.output = self.core.fetch_active_tasks();
                }

                if ui
                    .add(Button::new("🔍 Search").min_size(vec2(80.0, 28.0)))
                    .clicked()
                {
                    info!("Search Button Pressed");
                    self.output = self.core.scan_tasks_by_fts(&self.scan_pattern);
                }

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
        // FIX: Clone the data out to resolve the aliasing borrow conflict on `self`
        let Some(tasks) = self.displayed_tasks.clone() else {
            return;
        };

        // Guard 2: Display informational placeholder if dataset is zero-length
        if tasks.is_empty() {
            ui.colored_label(Color32::GRAY, "No active tasks found.");
            return;
        }

        ui.separator();

        // Iterate over the cloned vector references securely
        for task in &tasks {
            self.render_task_row(ui, task);
            ui.separator();
        }
    }

    /// Renders a single horizontal row for an individual task.
    fn render_task_row(&mut self, ui: &mut Ui, task: &Task) {
        let row_size = vec2(ui.available_width(), 28.0);
        let row_layout = Layout::left_to_right(Align::Center);

        ui.allocate_ui_with_layout(row_size, row_layout, |ui| {
            // Checkbox status setup
            let mut is_complete: bool = task.status == TaskStatus::Complete;
            ui.add_enabled(false, egui::Checkbox::new(&mut is_complete, ""));
            ui.label(format!("{}: {}", task.id, task.title));

            // Priority Indicator Match Block
            match task.priority {
                Priority::High => {
                    ui.label(egui::RichText::new("🟥").color(Color32::from_rgb(255, 60, 60)));
                }
                Priority::Medium => {
                    ui.label(egui::RichText::new("🟨").color(Color32::from_rgb(255, 215, 0)));
                }
                Priority::Low => {
                    ui.label(egui::RichText::new("🟩").color(Color32::from_rgb(60, 255, 60)));
                }
            }

            // Date & Progress Metrics UI elements
            ui.label(task.due_date.strftime("Due Date : %Y/%m/%d").to_string());

            let progress_fraction = task.progress / 100.0;
            ui.add_sized(
                [100.0, 28.0],
                egui::ProgressBar::new(progress_fraction)
                    .show_percentage()
                    .text(format!("{:.1}% Done", task.progress)),
            );

            // Right-aligned edit entry button context
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(Button::new("✏ Edit").min_size(vec2(60.0, 24.0)))
                    .clicked()
                {
                    info!("Edit Button Pressed: {:?}", task);
                    self.temp_task = task.clone();
                    self.state = AppState::Edit;
                }
            });
        });
    }
}
