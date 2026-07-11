use crate::page::{Page, Pages};
use crate::widget::DailyTaskEdit; // 【変更】DailyTaskEditをインポート
use crate::widget::WarningModal;
use crate::widget::YesNoCancelModal;
use crate::widget::YesNoModal;
use crate::widget::yes_no_cancel_modal;
use crate::widget::yes_no_modal;
use crate::work::Work;
use core::prelude::*;
use core::{CoreOutput, DailyTask}; // 【変更】DailyTask構造体を使用
use eframe::egui::{self, Align, Button, Layout, Ui, vec2};
use tracing::info;

pub struct CreateDailyTaskPage {
    yes_no_cancel: YesNoCancelModal,
    yes_no: YesNoModal,
    warning: WarningModal,
}

impl CreateDailyTaskPage {
    pub fn new() -> Self {
        Self {
            yes_no_cancel: YesNoCancelModal::new("create_daily_task_yes_no_cancel"),
            yes_no: YesNoModal::new("create_daily_task_yes_no"),
            warning: WarningModal::new("create_daily_task_warning"),
        }
    }
}

impl Page for CreateDailyTaskPage {
    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) -> Pages {
        // 【変更】プロジェクトのルーティング定義に合わせて調整してください（例: Pages::CreateTask など）
        let mut next_page = Pages::CreateDailyTask;

        // --- Bottom Action Bar ---
        egui::Panel::bottom("bottom_panel").show(ui, |ui: &mut Ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Cancel Action
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
                            work.output = work.core.upsert_daily_task(&work.daily_task);
                            work.daily_task = DailyTask::default();
                            next_page = Pages::DailyMain;
                            work.daily_tasks = None;
                        } else {
                            // DailyTaskはプロジェクトを持たないため、タイトル空エラーのみに簡素化
                            self.warning.open(fl!("save-error"), fl!("title-empty"));
                        }
                    }
                    yes_no_cancel_modal::ModalResult::No => {
                        work.daily_task = DailyTask::default();
                        next_page = Pages::DailyMain;
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
                        work.output = work.core.upsert_daily_task(&work.daily_task);
                    } else {
                        self.warning.open(fl!("save-error"), fl!("title-empty"));
                    }
                }
                self.warning.show(ui);
            });
        });

        // --- Main Form Panel ---
        egui::CentralPanel::default().show(ui, |ui: &mut Ui| {
            ui.heading(fl!("create-daily-task"));

            if !matches!(work.output, CoreOutput::Idle) {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.add(egui::Spinner::new().size(64.0));
                    },
                );
                return;
            }

            // Render input form for DailyTask
            let mut daily_task_edit = DailyTaskEdit::new(&mut work.daily_task);
            daily_task_edit.show(ui);
        });

        next_page
    }
}
