use core::MonthlyTask; // MonthlyTask に変更
use core::TaskPriority;
use egui::{Color32, Label, Response, RichText, Ui};
use egui_extras::{Column, TableBuilder};

#[derive(Debug)]
pub struct MonthlyTaskTable {
    pub clicked: bool,
    pub clicked_task: Option<MonthlyTask>,
}

impl MonthlyTaskTable {
    pub const fn new() -> Self {
        Self {
            clicked: false,
            clicked_task: None,
        }
    }

    pub const fn clicked(&mut self) -> bool {
        let ret = self.clicked;
        self.clicked = false;
        ret
    }

    pub fn clicked_task(&mut self) -> Option<MonthlyTask> {
        let ret = self.clicked_task.clone();
        self.clicked_task = None;
        ret
    }

    /// メインのテーブル描画エントリーポイント
    pub fn show(&mut self, ui: &mut Ui, tasks: &[MonthlyTask]) -> Response {
        let inner = ui.scope(|ui| {
            TableBuilder::new(ui)
                .id_salt("monthly-task-table-builder") // IDを変更
                .striped(true)
                .column(Column::exact(50.0)) // Active (有効フラグ)
                .column(Column::initial(120.0).at_least(80.0)) // Project (プロジェクト名)
                .column(Column::remainder()) // Title (タイトル)
                .column(Column::exact(110.0)) // Duration (期間: ○日 ~ ○日) ※幅を微調整
                .column(Column::exact(80.0)) // Priority (優先度)
                .column(Column::exact(60.0)) // Edit Button (編集ボタン)
                .header(28.0, |header| self.render_header(header))
                .body(|body| {
                    body.rows(28.0, tasks.len(), |mut row| {
                        let task = &tasks[row.index()];

                        // 各行ごとに一意なベースIDを生成
                        let row_id = egui::Id::new(("monthly-task-row", task.uuid));

                        row.col(|ui| {
                            ui.push_id(row_id.with("active"), |ui| {
                                self.render_active_check(ui, task);
                            });
                        });
                        row.col(|ui| {
                            ui.push_id(row_id.with("project"), |ui| {
                                self.render_project(ui, task);
                            });
                        });
                        row.col(|ui| {
                            ui.push_id(row_id.with("title"), |ui| {
                                self.render_title(ui, task);
                            });
                        });
                        // カラム: 期間（日付）
                        row.col(|ui| {
                            ui.push_id(row_id.with("duration"), |ui| {
                                self.render_duration(ui, task);
                            });
                        });
                        row.col(|ui| {
                            ui.push_id(row_id.with("priority"), |ui| {
                                self.render_priority(ui, task);
                            });
                        });
                        row.col(|ui| {
                            ui.push_id(row_id.with("edit"), |ui| {
                                self.render_edit_button(ui, task);
                            });
                        });
                    });
                });
        });
        inner.response
    }

    /// テーブルヘッダーの描画
    fn render_header(&self, mut header: egui_extras::TableRow<'_, '_>) {
        let get_theme_color = |ui: &egui::Ui| {
            ui.visuals()
                .override_text_color
                .unwrap_or_else(|| ui.visuals().widgets.inactive.text_color())
        };

        header.col(|ui| {
            ui.strong(RichText::new(fl!("active")).color(get_theme_color(ui)));
        });
        header.col(|ui| {
            ui.strong(RichText::new(fl!("project")).color(get_theme_color(ui)));
        });
        header.col(|ui| {
            ui.strong(RichText::new(fl!("title")).color(get_theme_color(ui)));
        });
        header.col(|ui| {
            ui.strong(RichText::new(fl!("duration")).color(get_theme_color(ui)));
        });
        header.col(|ui| {
            ui.strong(RichText::new(fl!("priority")).color(get_theme_color(ui)));
        });
        header.col(|ui| {
            ui.strong(RichText::new(fl!("edit")).color(get_theme_color(ui)));
        });
    }

    /// カラム1: 有効状態チェックボックス (読み取り専用)
    fn render_active_check(&self, ui: &mut Ui, task: &MonthlyTask) {
        let mut is_active = task.active;
        ui.add_enabled(false, egui::Checkbox::new(&mut is_active, ""));
    }

    /// カラム2: プロジェクト名
    fn render_project(&self, ui: &mut Ui, task: &MonthlyTask) {
        ui.add(Label::new(&task.project).truncate());
    }

    /// カラム3: タイトル
    fn render_title(&self, ui: &mut Ui, task: &MonthlyTask) {
        ui.add(Label::new(&task.title).truncate());
    }

    /// カラム4: 開始日 ~ 締切日
    fn render_duration(&self, ui: &mut Ui, task: &MonthlyTask) {
        let start_str = self.day_to_string(task.start_day);
        let due_str = self.day_to_string(task.due_day);
        ui.label(format!("{start_str} ~ {due_str}"));
    }

    /// カラム5: 優先度の色分けラベル
    fn render_priority(&self, ui: &mut Ui, task: &MonthlyTask) {
        let (text, color) = match task.priority {
            TaskPriority::High => (fl!("high"), Color32::from_rgb(255, 60, 60)),
            TaskPriority::Medium => (fl!("medium"), Color32::from_rgb(255, 215, 0)),
            TaskPriority::Low => (fl!("low"), Color32::from_rgb(60, 255, 60)),
        };
        ui.label(RichText::new(text).color(color));
    }

    /// カラム6: 編集ボタンとイベント発火
    fn render_edit_button(&mut self, ui: &mut Ui, task: &MonthlyTask) {
        if ui.button(fl!("edit")).clicked() {
            self.clicked = true;
            self.clicked_task = Some(task.clone());
        }
    }

    /// i16型の日付数値を画面表示用の文字列にフォーマットするヘルパー
    fn day_to_string(&self, day: i16) -> String {
        format!("{day}日")
    }
}
