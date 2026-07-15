use crate::page::{Page, Pages};
use crate::widget::MonthlyTaskEdit; // MonthlyTaskEdit に変更
use crate::widget::WarningModal;
use crate::widget::YesNoCancelModal;
use crate::widget::YesNoModal;
use crate::widget::yes_no_cancel_modal;
use crate::widget::yes_no_modal;
use crate::work::Work;
use core::MonthlyTask; // MonthlyTask 構造体を使用
use core::prelude::*;
use egui::{self, Align, Button, Layout, Ui, vec2};
use tracing::info;

pub struct EditMonthlyTaskPage {
    // 構造体名を Monthly に変更
    yes_no_cancel: YesNoCancelModal,
    yes_no: YesNoModal,
    warning: WarningModal,
}

impl EditMonthlyTaskPage {
    pub fn new() -> Self {
        Self {
            yes_no_cancel: YesNoCancelModal::new("edit_monthly_task_yes_no_cancel"),
            yes_no: YesNoModal::new("edit_monthly_task_yes_no"),
            warning: WarningModal::new("edit_monthly_task_warning"),
        }
    }
}

impl Page for EditMonthlyTaskPage {
    fn on_entry(&mut self, work: &mut crate::work::Work) {}

    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) {
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
                        if work.monthly_task.is_saveable() {
                            work.outputs
                                .push(work.core.upsert_monthly_task(&work.monthly_task));
                            work.monthly_task = MonthlyTask::default();
                            work.next_page = Pages::MonthlyMain;
                            work.monthly_tasks = None; // キャッシュクリア
                        } else {
                            self.warning.open(fl!("save-error"), fl!("title-empty"));
                        }
                    }
                    yes_no_cancel_modal::ModalResult::No => {
                        work.monthly_task = MonthlyTask::default();
                        work.next_page = Pages::MonthlyMain;
                        work.monthly_tasks = None;
                    }
                    _ => {}
                }

                // Save Button Action
                if ui
                    .add(Button::new(fl!("save")).min_size(vec2(90.0, 28.0)))
                    .clicked()
                {
                    info!("Save Button Pressed");
                    self.yes_no.open(fl!("save-task"), fl!("save-task-message"));
                }

                if self.yes_no.show(ui) == yes_no_modal::ModalResult::Yes {
                    if work.monthly_task.is_saveable() {
                        work.outputs
                            .push(work.core.upsert_monthly_task(&work.monthly_task));
                        work.next_page = Pages::MonthlyMain;
                        work.monthly_tasks = None;
                    } else {
                        self.warning.open(fl!("save-error"), fl!("title-empty"));
                    }
                }
                self.warning.show(ui);
            });
        });

        // --- Main Form Panel ---
        egui::CentralPanel::default().show(ui, |ui: &mut Ui| {
            ui.heading(fl!("edit-monthly-task"));

            if !work.outputs.is_empty() {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.add(egui::Spinner::new().size(64.0));
                    },
                );
                return;
            }

            // MonthlyTaskEdit を使用してレンダリング
            let mut monthly_task_edit = MonthlyTaskEdit::new(&mut work.monthly_task);
            monthly_task_edit.show(ui);
        });
    }

    fn on_exit(&mut self, work: &mut crate::work::Work) {}
}
