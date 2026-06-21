use super::main_app::{App, AppState};
use crate::core::CoreOutput;
use crate::driver::Priority;
use eframe::egui::{self, Align, Button, Color32, Layout, ScrollArea, Ui, vec2};
use tracing::info;

impl App {
    pub fn default_view(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        // 💡 初期状態（Idle かつ データ未取得）なら自動フェッチ
        if matches!(self.output, CoreOutput::Idle) && self.displayed_tasks.is_none() {
            self.output = self.core.fetch_active_tasks();
        }
        egui::TopBottomPanel::bottom("bottom_panel").show_inside(ui, |ui: &mut Ui| {
            // 全体を縦並びのレイアウトにするため、各行を vertical で囲むか、
            // または horizontal を並べることで自動的に改行させます。

            // --- 1行目: 検索機能 (右寄せ) ---
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Create New ボタン
                let btn_create = Button::new("➕ Create New").min_size(vec2(110.0, 28.0));
                if ui.add(btn_create).clicked() {
                    self.state = AppState::Create;
                }

                ui.add_space(8.0);

                // Email Report ボタン
                let btn_email = Button::new("📧 Email Report").min_size(vec2(120.0, 28.0));
                if ui.add(btn_email).clicked() {
                    info!("Email Report Button Pressed");
                    if let Some(ref tasks) = self.displayed_tasks {
                        self.output = self.core.mail_daily(tasks.clone());
                    }
                }
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui: &mut Ui| {
            // --- 1. ヘッダーエリア ---
            ui.heading("📋 タスク一覧");
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                let btn_reset = Button::new("↩ Reset").min_size(vec2(80.0, 28.0));
                if ui.add(btn_reset).clicked() {
                    info!("Reset Button Pressed");
                    self.scan_pattern.clear();
                    self.output = self.core.fetch_active_tasks();
                }
                // 先に右端に Search ボタンを配置
                let btn_search = Button::new("🔍Search").min_size(vec2(80.0, 28.0));
                if ui.add(btn_search).clicked() {
                    info!("Search Button Pressed");
                    self.output = self.core.scan_tasks_by_fts(&self.scan_pattern);
                }

                // 残りの幅すべてを TextEdit に割り当てる
                ui.add(
                    egui::TextEdit::singleline(&mut self.scan_pattern)
                        .desired_width(ui.available_width()),
                );
            });

            // --- 2b. スクロール可能なタスク表示エリア ---
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                        let Some(ref tasks) = self.displayed_tasks else {
                            return;
                        };

                        if tasks.is_empty() {
                            ui.colored_label(Color32::GRAY, "有効なタスクはありません。");
                            return;
                        }

                        ui.separator();
                        for task in tasks {
                            // 横幅いっぱいの行を作成
                            let row_size = vec2(ui.available_width(), 28.0);
                            let row_layout = Layout::left_to_right(Align::Center);

                            ui.allocate_ui_with_layout(row_size, row_layout, |ui| {
                                // 左側：タイトル
                                ui.add_enabled(
                                    false,
                                    egui::Checkbox::new(&mut task.done.clone(), ""),
                                );
                                ui.label(format!("{}: {}", task.id, task.title));
                                match task.priority {
                                    Priority::High => {
                                        ui.label(
                                            egui::RichText::new("🟥")
                                                .color(egui::Color32::from_rgb(255, 60, 60)),
                                        );
                                    }
                                    Priority::Medium => {
                                        ui.label(
                                            egui::RichText::new("🟨")
                                                .color(egui::Color32::from_rgb(255, 215, 0)),
                                        );
                                    }
                                    Priority::Low => {
                                        ui.label(
                                            egui::RichText::new("🟩")
                                                .color(egui::Color32::from_rgb(60, 255, 60)),
                                        );
                                    }
                                }
                                ui.label(task.due_date.strftime("Due Date : %Y/%m/%d").to_string());
                                let progress_fraction = task.progress / 100.0;
                                ui.add_sized(
                                    [100.0, 28.0],
                                    egui::ProgressBar::new(progress_fraction)
                                        .show_percentage() // バーの中に「50%」のようにテキストを表示
                                        .text(format!("{:.1}% Done", task.progress)), // カスタムテキストを表示したい場合
                                );

                                // 右側：編集ボタン
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let btn_edit = Button::new("✏ Edit").min_size(vec2(60.0, 24.0));
                                    if ui.add(btn_edit).clicked() {
                                        info!("Edit Button Pressed: {:?}", task);
                                        self.state = AppState::Edit(task.clone());
                                    }
                                });
                            });
                            ui.separator();
                        }
                    });
                });
        });
    }
}
