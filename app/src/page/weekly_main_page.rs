use crate::page::{Page, Pages};
use crate::widget::MenuBar;
use crate::widget::WeeklyTaskTable; // WeeklyTaskTable をインポート
use crate::work::Work;
use core::prelude::*;
use core::{WeeklyTaskFilterFlags, WeeklyTaskOrderFlags}; // Weekly用のフラグを使用
use egui::{self, Align, Button, Color32, Layout, ScrollArea, Ui, vec2};
use tracing::{error, info};

pub struct WeeklyMainPage {
    // 構造体名を Weekly に変更
    weekly_task_table: WeeklyTaskTable, // Weekly 向けに変更
    show_only_active: bool,
    menu_bar: MenuBar,
}

impl WeeklyMainPage {
    pub fn new() -> Self {
        Self {
            weekly_task_table: WeeklyTaskTable::new(), // Weekly 向けに変更
            show_only_active: true,
            menu_bar: MenuBar::new(),
        }
    }

    /// メインコンテンツ（WeeklyTask一覧リスト / ローディング / 空表示）のレンダリング
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

        // 2. WeeklyTaskデータの存在チェック
        let Some(ref tasks) = work.weekly_tasks else {
            return;
        };

        // 3. データが空の場合のプレースホルダー表示
        if tasks.is_empty() {
            ui.colored_label(Color32::GRAY, fl!("no-active"));
            return;
        }

        // 4. テーブルの描画とクリックイベントのハンドリング
        ui.separator();
        self.weekly_task_table.show(ui, tasks);

        if self.weekly_task_table.clicked() {
            if let Some(clicked_task) = self.weekly_task_table.clicked_task() {
                // Work 内の編集ターゲットを weekly_task フィールドに同期
                work.weekly_task = clicked_task;
                work.next_page = Pages::EditWeeklyTask;
                info!("Edit Button Pressed: {:?}", work.weekly_task);
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
                    work.outputs.push(work.core.sync_all_weekly_task());
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
            work.next_page = Pages::CreateWeeklyTask;
            // WeeklyTask 用としてIDを取得
            if let Ok(id) = work.core.get_next_weekly_task_id() {
                work.weekly_task.id = id;
                info!("The next weekly task id is {}", id);
            } else {
                error!("Failed to get weekly task id");
            }
        }
    }

    /// 上部コントロールバー（タイトルとリフレッシュ）
    fn render_control_bar(&mut self, ui: &mut Ui, work: &mut Work) {
        egui::Sides::new().show(
            ui,
            |ui| {
                ui.heading(fl!("weekly-task-list"));
            },
            |ui| {
                if ui
                    .checkbox(&mut self.show_only_active, fl!("show-only-active"))
                    .changed()
                {
                    /* Request redraw of monthly task table */
                    work.weekly_tasks = None;
                }
            },
        );
    }
}

impl Page for WeeklyMainPage {
    fn on_entry(&mut self, _: &mut crate::work::Work) {
        info!("Enter to WeeklyMain Page");
    }

    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) {
        if work.outputs.is_empty() && work.weekly_tasks.is_none() {
            let filter_flag = if self.show_only_active {
                WeeklyTaskFilterFlags::All ^ WeeklyTaskFilterFlags::Inactive
            } else {
                WeeklyTaskFilterFlags::All
            };
            let order_flag = WeeklyTaskOrderFlags::OrderByPriority
                | WeeklyTaskOrderFlags::OrderByDueDay
                | WeeklyTaskOrderFlags::Reversed;
            work.outputs
                .push(work.core.fetch_all_weekly_task(filter_flag, order_flag));
        }

        self.menu_bar.show(ui, work);

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
                .id_salt("weekly-page-scroll") // IDの重複衝突を防ぐために一意な文字列へ
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                        self.render_task_list_content(work, ui);
                    });
                });
        });
    }

    fn on_exit(&mut self, _: &mut crate::work::Work) {
        info!("Exit from WeeklyMain Page");
    }
}
