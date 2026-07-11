use crate::page::{Page, Pages};
use crate::widget::AboutModal;
use crate::widget::DailyTaskTable; // 【変更】DailyTaskTableをインポート
use crate::work::Work;
use core::ColorScheme;
use core::prelude::*;
use core::{CoreOutput, DailyTaskFilterFlags, DailyTaskOrderFlags};
use eframe::egui::{self, Align, Button, Color32, Layout, ScrollArea, Ui, vec2};
use tracing::{error, info};

pub struct DailyMainPage {
    about_modal: AboutModal,
    daily_task_table: DailyTaskTable, // 【変更】
    color_scheme_index: usize,
}

impl DailyMainPage {
    pub fn new() -> Self {
        Self {
            about_modal: AboutModal::new(),
            daily_task_table: DailyTaskTable::new(), // 【変更】
            color_scheme_index: 0usize,
        }
    }

    fn render_top_tool_bar(&mut self, work: &mut Work, ui: &mut Ui) {
        const COLOR_SCHEMES: [ColorScheme; 7] = [
            ColorScheme::LightBlue,
            ColorScheme::DarkOrange,
            ColorScheme::WindowsLight,
            ColorScheme::WindowsDark,
            ColorScheme::Sakura,
            ColorScheme::Violet,
            ColorScheme::Chrome,
        ];
        egui::Panel::top("top_menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Menu", |ui| {
                    if ui.button("Change Color Scheme").clicked() {
                        let color_scheme = COLOR_SCHEMES[self.color_scheme_index];
                        work.config.color_scheme = color_scheme;
                        work.core
                            .save_config(&work.config)
                            .expect("Failed to save config");
                        self.color_scheme_index += 1;
                        self.color_scheme_index %= 7;
                    }
                    if ui.button("About").clicked() {
                        self.about_modal.open();
                    }
                    if ui.button("Quit").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        let ctx = ui.ctx();
        self.about_modal.show(ctx);
    }

    /// メインコンテンツ（DailyTask一覧リスト / ローディング / 空表示）のレンダリング
    fn render_task_list_content(&mut self, work: &mut Work, ui: &mut Ui) -> Pages {
        let mut next_page = Pages::DailyMain;

        // 1. ローディング状態のハンドリング
        if !matches!(work.output, CoreOutput::Idle) {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.push_id("main-page-spinner", |ui| {
                        ui.add(egui::Spinner::new().size(64.0));
                    });
                },
            );
            return next_page;
        }

        // 2. DailyTaskデータの存在チェック (Work 内のフィールド名を daily_tasks と仮定)
        let Some(ref tasks) = work.daily_tasks else {
            return next_page;
        };

        // 3. データが空の場合のプレースホルダー表示
        if tasks.is_empty() {
            ui.colored_label(Color32::GRAY, fl!("no-active"));
            return next_page;
        }

        // 4. テーブルの描画とクリックイベントのハンドリング
        ui.separator();
        self.daily_task_table.show(ui, tasks);

        if self.daily_task_table.clicked() {
            if let Some(clicked_task) = self.daily_task_table.clicked_task() {
                // Work 内の編集ターゲットを daily_task フィールドに同期
                work.daily_task = clicked_task;
                next_page = Pages::EditDailyTask;
                info!("Edit Button Pressed: {:?}", work.daily_task);
            }
        }

        next_page
    }

    /// 下部アクションパネルの描画と遷移・副作用の処理
    fn render_bottom_panel(&self, ui: &mut Ui, work: &mut Work, next_page: &mut Pages) {
        let mut clicked_create = false;

        egui::containers::Sides::new().show(
            ui,
            |_ui| {
                // 左側要素（必要であれば戻るボタンなどを配置、現状は空）
            },
            |ui| {
                // 右側要素：新規作成
                if ui
                    .add(Button::new(fl!("back")).min_size(vec2(110.0, 28.0)))
                    .clicked()
                {
                    work.output = work.core.sync_all_daily_task();
                    work.tasks = None;
                    *next_page = Pages::Main;
                }
                if ui
                    .add(Button::new(fl!("create-new")).min_size(vec2(110.0, 28.0)))
                    .clicked()
                {
                    clicked_create = true;
                }
            },
        );

        // --- 副作用の処理 ---
        if clicked_create {
            *next_page = Pages::CreateDailyTask;
            // 既存のID取得ロジック（DailyTask用としてIDを取得）
            if let Ok(id) = work.core.get_next_daily_task_id() {
                work.daily_task.id = id;
                info!("The next daily task id is {}", id);
            } else {
                error!("Failed to get daily task id");
            }
        }
    }

    /// 上部コントロールバー（タイトルとリフレッシュ）
    fn render_control_bar(&self, ui: &mut Ui, _work: &mut Work) {
        egui::Sides::new().show(
            ui,
            |ui| {
                ui.heading(fl!("daily-task-list"));
            },
            |_| { /* Empty */ },
        );
    }
}

impl Page for DailyMainPage {
    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) -> Pages {
        let mut next_page = Pages::DailyMain;

        // アプリ起動時など、アイドルかつタスク未取得なら自動フェッチを実行
        if matches!(work.output, CoreOutput::Idle) && work.daily_tasks.is_none() {
            let filter_flag = DailyTaskFilterFlags::All;
            let order_flag = DailyTaskOrderFlags::OrderByPriority
                | DailyTaskOrderFlags::OrderByDueDate
                | DailyTaskOrderFlags::Reversed;
            work.output = work.core.fetch_all_daily_task(filter_flag, order_flag);
        }

        self.render_top_tool_bar(work, ui);

        // --- Bottom Action Panel ---
        egui::Panel::bottom("bottom_panel").show(ui, |ui| {
            self.render_bottom_panel(ui, work, &mut next_page);
        });

        // --- Central Dashboard Content ---
        egui::CentralPanel::default().show(ui, |ui| {
            // コントロールバーのレンダリング
            self.render_control_bar(ui, work);

            // スクロール可能なタスク一覧ワークスペース
            ScrollArea::vertical()
                .id_salt("main-page-scroll")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                        let list_next_page = self.render_task_list_content(work, ui);

                        // 他のページへの遷移が決まっていない場合のみ、リスト内の要素クリックによる遷移を適用
                        if matches!(next_page, Pages::DailyMain) {
                            next_page = list_next_page;
                        }
                    });
                });
        });

        next_page
    }
}
