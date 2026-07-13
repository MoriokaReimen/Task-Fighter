use core::DailyTask;
use core::TaskPriority;
use egui::{Color32, ComboBox, Grid, Response, RichText, ScrollArea, TextEdit, Ui};

#[derive(Debug)]
pub struct DailyTaskEdit<'a> {
    task: &'a mut DailyTask,
}

impl<'a> DailyTaskEdit<'a> {
    pub const fn new(task: &'a mut DailyTask) -> Self {
        Self { task }
    }

    pub fn show(&mut self, ui: &mut Ui) -> Response {
        ui.scope(|ui| {
            self.show_metadata_grid(ui);
            self.show_text_grid(ui);
            self.show_details(ui);
        })
        .response
    }

    /// 有効フラグ（active）と優先度（priority）のメタデータグリッドを表示
    fn show_metadata_grid(&mut self, ui: &mut Ui) {
        Grid::new("daily_task_metadata_grid")
            .striped(false)
            .num_columns(4)
            .spacing([20.0, 8.0])
            .min_col_width(0.0)
            .show(ui, |ui| {
                // 1列目 & 2列目: 有効設定
                ui.label(fl!("active"));
                ui.checkbox(&mut self.task.active, "");

                // 3列目 & 4列目: 優先度設定
                self.show_priority_combo(ui);
                ui.end_row();
            });
    }

    /// プロジェクト名、タイトルのテキスト入力グリッドを表示
    fn show_text_grid(&mut self, ui: &mut Ui) {
        Grid::new("daily_task_text_grid")
            .striped(false)
            .num_columns(2)
            .spacing([12.0, 8.0])
            .min_col_width(60.0)
            .show(ui, |ui| {
                // プロジェクト名入力
                ui.label(fl!("project"));
                ui.add(
                    TextEdit::singleline(&mut self.task.project)
                        .desired_width(ui.available_width())
                        .hint_text("Required"),
                );
                ui.end_row();

                // タイトル入力
                ui.label(fl!("title"));
                ui.add(
                    TextEdit::singleline(&mut self.task.title)
                        .desired_width(ui.available_width())
                        .hint_text("Required"),
                );
                ui.end_row();
            });
    }

    /// 詳細説明欄（スクロールエリア）を表示
    fn show_details(&mut self, ui: &mut Ui) {
        ui.label(fl!("details"));

        // 画面の最下部までの残り高さを取得してエリアを広げる
        let available_height = ui.available_height() - 8.0;
        let available_size = egui::vec2(ui.available_width(), available_height.max(85.0));

        ScrollArea::vertical()
            .max_height(available_height)
            .auto_shrink([false; 2]) // 中身が空でもエリアを縮ませない
            .show(ui, |ui| {
                ui.add_sized(
                    available_size,
                    TextEdit::multiline(&mut self.task.detail)
                        .desired_width(ui.available_width())
                        .hint_text("Optional details..."),
                );
            });
    }

    fn show_priority_combo(&mut self, ui: &mut Ui) {
        ui.label(fl!("priority"));
        let priorities = [
            (TaskPriority::Low, fl!("low"), Color32::GREEN),
            (TaskPriority::Medium, fl!("medium"), Color32::YELLOW),
            (TaskPriority::High, fl!("high"), Color32::RED),
        ];

        let idx = match self.task.priority {
            TaskPriority::Low => 0,
            TaskPriority::Medium => 1,
            TaskPriority::High => 2,
        };
        let (_, current_label, current_color) = priorities[idx].clone();

        ComboBox::from_id_salt("daily_task_priority_combo")
            .selected_text(RichText::new(current_label).color(current_color))
            .show_ui(ui, |ui| {
                for (priority, label, color) in priorities {
                    ui.selectable_value(
                        &mut self.task.priority,
                        priority,
                        RichText::new(label).color(color),
                    );
                }
            });
    }
}
