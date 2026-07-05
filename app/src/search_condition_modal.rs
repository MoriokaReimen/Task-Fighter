use core::{TaskFilterFlags, TaskOrderFlags, TaskSearchFlags};
use egui::Ui;

pub struct SearchConditionModal {
    is_open: bool,
    id: egui::Id,
    pattern: String,
    filter_flags: TaskFilterFlags,
    search_flags: TaskSearchFlags,
    order_flags: TaskOrderFlags,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalResult {
    Search(String, TaskFilterFlags, TaskOrderFlags, TaskSearchFlags),
    Cancel,
    None,
}

impl SearchConditionModal {
    pub fn new(id_source: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            is_open: false,
            id: egui::Id::new(id_source),
            pattern: String::new(),
            filter_flags: TaskFilterFlags::default(),
            search_flags: TaskSearchFlags::default(),
            order_flags: TaskOrderFlags::default(),
        }
    }

    pub fn open(&mut self) {
        self.pattern = String::new();
        self.is_open = true;
    }

    /// ポップアップを描画する（ウィジェット関数）
    pub fn show(&mut self, ctx: &egui::Context) -> ModalResult {
        if !self.is_open {
            return ModalResult::None;
        }

        let mut result = ModalResult::None;

        // 1. egui::Modal を一意のIDで生成
        let _ = egui::Modal::new(self.id).show(ctx, |ui| {
            // タイトルをモーダルのヘッダーとして表示
            ui.heading("Search Condition");
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Input Search Words/Pattern");
                ui.add(
                    egui::TextEdit::singleline(&mut self.pattern)
                        .desired_width(ui.available_width()),
                );
            });

            draw_task_flags_ui(
                ui,
                &mut self.filter_flags,
                &mut self.search_flags,
                &mut self.order_flags,
            );
            ui.columns(3, |columns| {
                columns[1].horizontal(|ui| {
                    if ui.button("Search").clicked() {
                        result = ModalResult::Search(
                            self.pattern.clone(),
                            self.filter_flags,
                            self.order_flags,
                            self.search_flags,
                        );
                        self.is_open = false;
                    }
                    ui.add_space(4.0); // ボタン間の隙間
                    if ui.button("Cancel").clicked() {
                        result = ModalResult::Cancel;
                        self.is_open = false;
                    }
                });
            });
        });
        result
    }
}

fn flag_checkbox<F>(ui: &mut Ui, flags: &mut F, flag: F, label: &str)
where
    F: bitflags::Flags + Copy,
{
    let mut is_checked = flags.contains(flag);

    if ui.checkbox(&mut is_checked, label).changed() {
        flags.toggle(flag);
    }
}
pub fn draw_task_flags_ui(
    ui: &mut Ui,
    filter_flags: &mut TaskFilterFlags,
    search_flags: &mut TaskSearchFlags,
    order_flags: &mut TaskOrderFlags,
) {
    let available_height = ui.available_height() * 0.9;
    egui::ScrollArea::vertical()
        .max_height(available_height)
        .auto_shrink([false; 2]) // 中身が空でもエリアを縮ませない
        .show(ui, |ui| {
            // --- 1. フィルター設定 (折りたたみ) ---
            ui.collapsing("📁 フィルター設定", |ui| {
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.label("【状態の絞り込み】");
                    ui.horizontal(|ui| {
                        flag_checkbox(ui, filter_flags, TaskFilterFlags::Active, "有効");
                        flag_checkbox(ui, filter_flags, TaskFilterFlags::Inactive, "無効");
                    });

                    ui.separator();
                    ui.label("【優先度】");
                    ui.horizontal(|ui| {
                        flag_checkbox(ui, filter_flags, TaskFilterFlags::PriorityLow, "低");
                        flag_checkbox(ui, filter_flags, TaskFilterFlags::PriorityMiddle, "中");
                        flag_checkbox(ui, filter_flags, TaskFilterFlags::PriorityHigh, "高");
                    });

                    ui.separator();
                    ui.label("【タスクステータス】");
                    ui.horizontal(|ui| {
                        flag_checkbox(ui, filter_flags, TaskFilterFlags::StatusPending, "保留");
                        flag_checkbox(ui, filter_flags, TaskFilterFlags::StatusWIP, "進行中");
                        flag_checkbox(ui, filter_flags, TaskFilterFlags::StatusComplete, "完了");
                        flag_checkbox(
                            ui,
                            filter_flags,
                            TaskFilterFlags::StatusCanceled,
                            "キャンセル",
                        );
                    });
                });
            });

            ui.add_space(8.0);

            // --- 2. 検索設定 (折りたたみ) ---
            ui.collapsing("🔍 検索設定", |ui| {
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.label("【検索対象フィールド】");
                    ui.horizontal(|ui| {
                        flag_checkbox(ui, search_flags, TaskSearchFlags::SearchTitle, "タイトル");
                        flag_checkbox(
                            ui,
                            search_flags,
                            TaskSearchFlags::SearchProject,
                            "プロジェクト",
                        );
                        flag_checkbox(ui, search_flags, TaskSearchFlags::SearchDetail, "詳細");
                    });

                    ui.separator();
                    ui.label("【オプション】");
                    flag_checkbox(
                        ui,
                        search_flags,
                        TaskSearchFlags::EnableRegex,
                        "💡 正規表現を有効にする",
                    );
                });
            });

            ui.add_space(8.0);

            // --- 3. 並び替え設定 (折りたたみ) ---
            ui.collapsing("↕ 並び替え設定", |ui| {
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.label("【ソート基準】");
                    ui.horizontal(|ui| {
                        flag_checkbox(
                            ui,
                            order_flags,
                            TaskOrderFlags::OrderByStatus,
                            "ステータス順",
                        );
                        flag_checkbox(
                            ui,
                            order_flags,
                            TaskOrderFlags::OrderByStartDate,
                            "開始日順",
                        );
                        flag_checkbox(ui, order_flags, TaskOrderFlags::OrderByDueDate, "期限日順");
                        flag_checkbox(
                            ui,
                            order_flags,
                            TaskOrderFlags::OrderByEntryDate,
                            "登録日順",
                        );
                    });
                    ui.horizontal(|ui| {
                        flag_checkbox(ui, order_flags, TaskOrderFlags::OrderByEndDate, "完了日順");
                        flag_checkbox(ui, order_flags, TaskOrderFlags::OrderByPriority, "優先度順");
                        flag_checkbox(ui, order_flags, TaskOrderFlags::OrderByProgress, "進捗率順");
                        flag_checkbox(
                            ui,
                            order_flags,
                            TaskOrderFlags::OrderByTimeSpent,
                            "消費時間順",
                        );
                    });

                    ui.separator();
                    ui.label("【順序】");
                    flag_checkbox(
                        ui,
                        order_flags,
                        TaskOrderFlags::Reversed,
                        "🔄 降順 (並び替えを反転)",
                    );
                });
            });
        });
}
