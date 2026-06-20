use super::main_app::App;
use super::main_app::AppState;
use crate::driver::Priority;
use crate::driver::Task;
use eframe::egui::{self, Align, Layout, Ui};
use tracing::info;

impl App {
    pub fn create_view(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("bottom_panel").show_inside(ui, |ui: &mut Ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.horizontal(|ui| {
                    // --- Cancel ボタンの処理 ---
                    if ui
                        .add(egui::Button::new("❌ Cancel").min_size(egui::vec2(90.0, 28.0)))
                        .clicked()
                    {
                        info!("Cancel Button Pressed");
                        self.temp_task = Task::default();
                        self.state = AppState::Default;
                        self.displayed_tasks = None;
                    }
                    // --- Save ボタンの処理 ---
                    if ui
                        .add(egui::Button::new("💾 Save").min_size(egui::vec2(90.0, 28.0)))
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
        });

        egui::CentralPanel::default().show_inside(ui, |ui: &mut Ui| {
            ui.heading("✏ タスクを編集");
            ui.add_space(10.0);
            egui::Grid::new("create_task_date_grid")
                .num_columns(4)
                .spacing([12.0, 8.0])
                .min_col_width(80.0) // ラベル側の最小幅を固定
                .show(ui, |ui| {
                    ui.checkbox(&mut self.temp_task.done, "完了");
                    ui.checkbox(&mut self.temp_task.active, "有効");
                    // 1. 各優先度に対応する色を定義（Color32 を使用）
                    let text_color = match self.temp_task.priority {
                        Priority::Low => egui::Color32::GREEN,
                        Priority::Medium => egui::Color32::YELLOW,
                        Priority::High => egui::Color32::RED,
                    };

                    let items = ["■Low", "■Medium", "■High"];

                    // 2. selected_text に RichText を渡す
                    egui::ComboBox::from_id_salt("優先度")
                        .selected_text(
                            egui::RichText::new(items[self.temp_task.priority as usize])
                                .color(text_color),
                        )
                        .show_ui(ui, |ui| {
                            // 3. 各選択肢のラベルも RichText で色付けする
                            ui.selectable_value(
                                &mut self.temp_task.priority,
                                Priority::Low,
                                egui::RichText::new(items[0]).color(egui::Color32::GREEN),
                            );
                            ui.selectable_value(
                                &mut self.temp_task.priority,
                                Priority::Medium,
                                egui::RichText::new(items[1]).color(egui::Color32::YELLOW),
                            );
                            ui.selectable_value(
                                &mut self.temp_task.priority,
                                Priority::High,
                                egui::RichText::new(items[2]).color(egui::Color32::RED),
                            );
                        });

                    ui.end_row();
                    ui.label("開始日:");
                    ui.add(
                        egui_extras::DatePickerButton::new(&mut self.temp_task.start_date)
                            .id_salt("start_date"),
                    );
                    ui.label("締め切り:");
                    ui.add(
                        egui_extras::DatePickerButton::new(&mut self.temp_task.due_date)
                            .id_salt("due_date"),
                    );
                    ui.end_row();
                });
            // ラベルと入力欄の縦横を綺麗に揃えつつ、横幅に追従させる
            egui::Grid::new("edit_task_text_grid")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .min_col_width(80.0) // ラベル側の最小幅を固定
                .show(ui, |ui| {
                    ui.label("プロジェクト:");
                    // グリッド内で available_width を使うことで、無限ループを起こさず横幅いっぱいに追従します
                    ui.add(
                        egui::TextEdit::singleline(&mut self.temp_task.project)
                            .desired_width(ui.available_width()),
                    );
                    ui.end_row();

                    ui.label("タイトル:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.temp_task.title)
                            .desired_width(ui.available_width()),
                    );
                    ui.end_row();
                });
            ui.label("詳細:");
            egui::ScrollArea::vertical()
                .max_height(ui.available_height()) // 縦幅の最大値を固定（これを超えるとスクロール）
                .auto_shrink([false; 2]) // 中身が少なくてもエリアが縮まないようにする
                .show(ui, |ui| {
                    // 2. TextEdit をスクロールエリア内の利用可能なサイズいっぱいに広げる
                    ui.add_sized(
                        ui.available_size(), // スクロールエリア内の全幅・全高（200px）を取得
                        egui::TextEdit::multiline(&mut self.temp_task.detail),
                    );
                });
        });
    }
}
