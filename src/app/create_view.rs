use super::main_app::{App, AppState};
use crate::driver::{Priority, Task, TaskStatus};
use eframe::egui::{
    self, Align, Button, Color32, ComboBox, DragValue, Grid, Layout, RichText, ScrollArea, Slider,
    TextEdit, Ui, vec2,
};
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
                    .add(Button::new("❌ Cancel").min_size(vec2(90.0, 28.0)))
                    .clicked()
                {
                    info!("Cancel Button Pressed");
                    self.temp_task = Task::default();
                    self.state = AppState::Default;
                    self.displayed_tasks = None;
                }

                ui.add_space(8.0);

                // Save Action
                if ui
                    .add(Button::new("💾 Save").min_size(vec2(90.0, 28.0)))
                    .clicked()
                {
                    info!("Save Button Pressed");
                    let task_to_insert = self.temp_task.clone();
                    self.output = self.core.insert_task(task_to_insert);
                    self.temp_task = Task::default();
                    self.state = AppState::Default;
                    self.displayed_tasks = None;
                }
            });
        });

        // --- Main Form Panel ---
        egui::CentralPanel::default().show_inside(ui, |ui: &mut Ui| {
            ui.heading("➕ Create Task");
            ui.add_space(10.0);

            // Render input form with minimized layout nesting depth
            self.render_create_form(ui);
        });
    }

    /// Extracted helper layout to isolate form inputs and flatten index nesting depths.
    fn render_create_form(&mut self, ui: &mut Ui) {
        // Grid 1: Status, Metadata, Dates, and Workload metrics
        Grid::new("create_task_date_grid")
            .num_columns(4)
            .spacing([12.0, 8.0])
            .min_col_width(120.0)
            .show(ui, |ui| {
                let status_items = ["⏳ Pending", "🏃 In Progress", "✅ Complete"];

                // Status Dropdown
                ComboBox::from_id_salt("create_status_combo")
                    .selected_text(status_items[self.temp_task.status as usize])
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.temp_task.status,
                            TaskStatus::Pending,
                            RichText::new(status_items[0]),
                        );
                        ui.selectable_value(
                            &mut self.temp_task.status,
                            TaskStatus::WorkInProgress,
                            RichText::new(status_items[1]),
                        );
                        ui.selectable_value(
                            &mut self.temp_task.status,
                            TaskStatus::Complete,
                            RichText::new(status_items[2]),
                        );
                    });

                ui.checkbox(&mut self.temp_task.active, "Active");

                // Priority Dropdown Context
                let priority_color = match self.temp_task.priority {
                    Priority::Low => Color32::GREEN,
                    Priority::Medium => Color32::YELLOW,
                    Priority::High => Color32::RED,
                };
                let priority_items = ["■Low", "■Medium", "■High"];
                ComboBox::from_id_salt("create_priority_combo")
                    .selected_text(
                        RichText::new(priority_items[self.temp_task.priority as usize])
                            .color(priority_color),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.temp_task.priority,
                            Priority::Low,
                            RichText::new(priority_items[0]).color(Color32::GREEN),
                        );
                        ui.selectable_value(
                            &mut self.temp_task.priority,
                            Priority::Medium,
                            RichText::new(priority_items[1]).color(Color32::YELLOW),
                        );
                        ui.selectable_value(
                            &mut self.temp_task.priority,
                            Priority::High,
                            RichText::new(priority_items[2]).color(Color32::RED),
                        );
                    });

                ui.end_row();

                // Timeline Pickers
                ui.label("Start Date:");
                ui.add(
                    egui_extras::DatePickerButton::new(&mut self.temp_task.start_date)
                        .id_salt("create_start_date"),
                );

                ui.label("Due Date:");
                ui.add(
                    egui_extras::DatePickerButton::new(&mut self.temp_task.due_date)
                        .id_salt("create_due_date"),
                );

                ui.end_row();

                // Progress Metrics
                ui.label("Progress:");
                ui.add_sized(
                    [100.0, 28.0],
                    Slider::new(&mut self.temp_task.progress, 0.0..=100.0)
                        .suffix("%")
                        .step_by(1.0),
                );

                ui.label("Time Spent:");
                ui.add_sized(
                    [80.0, 28.0],
                    DragValue::new(&mut self.temp_task.time_spent)
                        .speed(0.5)
                        .range(0.0..=999.0)
                        .suffix(" hrs"),
                );

                ui.end_row();
            });

        // Grid 2: Text inputs (Project grouping and Task Title)
        Grid::new("create_task_text_grid")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .min_col_width(80.0)
            .show(ui, |ui| {
                ui.label("Project:");
                ui.add(
                    TextEdit::singleline(&mut self.temp_task.project)
                        .desired_width(ui.available_width()),
                );
                ui.end_row();

                ui.label("Title:");
                ui.add(
                    TextEdit::singleline(&mut self.temp_task.title)
                        .desired_width(ui.available_width()),
                );
                ui.end_row();
            });

        // Large Multi-line Description Field
        ui.label("Details:");
        ScrollArea::vertical()
            .max_height(ui.available_height())
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_sized(
                    ui.available_size(),
                    TextEdit::multiline(&mut self.temp_task.detail),
                );
            });
    }
}
