use crate::fl;
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
            ui.heading(fl!("search-condition"));
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(fl!("search-condition-prompt"));
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
                    if ui.button(fl!("search")).clicked() {
                        result = ModalResult::Search(
                            self.pattern.clone(),
                            self.filter_flags,
                            self.order_flags,
                            self.search_flags,
                        );
                        self.is_open = false;
                    }
                    ui.add_space(4.0); // ボタン間の隙間
                    if ui.button(fl!("cancel")).clicked() {
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
            ui.collapsing(fl!("filter-settings"), |ui| {
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.label(fl!("status-filter"));
                    ui.horizontal(|ui| {
                        flag_checkbox(ui, filter_flags, TaskFilterFlags::Active, &fl!("active"));
                        flag_checkbox(
                            ui,
                            filter_flags,
                            TaskFilterFlags::Inactive,
                            &fl!("inactive"),
                        );
                    });

                    ui.separator();
                    ui.label(fl!("priority"));
                    ui.horizontal(|ui| {
                        flag_checkbox(ui, filter_flags, TaskFilterFlags::PriorityLow, &fl!("low"));
                        flag_checkbox(
                            ui,
                            filter_flags,
                            TaskFilterFlags::PriorityMiddle,
                            &fl!("medium"),
                        );
                        flag_checkbox(
                            ui,
                            filter_flags,
                            TaskFilterFlags::PriorityHigh,
                            &fl!("high"),
                        );
                    });

                    ui.separator();
                    ui.label(fl!("task-status"));
                    ui.horizontal(|ui| {
                        flag_checkbox(
                            ui,
                            filter_flags,
                            TaskFilterFlags::StatusPending,
                            &fl!("pending"),
                        );
                        flag_checkbox(
                            ui,
                            filter_flags,
                            TaskFilterFlags::StatusWIP,
                            &fl!("work-in-progress"),
                        );
                        flag_checkbox(
                            ui,
                            filter_flags,
                            TaskFilterFlags::StatusComplete,
                            &fl!("complete"),
                        );
                        flag_checkbox(
                            ui,
                            filter_flags,
                            TaskFilterFlags::StatusCanceled,
                            &fl!("cancel"),
                        );
                    });
                });
            });

            ui.add_space(8.0);

            // --- 2. 検索設定 (折りたたみ) ---
            ui.collapsing(fl!("search-condition"), |ui| {
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.label(fl!("search-target"));
                    ui.horizontal(|ui| {
                        flag_checkbox(
                            ui,
                            search_flags,
                            TaskSearchFlags::SearchTitle,
                            &fl!("title"),
                        );
                        flag_checkbox(
                            ui,
                            search_flags,
                            TaskSearchFlags::SearchProject,
                            &fl!("project"),
                        );
                        flag_checkbox(
                            ui,
                            search_flags,
                            TaskSearchFlags::SearchDetail,
                            &fl!("details"),
                        );
                    });

                    ui.separator();
                    ui.label(fl!("option"));
                    flag_checkbox(
                        ui,
                        search_flags,
                        TaskSearchFlags::EnableRegex,
                        &fl!("enable-regex"),
                    );
                });
            });

            ui.add_space(8.0);

            // --- 3. 並び替え設定 (折りたたみ) ---
            ui.collapsing(fl!("order-setting"), |ui| {
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.label(fl!("sorting-criteria"));
                    ui.horizontal(|ui| {
                        flag_checkbox(
                            ui,
                            order_flags,
                            TaskOrderFlags::OrderByStatus,
                            &fl!("status-order"),
                        );
                        flag_checkbox(
                            ui,
                            order_flags,
                            TaskOrderFlags::OrderByStartDate,
                            &fl!("start-order"),
                        );
                        flag_checkbox(
                            ui,
                            order_flags,
                            TaskOrderFlags::OrderByDueDate,
                            &fl!("due-date-order"),
                        );
                        flag_checkbox(
                            ui,
                            order_flags,
                            TaskOrderFlags::OrderByEntryDate,
                            &fl!("register-order"),
                        );
                    });
                    ui.horizontal(|ui| {
                        flag_checkbox(
                            ui,
                            order_flags,
                            TaskOrderFlags::OrderByEndDate,
                            &fl!("end-date-order"),
                        );
                        flag_checkbox(
                            ui,
                            order_flags,
                            TaskOrderFlags::OrderByPriority,
                            &fl!("priority-order"),
                        );
                        flag_checkbox(
                            ui,
                            order_flags,
                            TaskOrderFlags::OrderByProgress,
                            &fl!("progress-order"),
                        );
                        flag_checkbox(
                            ui,
                            order_flags,
                            TaskOrderFlags::OrderByTimeSpent,
                            &fl!("time-spent"),
                        );
                    });

                    ui.separator();
                    ui.label(fl!("order"));
                    flag_checkbox(
                        ui,
                        order_flags,
                        TaskOrderFlags::Reversed,
                        &fl!("reverse-order"),
                    );
                });
            });
        });
}
