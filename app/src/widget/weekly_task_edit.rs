use core::TaskPriority;
use core::WeeklyTask;
use egui::{Color32, ComboBox, Grid, Response, RichText, ScrollArea, TextEdit, Ui};
use jiff::civil::Weekday;

#[derive(Debug)]
pub struct WeeklyTaskEdit<'a> {
    weekly_task: &'a mut WeeklyTask,
}

impl<'a> WeeklyTaskEdit<'a> {
    pub const fn new(weekly_task: &'a mut WeeklyTask) -> Self {
        Self { weekly_task }
    }

    pub fn show(&mut self, ui: &mut Ui) -> Response {
        ui.scope(|ui| {
            self.show_metadata_grid(ui);
            self.show_text_grid(ui);
            self.show_details(ui);
        })
        .response
    }

    /// 有効フラグ、優先度、開始曜日、締切曜日のメタデータグリッドを表示
    fn show_metadata_grid(&mut self, ui: &mut Ui) {
        Grid::new("weekly_task_metadata_grid")
            .striped(false)
            .num_columns(6)
            .spacing([20.0, 8.0])
            .min_col_width(0.0)
            .show(ui, |ui| {
                // 1-2列目: 有効設定
                ui.label(fl!("active"));
                ui.checkbox(&mut self.weekly_task.active, "");

                // 3-4列目: 優先度設定
                self.show_priority_combo(ui);

                // 5-6列目: 開始曜日と締切曜日の設定
                ui.label(fl!("duration"));
                ui.horizontal(|ui| {
                    let mut start_day = self.weekly_task.start_day;
                    let mut due_day = self.weekly_task.due_day;
                    self.show_weekday_combo(ui, "start_day_combo", &mut start_day);
                    ui.label("~");
                    self.show_weekday_combo(ui, "due_day_combo", &mut due_day);
                    self.weekly_task.start_day = start_day;
                    self.weekly_task.due_day = due_day;
                });

                ui.end_row();
            });
    }

    /// プロジェクト名、タイトルのテキスト入力グリッドを表示
    fn show_text_grid(&mut self, ui: &mut Ui) {
        Grid::new("weekly_task_text_grid")
            .striped(false)
            .num_columns(2)
            .spacing([12.0, 8.0])
            .min_col_width(60.0)
            .show(ui, |ui| {
                ui.label(fl!("project"));
                ui.add(
                    TextEdit::singleline(&mut self.weekly_task.project)
                        .desired_width(ui.available_width())
                        .hint_text(fl!("required")),
                );
                ui.end_row();

                ui.label(fl!("title"));
                ui.add(
                    TextEdit::singleline(&mut self.weekly_task.title)
                        .desired_width(ui.available_width())
                        .hint_text(fl!("required")),
                );
                ui.end_row();
            });
    }

    /// 詳細説明欄（スクロールエリア）を表示
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
                    TextEdit::multiline(&mut self.weekly_task.detail)
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

        let idx = match self.weekly_task.priority {
            TaskPriority::Low => 0,
            TaskPriority::Medium => 1,
            TaskPriority::High => 2,
        };
        let (_, current_label, current_color) = priorities[idx].clone();

        ComboBox::from_id_salt("weekly_task_priority_combo")
            .selected_text(RichText::new(current_label).color(current_color))
            .show_ui(ui, |ui| {
                for (priority, label, color) in priorities {
                    ui.selectable_value(
                        &mut self.weekly_task.priority,
                        priority,
                        RichText::new(label).color(color),
                    );
                }
            });
    }

    /// `jiff::Weekday` 用の曜日選択コンボボックス
    fn show_weekday_combo(&self, ui: &mut Ui, id_salt: &str, current_day: &mut Weekday) {
        // jiff::Weekday の列挙子をマッピング
        let weekdays = [
            (Weekday::Monday, fl!("monday")),
            (Weekday::Tuesday, fl!("tuesday")),
            (Weekday::Wednesday, fl!("wednesday")),
            (Weekday::Thursday, fl!("thursday")),
            (Weekday::Friday, fl!("friday")),
            (Weekday::Saturday, fl!("saturday")),
            (Weekday::Sunday, fl!("sunday")),
        ];

        // 現在選択されている曜日のラベルを取得
        let current_label = weekdays
            .iter()
            .find(|(d, _)| d == current_day)
            .map_or_else(String::new, |(_, l)| l.clone());

        ComboBox::from_id_salt(id_salt)
            .selected_text(current_label)
            .show_ui(ui, |ui| {
                for (day, label) in weekdays {
                    ui.selectable_value(current_day, day, label);
                }
            });
    }
}
