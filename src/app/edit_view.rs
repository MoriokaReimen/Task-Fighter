use super::main_app::App;
use super::main_app::AppState;
use crate::driver::{Priority, Task, TaskStatus};
use eframe::egui::{self, Ui};
use tracing::info;

impl App {
    pub fn edit_view(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("bottom_panel").show_inside(ui, |ui: &mut Ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 右寄せ（right_to_left）のため、右端に置きたいボタンから順に配置します

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

                ui.add_space(8.0); // ボタン間の隙間

                // --- Save ボタンの処理 ---
                if ui
                    .add(egui::Button::new("💾 Save").min_size(egui::vec2(90.0, 28.0)))
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

        egui::CentralPanel::default().show_inside(ui, |ui: &mut Ui| {
            ui.heading("✏ タスクを編集");
            ui.add_space(10.0);
            // 1. AppState::Edit から編集対象のタスクを参照
            if let AppState::Edit(ref mut task) = self.state {
                egui::Grid::new("edit_task_date_grid")
                    .num_columns(4)
                    .spacing([12.0, 8.0])
                    .min_col_width(80.0) // ラベル側の最小幅を固定
                    .show(ui, |ui| {
                        let items = ["■Pending", "■Work In Progress", "■Complete"];

                        // 2. selected_text に RichText を渡す
                        egui::ComboBox::from_id_salt("状態").show_ui(ui, |ui| {
                            // 3. 各選択肢のラベルも RichText で色付けする
                            ui.selectable_value(
                                &mut task.status,
                                TaskStatus::Pending,
                                egui::RichText::new(items[0]),
                            );
                            ui.selectable_value(
                                &mut task.status,
                                TaskStatus::WorkInProgress,
                                egui::RichText::new(items[1]),
                            );
                            ui.selectable_value(
                                &mut task.status,
                                TaskStatus::Complete,
                                egui::RichText::new(items[2]),
                            );
                        });
                        ui.checkbox(&mut task.active, "有効");
                        // 1. 各優先度に対応する色を定義（Color32 を使用）
                        let text_color = match task.priority {
                            Priority::Low => egui::Color32::GREEN,
                            Priority::Medium => egui::Color32::YELLOW,
                            Priority::High => egui::Color32::RED,
                        };

                        let items = ["■Low", "■Medium", "■High"];

                        // 2. selected_text に RichText を渡す
                        egui::ComboBox::from_id_salt("優先度")
                            .selected_text(
                                egui::RichText::new(items[task.priority as usize])
                                    .color(text_color),
                            )
                            .show_ui(ui, |ui| {
                                // 3. 各選択肢のラベルも RichText で色付けする
                                ui.selectable_value(
                                    &mut task.priority,
                                    Priority::Low,
                                    egui::RichText::new(items[0]).color(egui::Color32::GREEN),
                                );
                                ui.selectable_value(
                                    &mut task.priority,
                                    Priority::Medium,
                                    egui::RichText::new(items[1]).color(egui::Color32::YELLOW),
                                );
                                ui.selectable_value(
                                    &mut task.priority,
                                    Priority::High,
                                    egui::RichText::new(items[2]).color(egui::Color32::RED),
                                );
                            });

                        ui.end_row();
                        ui.label("開始日:");
                        ui.add(
                            egui_extras::DatePickerButton::new(&mut task.start_date)
                                .id_salt("start_date2"),
                        );
                        ui.label("締め切り:");
                        ui.add(
                            egui_extras::DatePickerButton::new(&mut task.due_date)
                                .id_salt("due_date2"),
                        );
                        ui.end_row();
                        ui.label("進捗:");
                        let _res = ui.add_sized(
                            [100.0, 28.0],
                            egui::Slider::new(&mut task.progress, 0.0..=100.0)
                                .suffix("%")
                                .step_by(1.0), // 1% 刻みに固定して直感的に操作しやすくする
                        );
                        ui.label("工数:");
                        let _res = ui.add_sized(
                            [80.0, 28.0],
                            egui::DragValue::new(&mut task.time_spent)
                                .speed(0.5) // ドラッグしたときの増減のスピード（0.5時間ずつなど）
                                .range(0.0..=999.0) // 入力可能な最小値と最大値
                                .suffix(" hrs"), // 数字の後ろに単位を表示（例: 2.5 hrs）
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
                            egui::TextEdit::singleline(&mut task.project)
                                .desired_width(ui.available_width()),
                        );
                        ui.end_row();

                        ui.label("タイトル:");
                        ui.add(
                            egui::TextEdit::singleline(&mut task.title)
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
                            egui::TextEdit::multiline(&mut task.detail),
                        );
                    });
            }
        });
    }
}
