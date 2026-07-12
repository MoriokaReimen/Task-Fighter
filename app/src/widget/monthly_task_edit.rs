use core::MonthlyTask; // MonthlyTask に変更
use core::TaskPriority;
use egui::{Color32, ComboBox, Grid, Response, RichText, ScrollArea, TextEdit, Ui};

#[derive(Debug)]
pub struct MonthlyTaskEdit<'a> {
    monthly_task: &'a mut MonthlyTask, // MonthlyTask へ変更
}

impl<'a> MonthlyTaskEdit<'a> {
    pub const fn new(monthly_task: &'a mut MonthlyTask) -> Self {
        Self { monthly_task }
    }

    pub fn show(&mut self, ui: &mut Ui) -> Response {
        ui.scope(|ui| {
            self.show_metadata_grid(ui);
            self.show_text_grid(ui);
            self.show_details(ui);
        })
        .response
    }

    fn show_metadata_grid(&mut self, ui: &mut Ui) {
        Grid::new("monthly_task_metadata_grid")
            .striped(false)
            .num_columns(6)
            .spacing([20.0, 8.0])
            .min_col_width(0.0)
            .show(ui, |ui| {
                ui.label(fl!("active"));
                ui.checkbox(&mut self.monthly_task.active, "");

                self.show_priority_combo(ui);

                ui.label(fl!("duration"));
                ui.horizontal(|ui| {
                    let mut start_day = self.monthly_task.start_day;
                    let mut due_day = self.monthly_task.due_day;

                    self.show_day_combo(ui, "start_day_combo", &mut start_day);
                    ui.label("~");
                    self.show_day_combo(ui, "due_day_combo", &mut due_day);

                    self.monthly_task.start_day = start_day;
                    self.monthly_task.due_day = due_day;
                });

                ui.end_row();
            });
    }

    fn show_text_grid(&mut self, ui: &mut Ui) {
        Grid::new("monthly_task_text_grid")
            .striped(false)
            .num_columns(2)
            .spacing([12.0, 8.0])
            .min_col_width(60.0)
            .show(ui, |ui| {
                ui.label(fl!("project"));
                ui.add(
                    TextEdit::singleline(&mut self.monthly_task.project)
                        .desired_width(ui.available_width())
                        .hint_text(fl!("required")),
                );
                ui.end_row();

                ui.label(fl!("title"));
                ui.add(
                    TextEdit::singleline(&mut self.monthly_task.title)
                        .desired_width(ui.available_width())
                        .hint_text(fl!("required")),
                );
                ui.end_row();
            });
    }

    fn show_details(&mut self, ui: &mut Ui) {
        ui.label(fl!("details"));

        let available_height = ui.available_height() - 8.0;
        let available_size = egui::vec2(ui.available_width(), available_height.max(85.0));

        ScrollArea::vertical()
            .max_height(available_height)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_sized(
                    available_size,
                    TextEdit::multiline(&mut self.monthly_task.detail)
                        .desired_width(ui.available_width())
                        .hint_text(fl!("optional-details")),
                );
            });
    }

    /// 優先度コンボボックス
    fn show_priority_combo(&mut self, ui: &mut Ui) {
        ui.label(fl!("priority"));
        let priorities = [
            (TaskPriority::Low, fl!("low"), Color32::GREEN),
            (TaskPriority::Medium, fl!("medium"), Color32::YELLOW),
            (TaskPriority::High, fl!("high"), Color32::RED),
        ];

        let idx = match self.monthly_task.priority {
            TaskPriority::Low => 0,
            TaskPriority::Medium => 1,
            TaskPriority::High => 2,
        };
        let (_, current_label, current_color) = priorities[idx].clone();

        ComboBox::from_id_salt("monthly_task_priority_combo")
            .selected_text(RichText::new(current_label).color(current_color))
            .show_ui(ui, |ui| {
                for (priority, label, color) in priorities {
                    ui.selectable_value(
                        &mut self.monthly_task.priority,
                        priority,
                        RichText::new(label).color(color),
                    );
                }
            });
    }

    fn show_day_combo(&self, ui: &mut Ui, id_salt: &str, current_day: &mut i16) {
        let format_label = |day: i16| fl!("day-format", day = day);

        ComboBox::from_id_salt(id_salt)
            .selected_text(format_label(*current_day))
            .show_ui(ui, |ui| {
                for day in 1..=31 {
                    ui.selectable_value(current_day, day, format_label(day));
                }
            });
    }
}
