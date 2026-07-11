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

#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// モーダルを表示・描画する
    pub fn show(&mut self, ctx: &egui::Context) -> ModalResult {
        if !self.is_open {
            return ModalResult::None;
        }

        let mut result = ModalResult::None;

        let _ = egui::Modal::new(self.id).show(ctx, |ui| {
            // ヘッダーと入力エリア
            ui.heading(fl!("search-condition"));
            ui.add_space(4.0);
            self.render_input_bar(ui);

            // スクロール可能なフラグ選択エリア
            let max_h = ui.available_height() * 0.9;
            egui::ScrollArea::vertical()
                .max_height(max_h)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    filter_section(ui, &mut self.filter_flags);
                    ui.add_space(8.0);
                    search_section(ui, &mut self.search_flags);
                    ui.add_space(8.0);
                    order_section(ui, &mut self.order_flags);
                });

            // 下部アクションボタン
            self.render_buttons(ui, &mut result);
        });

        result
    }

    /// 検索キーワードの入力バーを描画
    fn render_input_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(fl!("search-condition-prompt"));
            ui.add(
                egui::TextEdit::singleline(&mut self.pattern).desired_width(ui.available_width()),
            );
        });
    }

    /// 下部の「検索」「キャンセル」ボタンを描画
    fn render_buttons(&mut self, ui: &mut Ui, result: &mut ModalResult) {
        ui.columns(3, |cols| {
            cols[1].horizontal(|ui| {
                if ui.button(fl!("search")).clicked() {
                    *result = ModalResult::Search(
                        self.pattern.clone(),
                        self.filter_flags,
                        self.order_flags,
                        self.search_flags,
                    );
                    self.is_open = false;
                }
                ui.add_space(4.0);
                if ui.button(fl!("cancel")).clicked() {
                    *result = ModalResult::Cancel;
                    self.is_open = false;
                }
            });
        });
    }
}

/// ビットフラグ専用の簡潔なチェックボックスヘルパー
fn checkbox<F>(ui: &mut Ui, flags: &mut F, flag: F, label: &str)
where
    F: bitflags::Flags + Copy,
{
    let mut checked = flags.contains(flag);
    if ui.checkbox(&mut checked, label).changed() {
        flags.toggle(flag);
    }
}

/// 1. フィルター設定セクション
fn filter_section(ui: &mut Ui, flags: &mut TaskFilterFlags) {
    ui.collapsing(fl!("filter-settings"), |ui| {
        ui.vertical(|ui| {
            ui.add_space(4.0);
            ui.label(fl!("status-filter"));
            ui.horizontal(|ui| {
                checkbox(ui, flags, TaskFilterFlags::Active, &fl!("active"));
                checkbox(ui, flags, TaskFilterFlags::Inactive, &fl!("inactive"));
            });

            ui.separator();
            ui.label(fl!("priority"));
            ui.horizontal(|ui| {
                checkbox(ui, flags, TaskFilterFlags::PriorityLow, &fl!("low"));
                checkbox(ui, flags, TaskFilterFlags::PriorityMiddle, &fl!("medium"));
                checkbox(ui, flags, TaskFilterFlags::PriorityHigh, &fl!("high"));
            });

            ui.separator();
            ui.label(fl!("task-status"));
            ui.horizontal(|ui| {
                checkbox(ui, flags, TaskFilterFlags::StatusPending, &fl!("pending"));
                checkbox(
                    ui,
                    flags,
                    TaskFilterFlags::StatusWIP,
                    &fl!("work-in-progress"),
                );
                checkbox(ui, flags, TaskFilterFlags::StatusComplete, &fl!("complete"));
                checkbox(ui, flags, TaskFilterFlags::StatusCanceled, &fl!("cancel"));
            });
        });
    });
}

/// 2. 検索設定セクション
fn search_section(ui: &mut Ui, flags: &mut TaskSearchFlags) {
    ui.collapsing(fl!("search-condition"), |ui| {
        ui.vertical(|ui| {
            ui.add_space(4.0);
            ui.label(fl!("search-target"));
            ui.horizontal(|ui| {
                checkbox(ui, flags, TaskSearchFlags::SearchTitle, &fl!("title"));
                checkbox(ui, flags, TaskSearchFlags::SearchProject, &fl!("project"));
                checkbox(ui, flags, TaskSearchFlags::SearchDetail, &fl!("details"));
            });

            ui.separator();
            ui.label(fl!("option"));
            checkbox(
                ui,
                flags,
                TaskSearchFlags::EnableRegex,
                &fl!("enable-regex"),
            );
        });
    });
}

/// 3. 並び替え設定セクション
fn order_section(ui: &mut Ui, flags: &mut TaskOrderFlags) {
    ui.collapsing(fl!("order-setting"), |ui| {
        ui.vertical(|ui| {
            ui.add_space(4.0);
            ui.label(fl!("sorting-criteria"));
            ui.horizontal(|ui| {
                checkbox(
                    ui,
                    flags,
                    TaskOrderFlags::OrderByStatus,
                    &fl!("status-order"),
                );
                checkbox(
                    ui,
                    flags,
                    TaskOrderFlags::OrderByStartDate,
                    &fl!("start-order"),
                );
                checkbox(
                    ui,
                    flags,
                    TaskOrderFlags::OrderByDueDate,
                    &fl!("due-date-order"),
                );
                checkbox(
                    ui,
                    flags,
                    TaskOrderFlags::OrderByEntryDate,
                    &fl!("register-order"),
                );
            });
            ui.horizontal(|ui| {
                checkbox(
                    ui,
                    flags,
                    TaskOrderFlags::OrderByEndDate,
                    &fl!("end-date-order"),
                );
                checkbox(
                    ui,
                    flags,
                    TaskOrderFlags::OrderByPriority,
                    &fl!("priority-order"),
                );
                checkbox(
                    ui,
                    flags,
                    TaskOrderFlags::OrderByProgress,
                    &fl!("progress-order"),
                );
                checkbox(
                    ui,
                    flags,
                    TaskOrderFlags::OrderByTimeSpent,
                    &fl!("time-spent"),
                );
            });

            ui.separator();
            ui.label(fl!("order"));
            checkbox(ui, flags, TaskOrderFlags::Reversed, &fl!("reverse-order"));
        });
    });
}
