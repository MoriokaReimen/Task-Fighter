use crate::page::{Page, Pages};
use crate::widget::MenuBar;
use crate::widget::WarningModal;
use crate::widget::WeeklyTaskEdit; // WeeklyTaskEdit をインポート
use crate::widget::YesNoCancelModal;
use crate::widget::yes_no_cancel_modal;
use crate::work::Work;
use core::WeeklyTask; // WeeklyTask 構造体を使用
use core::prelude::*;
use egui::{self, Align, Button, Layout, Ui, vec2};
use tracing::info;

pub struct EditWeeklyTaskPage {
    // 構造体名を Weekly に変更
    yes_no_cancel: YesNoCancelModal,
    warning: WarningModal,
    menu_bar: MenuBar,
    last_page: Pages,
}

impl EditWeeklyTaskPage {
    pub fn new() -> Self {
        Self {
            yes_no_cancel: YesNoCancelModal::new("edit_weekly_task_yes_no_cancel"),
            warning: WarningModal::new("edit_weekly_task_warning"),
            menu_bar: MenuBar::new(),
            last_page: Pages::WeeklyMain,
        }
    }
}

impl Page for EditWeeklyTaskPage {
    fn on_entry(&mut self, work: &mut crate::work::Work) {
        info!("Enter to EditWeekly Page");
        if work.last_page != Pages::Config {
            self.last_page = work.last_page;
        }
    }

    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) {
        self.menu_bar.show(ui, work);
        // --- Bottom Action Bar ---
        egui::Panel::bottom("bottom_panel").show(ui, |ui: &mut Ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Cancel / Close Action
                if ui
                    .add(Button::new(fl!("close")).min_size(vec2(90.0, 28.0)))
                    .clicked()
                {
                    info!("Close Button Pressed");
                    self.yes_no_cancel
                        .open(fl!("save-task"), fl!("save-task-message"));
                }

                match self.yes_no_cancel.show(ui) {
                    yes_no_cancel_modal::ModalResult::Yes => {
                        if work.weekly_task.is_saveable() {
                            work.outputs
                                .push(work.core.upsert_weekly_task(&work.weekly_task));
                            work.weekly_task = WeeklyTask::default();
                            work.next_page = self.last_page;
                            work.weekly_tasks = None; // キャッシュクリア
                        } else {
                            self.warning.open(fl!("save-error"), fl!("title-empty"));
                        }
                    }
                    yes_no_cancel_modal::ModalResult::No => {
                        work.weekly_task = WeeklyTask::default();
                        work.next_page = self.last_page;
                        work.weekly_tasks = None;
                    }
                    _ => {}
                }

                // Save Button Action
                if ui
                    .add(Button::new(fl!("save")).min_size(vec2(90.0, 28.0)))
                    .clicked()
                {
                    info!("Save Button Pressed");
                    if work.weekly_task.is_saveable() {
                        work.outputs
                            .push(work.core.upsert_weekly_task(&work.weekly_task));
                        work.next_page = self.last_page;
                        work.weekly_tasks = None;
                    } else {
                        self.warning.open(fl!("save-error"), fl!("title-empty"));
                    }
                }

                self.warning.show(ui);
            });
        });

        // --- Main Form Panel ---
        egui::CentralPanel::default().show(ui, |ui: &mut Ui| {
            ui.heading(fl!("edit-weekly-task"));

            if !work.outputs.is_empty() {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.add(egui::Spinner::new().size(64.0));
                    },
                );
                return;
            }

            // WeeklyTaskEdit を使用してレンダリング
            let mut weekly_task_edit = WeeklyTaskEdit::new(&mut work.weekly_task);
            weekly_task_edit.show(ui);
        });
    }

    fn on_exit(&mut self, _: &mut crate::work::Work) {
        info!("Exit from EditWeekly Page");
    }
}
