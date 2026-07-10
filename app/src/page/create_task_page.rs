use crate::main_app::{App, AppState};
use crate::page::{Page, Pages};
use crate::widget::TaskEdit;
use crate::widget::WarningModal;
use crate::widget::YesNoCancelModal;
use crate::widget::YesNoModal;
use crate::widget::yes_no_cancel_modal;
use crate::widget::yes_no_modal;
use crate::work::Work;
use core::prelude::*;
use core::{CoreOutput, Task};
use eframe::egui::{self, Align, Button, Layout, Ui, vec2};
use tracing::info;

impl App {
    /// Renders the task creation page inside separate action and workspace panels.
    pub fn create_task_page(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
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
                    self.yes_no_cancel_modal
                        .open(fl!("save-task"), fl!("save-task-message"));
                }

                match self.yes_no_cancel_modal.show(ui) {
                    yes_no_cancel_modal::ModalResult::Yes => {
                        if self.temp_task.is_saveable() {
                            self.output = self.core.upsert_task(&self.temp_task);
                            self.temp_task = Task::default();
                            self.state = AppState::Default;
                            self.displayed_tasks = None;
                        } else {
                            let message = if self.temp_task.project.is_empty() {
                                fl!("project-empty")
                            } else {
                                fl!("title-empty")
                            };
                            self.warning_modal.open(fl!("save-error"), message);
                        }
                    }
                    yes_no_cancel_modal::ModalResult::No => {
                        self.temp_task = Task::default();
                        self.state = AppState::Default;
                        self.displayed_tasks = None;
                    }
                    _ => {}
                }
                // Save Button Action
                if ui
                    .add(Button::new(fl!("save")).min_size(vec2(90.0, 28.0)))
                    .clicked()
                {
                    info!("Save Button Pressed");
                    self.yes_no_modal
                        .open(fl!("save-task"), fl!("save-task-message"));
                }
                if self.yes_no_modal.show(ui) == yes_no_modal::ModalResult::Yes {
                    if self.temp_task.is_saveable() {
                        self.output = self.core.upsert_task(&self.temp_task);
                    } else {
                        let message = if self.temp_task.project.is_empty() {
                            fl!("project-empty")
                        } else {
                            fl!("title-empty")
                        };
                        self.warning_modal.open(fl!("save-error"), message);
                    }
                }
                self.warning_modal.show(ui);
            });
        });

        // --- Main Form Panel ---
        egui::CentralPanel::default().show(ui, |ui: &mut Ui| {
            ui.heading(fl!("create-task"));
            if !matches!(self.output, CoreOutput::Idle) {
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
            let mut task_edit = TaskEdit::new(&mut self.temp_task);
            task_edit.show(ui);
        });
    }
}

struct CreateTaskPage {
    yes_no_cancel_modal: YesNoCancelModal,
    yes_no_modal: YesNoModal,
    pub warning_modal: WarningModal,
}

impl CreateTaskPage {
    pub fn new() -> Self {
        Self {
            yes_no_cancel_modal: YesNoCancelModal::new("create_task_yes_no_cancel"),
            yes_no_modal: YesNoModal::new("create_task_yes_no"),
            warning_modal: WarningModal::new("create_task_warning"),
        }
    }
}

impl Page for CreateTaskPage {
    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) -> Pages {
        let mut next_page = Pages::CreateTask;
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
                    self.yes_no_cancel_modal
                        .open(fl!("save-task"), fl!("save-task-message"));
                }

                match self.yes_no_cancel_modal.show(ui) {
                    yes_no_cancel_modal::ModalResult::Yes => {
                        if work.task.is_saveable() {
                            work.output = work.core.upsert_task(&work.task);
                            work.task = Task::default();
                            next_page = Pages::Main;
                            work.tasks = None;
                        } else {
                            let message = if work.task.project.is_empty() {
                                fl!("project-empty")
                            } else {
                                fl!("title-empty")
                            };
                            self.warning_modal.open(fl!("save-error"), message);
                        }
                    }
                    yes_no_cancel_modal::ModalResult::No => {
                        work.task = Task::default();
                        next_page = Pages::Main;
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
                    self.yes_no_modal
                        .open(fl!("save-task"), fl!("save-task-message"));
                }
                if self.yes_no_modal.show(ui) == yes_no_modal::ModalResult::Yes {
                    if work.task.is_saveable() {
                        work.output = work.core.upsert_task(&work.task);
                    } else {
                        let message = if work.task.project.is_empty() {
                            fl!("project-empty")
                        } else {
                            fl!("title-empty")
                        };
                        self.warning_modal.open(fl!("save-error"), message);
                    }
                }
                self.warning_modal.show(ui);
            });
        });

        // --- Main Form Panel ---
        egui::CentralPanel::default().show(ui, |ui: &mut Ui| {
            ui.heading(fl!("create-task"));
            if !matches!(work.output, CoreOutput::Idle) {
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

        next_page
    }
}
