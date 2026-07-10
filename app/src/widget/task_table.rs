use core::{Task, TaskPriority, TaskStatus};
use egui::{Color32, Label, Response, RichText, Ui};
use egui_extras::{Column, TableBuilder};

#[derive(Debug)]
pub struct TaskTable<'a> {
    tasks: &'a [Task],
    pub clicked: bool,
    pub clicked_task: Option<Task>,
}

impl<'a> TaskTable<'a> {
    pub const fn new(tasks: &'a [Task]) -> Self {
        Self {
            tasks,
            clicked: false,
            clicked_task: None,
        }
    }

    pub const fn clicked(&self) -> bool {
        self.clicked
    }

    pub fn clicked_task(&self) -> Option<Task> {
        self.clicked_task.clone()
    }

    /// メインのテーブル描画エントリーポイント
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let inner = ui.scope(|ui| {
            TableBuilder::new(ui)
                .striped(true)
                .column(Column::exact(40.0)) // Checkbox
                .column(Column::remainder()) // Title
                .column(Column::exact(80.0)) // Priority
                .column(Column::exact(90.0)) // Due Date
                .column(Column::exact(100.0)) // Progress
                .column(Column::exact(60.0)) // Edit Button
                .header(28.0, |header| self.render_header(header))
                .body(|body| {
                    body.rows(28.0, self.tasks.len(), |mut row| {
                        let task = &self.tasks[row.index()];

                        row.col(|ui| self.render_status_check(ui, task));
                        row.col(|ui| self.render_title(ui, task));
                        row.col(|ui| self.render_priority(ui, task));
                        row.col(|ui| self.render_due_date(ui, task));
                        row.col(|ui| self.render_progress(ui, task));
                        row.col(|ui| self.render_edit_button(ui, task));
                    });
                });
        });
        inner.response
    }
    /// テーブルヘッダーの描画
    fn render_header(&self, mut header: egui_extras::TableRow<'_, '_>) {
        let text_color = Color32::from_rgb(0, 240, 255);

        header.col(|ui| {
            ui.strong(RichText::new(fl!("done")).color(text_color));
        });
        header.col(|ui| {
            ui.strong(RichText::new(fl!("title")).color(text_color));
        });
        header.col(|ui| {
            ui.strong(RichText::new(fl!("priority")).color(text_color));
        });
        header.col(|ui| {
            ui.strong(RichText::new(fl!("due-date")).color(text_color));
        });
        header.col(|ui| {
            ui.strong(RichText::new(fl!("progress")).color(text_color));
        });
        header.col(|ui| {
            ui.strong(RichText::new(fl!("edit")).color(text_color));
        });
    }

    /// カラム1: 完了状態チェックボックス (読み取り専用)
    fn render_status_check(&self, ui: &mut Ui, task: &Task) {
        let mut is_done = task.status == TaskStatus::Complete;
        ui.add_enabled(false, egui::Checkbox::new(&mut is_done, ""));
    }

    /// カラム2: タイトル (溢れたら丸める)
    fn render_title(&self, ui: &mut Ui, task: &Task) {
        ui.add(Label::new(&task.title).truncate());
    }

    /// カラム3: 優先度の色分けラベル
    fn render_priority(&self, ui: &mut Ui, task: &Task) {
        let (text, color) = match task.priority {
            TaskPriority::High => (fl!("high"), Color32::from_rgb(255, 60, 60)),
            TaskPriority::Medium => (fl!("medium"), Color32::from_rgb(255, 215, 0)),
            TaskPriority::Low => (fl!("low"), Color32::from_rgb(60, 255, 60)),
        };
        ui.label(RichText::new(text).color(color));
    }

    /// カラム4: 期日表示
    fn render_due_date(&self, ui: &mut Ui, task: &Task) {
        ui.label(task.due_date.strftime("%Y/%m/%d").to_string());
    }

    /// カラム5: 進捗バーとステータスアイコン
    fn render_progress(&self, ui: &mut Ui, task: &Task) {
        let icon = match task.status {
            TaskStatus::Pending => "⏳",
            TaskStatus::WorkInProgress => "🏃",
            TaskStatus::Complete => "✅",
            TaskStatus::Canceled => "🚫",
        };
        ui.add(
            egui::ProgressBar::new(task.progress / 100.0)
                .show_percentage()
                .text(format!("{:.0}% {}", task.progress, icon)),
        );
    }

    /// カラム6: 編集ボタンとイベント発火
    fn render_edit_button(&mut self, ui: &mut Ui, task: &Task) {
        if ui.button(fl!("edit")).clicked() {
            self.clicked = true;
            self.clicked_task = Some(task.clone());
        }
    }
}
