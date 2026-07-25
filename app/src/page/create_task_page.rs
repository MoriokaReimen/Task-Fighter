use crate::page::{Page, Pages};
use crate::widget::MenuBar;
use crate::widget::TaskEdit;
use crate::widget::WarningModal;
use crate::widget::YesNoCancelModal;
use crate::widget::yes_no_cancel_modal;
use crate::work::Work;
use core::Task;
use core::prelude::*;
use egui::{self, Align, Button, Layout, Ui, vec2};
use tracing::{error, info};

pub struct CreateTaskPage {
    yes_no_cancel: YesNoCancelModal,
    warning: WarningModal,
    menu_bar: MenuBar,
    last_page: Pages,
}

impl CreateTaskPage {
    pub fn new() -> Self {
        Self {
            yes_no_cancel: YesNoCancelModal::new("create_task_yes_no_cancel"),
            warning: WarningModal::new("create_task_warning"),
            menu_bar: MenuBar::new(),
            last_page: Pages::Main,
        }
    }
}

impl Page for CreateTaskPage {
    fn on_entry(&mut self, work: &mut crate::work::Work) {
        info!("Entry to CreateTask Page");
        if !matches!(work.last_page, Pages::Config) {
            self.last_page = work.last_page;
        }

        if work.task.id == 0 {
            match work.core.get_next_task_id() {
                Ok(id) => {
                    work.task.id = id;
                    info!("The next id is {}", id);
                }
                Err(_) => error!("Failed to get id"),
            }
        }
    }

    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) {
        self.menu_bar.show(ui, work);

        // --- Bottom Action Bar ---
        egui::Panel::bottom("bottom_panel").show(ui, |ui: &mut Ui| {
            // Right-to-left layout automatically places items horizontally without nested horizontal blocks
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
                        if work.task.is_saveable() {
                            work.outputs.push(work.core.upsert_task(&work.task));
                            work.task = Task::default();
                            work.next_page = self.last_page;
                            work.tasks = None;
                        } else {
                            let message = if work.task.project.is_empty() {
                                fl!("project-empty")
                            } else {
                                fl!("title-empty")
                            };
                            self.warning.open(fl!("save-error"), message);
                        }
                    }
                    yes_no_cancel_modal::ModalResult::No => {
                        work.task = Task::default();
                        work.next_page = self.last_page;
                        work.tasks = None;
                    }
                    _ => {}
                }
                // Save Button Action
                if ui
                    .add(Button::new(fl!("save")).min_size(vec2(90.0, 28.0)))
                    .clicked()
                {
                    info!("Save Button Pressed");
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
                self.warning.show(ui);
            });
        });

        // --- Main Form Panel ---
        egui::CentralPanel::default().show(ui, |ui: &mut Ui| {
            ui.heading(fl!("create-task"));
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
            // Render input form with minimized layout nesting depth
            let mut task_edit = TaskEdit::new(&mut work.task);
            task_edit.show(ui);
        });
    }

    fn on_exit(&mut self, _: &mut crate::work::Work) {
        info!("Exit to CreateTask Page");
    }
}
