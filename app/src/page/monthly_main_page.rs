use crate::page::{Page, Pages};
use crate::widget::AboutModal;
use crate::widget::MonthlyTaskTable; // MonthlyTaskTable に変更
use crate::work::Work;
use core::ColorScheme;
use core::prelude::*;
use core::{MonthlyTaskFilterFlags, MonthlyTaskOrderFlags}; // Monthly用のフラグを使用
use egui::{self, Align, Button, Color32, Layout, ScrollArea, Ui, vec2};
use tracing::{error, info};

pub struct MonthlyMainPage {
    // 構造体名を Monthly に変更
    about_modal: AboutModal,
    monthly_task_table: MonthlyTaskTable, // Monthly 向けに変更
    color_scheme_index: usize,
    show_only_active: bool,
}

impl MonthlyMainPage {
    pub fn new() -> Self {
        Self {
            about_modal: AboutModal::new(),
            monthly_task_table: MonthlyTaskTable::new(), // Monthly 向けに変更
            color_scheme_index: 0usize,
            show_only_active: true,
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
                ui.menu_button(fl!("menu"), |ui| {
                    if ui.button(fl!("edit-task")).clicked() {
                        info!("Switch to Main Page");
                        work.outputs.push(work.core.sync_all_monthly_task());
                        work.tasks = None;
                        work.next_page = Pages::Main;
                    }
                    if ui.button(fl!("edit-daily-task")).clicked() {
                        info!("Switch to Weekly Main Page");
                        work.daily_tasks = None;
                        work.next_page = Pages::DailyMain;
                    }
                    if ui.button(fl!("edit-weekly-task")).clicked() {
                        info!("Switch to Weekly Main Page");
                        work.weekly_tasks = None;
                        work.next_page = Pages::WeeklyMain;
                    }
                    if ui.button(fl!("change-color-scheme")).clicked() {
                        let color_scheme = COLOR_SCHEMES[self.color_scheme_index];
                        work.config.color_scheme = color_scheme;
                        work.core
                            .save_config(&work.config)
                            .expect("Failed to save config");
                        self.color_scheme_index += 1;
                        self.color_scheme_index %= 7;
                    }
                    if ui.button(fl!("about")).clicked() {
                        self.about_modal.open();
                    }
                    if ui.button(fl!("quit")).clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        let ctx = ui.ctx();
        self.about_modal.show(ctx);
    }

    /// メインコンテンツ（MonthlyTask一覧リスト / ローディング / 空表示）のレンダリング
    fn render_task_list_content(&mut self, work: &mut Work, ui: &mut Ui) {
        // 1. ローディング状態のハンドリング
        if !work.outputs.is_empty() {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.push_id("main-page-spinner", |ui| {
                        ui.add(egui::Spinner::new().size(64.0));
                    });
                },
            );
            return;
        }

        // 2. MonthlyTaskデータの存在チェック
        let Some(ref tasks) = work.monthly_tasks else {
            return;
        };

        // 3. データが空の場合のプレースホルダー表示
        if tasks.is_empty() {
            ui.colored_label(Color32::GRAY, fl!("no-active"));
            return;
        }

        // 4. テーブルの描画とクリックイベントのハンドリング
        ui.separator();
        self.monthly_task_table.show(ui, tasks);

        if self.monthly_task_table.clicked() {
            if let Some(clicked_task) = self.monthly_task_table.clicked_task() {
                // Work 内の編集ターゲットを monthly_task フィールドに同期
                work.monthly_task = clicked_task;
                work.next_page = Pages::EditMonthlyTask; // 遷移先を変更
                info!("Edit Button Pressed: {:?}", work.monthly_task);
            }
        }
    }

    /// 下部アクションパネルの描画と遷移・副作用の処理
    fn render_bottom_panel(&self, ui: &mut Ui, work: &mut Work) {
        let mut clicked_create = false;

        egui::containers::Sides::new().show(
            ui,
            |_ui| {
                // 左側要素（必要であれば戻るボタンなどを配置、現状は空）
            },
            |ui| {
                // 右側要素：同期して戻る / 新規作成
                if ui
                    .add(Button::new(fl!("back")).min_size(vec2(110.0, 28.0)))
                    .clicked()
                {
                    work.outputs.push(work.core.sync_all_monthly_task());
                    work.tasks = None;
                    work.next_page = Pages::Main;
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
            work.next_page = Pages::CreateMonthlyTask; // 遷移先を変更
            // MonthlyTask 用としてIDを取得
            if let Ok(id) = work.core.get_next_monthly_task_id() {
                // monthly に変更
                work.monthly_task.id = id;
                info!("The next monthly task id is {}", id);
            } else {
                error!("Failed to get monthly task id");
            }
        }
    }

    /// 上部コントロールバー（タイトルとリフレッシュ）
    fn render_control_bar(&mut self, ui: &mut Ui, work: &mut Work) {
        egui::Sides::new().show(
            ui,
            |ui| {
                ui.heading(fl!("monthly-task-list")); // ローカライズキーを変更（必要に応じて）
            },
            |ui| {
                if ui
                    .checkbox(&mut self.show_only_active, fl!("show-only-active"))
                    .changed()
                {
                    /* Request redraw of monthly task table */
                    work.monthly_tasks = None;
                }
            },
        );
    }
}

impl Page for MonthlyMainPage {
    fn on_entry(&mut self, work: &mut crate::work::Work) {}
    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) {
        // アプリ起動時など、アイドルかつタスク未取得なら自動フェッチを実行
        if work.outputs.is_empty() && work.monthly_tasks.is_none() {
            let filter_flag = if self.show_only_active {
                MonthlyTaskFilterFlags::All ^ MonthlyTaskFilterFlags::Inactive
            } else {
                MonthlyTaskFilterFlags::All
            };
            let order_flag = MonthlyTaskOrderFlags::OrderByPriority
                | MonthlyTaskOrderFlags::OrderByDueDate
                | MonthlyTaskOrderFlags::Reversed;
            work.outputs
                .push(work.core.fetch_all_monthly_task(filter_flag, order_flag));
        }

        self.render_top_tool_bar(work, ui);

        // --- Bottom Action Panel ---
        egui::Panel::bottom("bottom_panel").show(ui, |ui| {
            self.render_bottom_panel(ui, work);
        });

        // --- Central Dashboard Content ---
        egui::CentralPanel::default().show(ui, |ui| {
            // コントロールバーのレンダリング
            self.render_control_bar(ui, work);

            // スクロール可能なタスク一覧ワークスペース
            ScrollArea::vertical()
                .id_salt("monthly-page-scroll") // 一意のIDへ変更
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                        self.render_task_list_content(work, ui);
                    });
                });
        });
    }

    fn on_exit(&mut self, work: &mut crate::work::Work) {}
}
