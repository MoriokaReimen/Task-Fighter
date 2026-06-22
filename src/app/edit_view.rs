use super::main_app::{App, AppState};
use crate::driver::{Priority, Task, TaskStatus};
use eframe::egui::{
    self, Align, Button, Color32, ComboBox, DragValue, Grid, Layout, RichText, ScrollArea, Slider,
    TextEdit, Ui, vec2,
};
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

                ui.add_space(8.0); // Spacing between action buttons

                // Save Button Action
                if ui
                    .add(Button::new("💾 Save").min_size(vec2(90.0, 28.0)))
                    .clicked()
                {
                    info!("Save Button Pressed");
                    if let AppState::Edit(ref mut task) = self.state {
                        self.output = self.core.update_task(task.clone());
                    }
                    self.state = AppState::Default;
                    self.displayed_tasks = None;
                }
            });
        });

        // --- Main Form Content ---
        egui::CentralPanel::default().show_inside(ui, |ui: &mut Ui| {
            ui.heading("✏ Edit Task");
            ui.add_space(10.0);

            // Guard: Flatten nesting depth by returning early if not in Edit state
            let AppState::Edit(ref mut task) = self.state else {
                return;
            };

            // Grid 1: Status, Metadata, Dates, and Workload metrics
            Grid::new("edit_task_date_grid")
                .num_columns(4)
                .spacing([12.0, 8.0])
                .min_col_width(120.0) // Lock label column width for clean alignment
                .show(ui, |ui| {
                    let status_items = ["⏳ Pending", "🏃 In Progress", "✅ Complete"];

                    // Status Selector
                    ComboBox::from_id_salt("status_combo")
                        .selected_text(status_items[task.status as usize])
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut task.status,
                                TaskStatus::Pending,
                                RichText::new(status_items[0]),
                            );
                            ui.selectable_value(
                                &mut task.status,
                                TaskStatus::WorkInProgress,
                                RichText::new(status_items[1]),
                            );
                            ui.selectable_value(
                                &mut task.status,
                                TaskStatus::Complete,
                                RichText::new(status_items[2]),
                            );
                        });

                    // Active / Enabled toggle
                    ui.checkbox(&mut task.active, "Active");

                    // Priority Selector Setup
                    let priority_color = match task.priority {
                        Priority::Low => Color32::GREEN,
                        Priority::Medium => Color32::YELLOW,
                        Priority::High => Color32::RED,
                    };
                    let priority_items = ["■Low", "■Medium", "■High"];

                    ComboBox::from_id_salt("priority_combo")
                        .selected_text(
                            RichText::new(priority_items[task.priority as usize])
                                .color(priority_color),
                        )
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut task.priority,
                                Priority::Low,
                                RichText::new(priority_items[0]).color(Color32::GREEN),
                            );
                            ui.selectable_value(
                                &mut task.priority,
                                Priority::Medium,
                                RichText::new(priority_items[1]).color(Color32::YELLOW),
                            );
                            ui.selectable_value(
                                &mut task.priority,
                                Priority::High,
                                RichText::new(priority_items[2]).color(Color32::RED),
                            );
                        });

                    ui.end_row();

                    // Date Pickers
                    ui.label("Start Date:");
                    ui.add(
                        egui_extras::DatePickerButton::new(&mut task.start_date)
                            .id_salt("edit_start_date"),
                    );

                    ui.label("Due Date:");
                    ui.add(
                        egui_extras::DatePickerButton::new(&mut task.due_date)
                            .id_salt("edit_due_date"),
                    );

                    ui.end_row();

                    // Progress Slider
                    ui.label("Progress:");
                    ui.add_sized(
                        [100.0, 28.0],
                        Slider::new(&mut task.progress, 0.0..=100.0)
                            .suffix("%")
                            .step_by(1.0), // Snap to integer percentages for better precision
                    );

                    // Time Spent Tracker
                    ui.label("Time Spent:");
                    ui.add_sized(
                        [80.0, 28.0],
                        DragValue::new(&mut task.time_spent)
                            .speed(0.5) // Adjusts stepping sensitivity
                            .range(0.0..=999.0)
                            .suffix(" hrs"),
                    );

                    ui.end_row();
                });

            // Grid 2: Text inputs (Project namespace and Task Title)
            Grid::new("edit_task_text_grid")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .min_col_width(80.0)
                .show(ui, |ui| {
                    ui.label("Project:");
                    // Use available_width to stretch inputs dynamically without sizing loops
                    ui.add(
                        TextEdit::singleline(&mut task.project).desired_width(ui.available_width()),
                    );
                    ui.end_row();

                    ui.label("Title:");
                    ui.add(
                        TextEdit::singleline(&mut task.title).desired_width(ui.available_width()),
                    );
                    ui.end_row();
                });

            // Description Field (Scrollable multi-line text editor)
            ui.label("Details:");
            ScrollArea::vertical()
                .max_height(ui.available_height()) // Constrain height to fill remaining viewport
                .auto_shrink([false; 2]) // Prevent area from resizing down on low content volume
                .show(ui, |ui| {
                    // Expand TextEdit to fully populate the container's layout size
                    ui.add_sized(ui.available_size(), TextEdit::multiline(&mut task.detail));
                });
        });
    }
}
