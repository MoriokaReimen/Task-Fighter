use crate::page::{Page, Pages};
use crate::widget::DailyTaskEdit; // 【変更】DailyTaskEditをインポート
use crate::widget::MenuBar;
use crate::widget::WarningModal;
use crate::widget::YesNoCancelModal;
use crate::widget::YesNoModal;
use crate::widget::yes_no_cancel_modal;
use crate::widget::yes_no_modal;
use crate::work::Work;
use core::DailyTask; // 【変更】DailyTask構造体を使用
use core::prelude::*;
use egui::{self, Align, Button, Layout, Ui, vec2};
use tracing::info;

pub struct EditDailyTaskPage {
    yes_no_cancel: YesNoCancelModal,
    yes_no: YesNoModal,
    warning: WarningModal,
    menu_bar: MenuBar,
    last_page: Pages,
}

impl EditDailyTaskPage {
    pub fn new() -> Self {
        Self {
            yes_no_cancel: YesNoCancelModal::new("edit_daily_task_yes_no_cancel"),
            yes_no: YesNoModal::new("edit_daily_task_yes_no"),
            warning: WarningModal::new("edit_daily_task_warning"),
            menu_bar: MenuBar::new(),
            last_page: Pages::DailyMain,
        }
    }
}

impl Page for EditDailyTaskPage {
    fn on_entry(&mut self, work: &mut crate::work::Work) {
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
                        if work.daily_task.is_saveable() {
                            work.outputs
                                .push(work.core.upsert_daily_task(&work.daily_task));
                            work.daily_task = DailyTask::default();
                            work.next_page = self.last_page;
                            work.daily_tasks = None; // キャッシュをクリアして再フェッチを促す
                        } else {
                            self.warning.open(fl!("save-error"), fl!("title-empty"));
                        }
                    }
                    yes_no_cancel_modal::ModalResult::No => {
                        work.daily_task = DailyTask::default();
                        work.next_page = self.last_page;
                        work.daily_tasks = None;
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
                    if work.daily_task.is_saveable() {
                        work.outputs
                            .push(work.core.upsert_daily_task(&work.daily_task));
                        work.next_page = self.last_page;
                        work.daily_tasks = None;
                    } else {
                        self.warning.open(fl!("save-error"), fl!("title-empty"));
                    }
                }
                self.warning.show(ui);
            });
        });

        // --- Main Form Panel ---
        egui::CentralPanel::default().show(ui, |ui: &mut Ui| {
            ui.heading(fl!("edit-task")); // 必要に応じてヘッダーの鍵キーを変更してください

            if !work.outputs.is_empty() {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.add(egui::Spinner::new().size(64.0));
                    },
                );
                return;
            }

            // Render input form with minimized layout nesting depth
            let mut daily_task_edit = DailyTaskEdit::new(&mut work.daily_task);
            daily_task_edit.show(ui);
        });
    }

    fn on_exit(&mut self, _: &mut crate::work::Work) {}
}
