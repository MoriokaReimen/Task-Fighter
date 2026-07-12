use crate::page::{Page, Pages};
use crate::widget::TaskEdit;
use crate::widget::WarningModal;
use crate::widget::YesNoCancelModal;
use crate::widget::YesNoModal;
use crate::widget::yes_no_cancel_modal;
use crate::widget::yes_no_modal;
use crate::work::Work;
use core::Task;
use core::prelude::*;
use eframe::egui::{self, Align, Button, Layout, Ui, vec2};
use tracing::info;

pub struct EditTaskPage {
    yes_no_cancel: YesNoCancelModal,
    yes_no: YesNoModal,
    warning: WarningModal,
}

impl EditTaskPage {
    pub fn new() -> Self {
        Self {
            yes_no_cancel: YesNoCancelModal::new("create_task_yes_no_cancel"),
            yes_no: YesNoModal::new("create_task_yes_no"),
            warning: WarningModal::new("create_task_warning"),
        }
    }
}

impl Page for EditTaskPage {
    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) -> Pages {
        let mut next_page = Pages::EditTask;
        // --- Bottom Action Bar ---
        egui::Panel::bottom("bottom_panel").show(ui, |ui: &mut Ui| {
            // Right-to-left layout places buttons from rightmost to leftmost
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Cancel Button Action
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
                        if work.task.is_saveable() {
                            work.outputs.push(work.core.update_task(&work.task));
                            work.task = Task::default();
                            next_page = Pages::Main;
                            work.tasks = None;
                        } else {
                            let message = if work.task.project.is_empty() {
                                fl!("project-empty")
                            } else {
                                fl!("title-empty")
                            };
                            self.warning.open(fl!("save-fail"), message);
                        }
                    }
                    yes_no_cancel_modal::ModalResult::No => {
                        work.task = Task::default();
                        work.tasks = None;
                        next_page = Pages::Main;
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
                    if work.task.is_saveable() {
                        work.outputs.push(work.core.upsert_task(&work.task));
                    } else {
                        let message = if work.task.project.is_empty() {
                            fl!("project-empty")
                        } else {
                            fl!("title-empty")
                        };
                        self.warning.open(fl!("save-error"), message);
                    }
                }
                if ui
                    .add(Button::new(fl!("start-work")).min_size(vec2(90.0, 28.0)))
                    .clicked()
                {
                    info!("Start Work Button Pressed");
                    next_page = Pages::Timer;
                    work.start_time = jiff::Zoned::now();
                }
                self.warning.show(ui);
            });
        });

        // --- Main Form Content ---
        egui::CentralPanel::default().show(ui, |ui: &mut Ui| {
            ui.heading(fl!("edit-task"));
            if !work.outputs.is_empty() {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        // サイズを大きく設定（例: 64.0 ポイント）して表示
                        ui.add(egui::Spinner::new().size(64.0));
                    },
                );
                return;
            }
            let mut task_edit = TaskEdit::new(&mut work.task);
            task_edit.show(ui);
        });
        next_page
    }
}
