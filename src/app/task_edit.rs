use crate::driver::{Priority, Task, TaskStatus};
use eframe::egui::{
    Color32, ComboBox, DragValue, Grid, Response, RichText, ScrollArea, Slider, TextEdit, Ui,
};

#[derive(Debug)]
pub struct TaskEdit<'a> {
    task: &'a mut Task,
}

impl<'a> TaskEdit<'a> {
    pub fn new(task: &'a mut Task) -> Self {
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
    /// ステータス、優先度、日付、進捗などのメタデータグリッドを表示
    fn show_metadata_grid(&mut self, ui: &mut Ui) {
        // 💡 グリッド全体の幅をWindow幅いっぱいに広げるため、最大の利用可能幅を設定
        let total_width = ui.available_width();

        Grid::new("create_task_date_grid")
            .striped(false)
            .num_columns(4)
            .spacing([20.0, 8.0])
            .min_col_width(0.0)
            .show(ui, |ui| {
                self.show_status_combo(ui);
                ui.label("Active:");
                ui.checkbox(&mut self.task.active, "");
                ui.end_row();
                self.show_priority_combo(ui);
                ui.end_row();
                ui.label("Start Date:");
                ui.add(
                    egui_extras::DatePickerButton::new(&mut self.task.start_date)
                        .id_salt("create_start_date"),
                );
                ui.label("Due Date:");
                ui.add(
                    egui_extras::DatePickerButton::new(&mut self.task.due_date)
                        .id_salt("create_due_date"),
                );
                ui.end_row();

                // 3行目: 進捗スライダー、作業時間
                ui.label("Progress:");

                // 💡 【ここがポイント】
                // グリッド全体の幅から、他の列（ラベルやDragValueなど）の概算幅を差し引いて、
                // スライダーが残りの横幅をすべて埋めるように動的計算します。
                let other_cols_width = 240.0; // "Progress:", "Time Spent:", DragValue などの合計目安幅
                let slider_width = (total_width - other_cols_width).max(100.0);

                ui.add_sized(
                    [slider_width, 28.0],
                    Slider::new(&mut self.task.progress, 0.0..=100.0)
                        .suffix("%")
                        .step_by(1.0),
                );

                ui.label("Time Spent:");
                ui.add_sized(
                    [60.0, 28.0],
                    DragValue::new(&mut self.task.time_spent)
                        .speed(0.5)
                        .range(0.0..=999.0)
                        .suffix(" hrs"),
                );
                ui.end_row();
            });
    }

    /// プロジェクト名、タイトルのテキスト入力グリッドを表示
    fn show_text_grid(&mut self, ui: &mut Ui) {
        Grid::new("create_task_text_grid")
            .striped(false)
            .num_columns(2)
            .spacing([12.0, 8.0])
            .min_col_width(60.0)
            .show(ui, |ui| {
                ui.label("Project:");
                // 💡 -30.0 などのハードコードを辞め、現在利用可能な幅いっぱいに広げる
                ui.add(
                    TextEdit::singleline(&mut self.task.project)
                        .desired_width(ui.available_width()),
                );
                ui.end_row();

                ui.label("Title:");
                ui.add(
                    TextEdit::singleline(&mut self.task.title).desired_width(ui.available_width()),
                );
                ui.end_row();
            });
    }

    /// 詳細説明欄（スクロールエリア）を表示
    fn show_details(&mut self, ui: &mut Ui) {
        ui.label("Details:");

        // 💡 1. 画面の最下部までの残り高さを取得（余白として少し引くと綺麗に収まります）
        let available_height = ui.available_height() - 8.0;
        let available_size = egui::vec2(ui.available_width(), available_height.max(85.0));

        ScrollArea::vertical()
            .max_height(available_height)
            .auto_shrink([false; 2]) // 💡 中身が空でもエリアを縮ませない
            .show(ui, |ui| {
                // 💡 2. ScrollArea の内部で、TextEdit を残りの画面サイズいっぱいに広げる
                ui.add_sized(
                    available_size,
                    TextEdit::multiline(&mut self.task.detail).desired_width(ui.available_width()),
                );
            });
    }

    /// ステータス選択ドロップダウン
    fn show_status_combo(&mut self, ui: &mut Ui) {
        ui.label("Status:");
        let statuses = [
            (TaskStatus::Pending, "⏳ Pending"),
            (TaskStatus::WorkInProgress, "🏃 In Progress"),
            (TaskStatus::Complete, "✅ Complete"),
        ];

        ComboBox::from_id_salt("create_status_combo")
            .selected_text(statuses[self.task.status as usize].1)
            .show_ui(ui, |ui| {
                for (status, label) in statuses {
                    ui.selectable_value(&mut self.task.status, status, RichText::new(label));
                }
            });
    }

    /// 優先度選択ドロップダウン
    fn show_priority_combo(&mut self, ui: &mut Ui) {
        ui.label("Priority:");
        let priorities = [
            (Priority::Low, "■Low", Color32::GREEN),
            (Priority::Medium, "■Medium", Color32::YELLOW),
            (Priority::High, "■High", Color32::RED),
        ];

        let (_, current_label, current_color) = priorities[self.task.priority as usize];

        ComboBox::from_id_salt("create_priority_combo")
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
