use super::main_app::{App, AppState};
use crate::fl;
use crate::search_condition_modal::ModalResult;
use crate::task_table::TaskTable;
use core::prelude::*;
use core::{CoreOutput, TaskFilterFlags, TaskOrderFlags};
use eframe::egui::{self, Align, Button, Color32, Layout, ScrollArea, Ui, vec2};
use tracing::{error, info};

impl App {
    /// Renders the default task list dashboard view.
    pub fn default_view(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        // Trigger automatic tasks fetch if system is idle and no tasks are stored yet
        if matches!(self.output, CoreOutput::Idle) && self.displayed_tasks.is_none() {
            let filter_flag = TaskFilterFlags::All & !TaskFilterFlags::Inactive;
            let order_flag = TaskOrderFlags::OrderByPriority
                | TaskOrderFlags::OrderByDueDate
                | TaskOrderFlags::Reversed;
            self.output = self.core.fetch_all_task(filter_flag, order_flag);
        }

        // --- Bottom Action Panel ---
        egui::Panel::bottom("bottom_panel").show(ui, |ui: &mut Ui| {
            // クリックされたかどうかを記録するフラグを用意する
            let mut go_to_graph = false;
            let mut go_to_create = false;

            egui::containers::Sides::new().show(
                ui,
                |ui| {
                    if ui
                        .add(Button::new(fl!("graph")).min_size(vec2(110.0, 28.0)))
                        .clicked()
                    {
                        // ここではselfを書き換えず、フラグだけを立てる
                        go_to_graph = true;
                    }
                },
                |ui| {
                    if ui
                        .add(Button::new(fl!("create-new")).min_size(vec2(110.0, 28.0)))
                        .clicked()
                    {
                        // ここでもフラグだけを立てる
                        go_to_create = true;
                    }
                    if ui
                        .add(Button::new(fl!("email-report")).min_size(vec2(120.0, 28.0)))
                        .clicked()
                    {
                        info!("Email Report Button Pressed");
                        if let Some(ref tasks) = self.displayed_tasks {
                            self.output = self.core.mail_daily(tasks);
                        }
                    }
                },
            );

            // クロージャの実行が終わった（selfの借用が解除された）後で、安全に状態を更新する
            if go_to_graph {
                self.state = AppState::Graph;
                self.output = self.core.get_plot_data();
            }

            if go_to_create {
                self.state = AppState::Create;
                if let Ok(id) = self.core.get_next_task_id() {
                    self.temp_task.id = id;
                    info!("The next id is {}", id);
                } else {
                    error!("Failed to get id");
                }
            }
        });

        // --- Central Dashboard Content ---
        egui::CentralPanel::default().show(ui, |ui: &mut Ui| {
            ui.heading(fl!("task-list"));

            // Search Control Bar Layout
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                if ui
                    .add(Button::new(fl!("reset")).min_size(vec2(80.0, 28.0)))
                    .clicked()
                {
                    info!("Reset Button Pressed");
                    let filter_flag = TaskFilterFlags::All & !TaskFilterFlags::Inactive;
                    let order_flag = TaskOrderFlags::OrderByPriority
                        | TaskOrderFlags::OrderByDueDate
                        | TaskOrderFlags::Reversed;
                    self.output = self.core.fetch_all_task(filter_flag, order_flag);
                }

                if ui
                    .add(Button::new(fl!("search")).min_size(vec2(80.0, 28.0)))
                    .clicked()
                {
                    self.search_condition_modal.open();
                }

                if let ModalResult::Search(
                    pattern,
                    task_filter_flag,
                    task_order_flag,
                    task_search_flag,
                ) = self.search_condition_modal.show(ui)
                {
                    self.output = self.core.search_task(
                        &pattern,
                        task_search_flag,
                        task_filter_flag,
                        task_order_flag,
                    );
                }
            });

            // Scrollable Workspace Panels
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                        self.render_task_list_content(ui);
                    });
                });
        });
    }

    /// Extracted helper to process and render list entries or empty state placeholders.
    fn render_task_list_content(&mut self, ui: &mut Ui) {
        if !matches!(self.output, CoreOutput::Idle) {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.add(egui::Spinner::new().size(64.0));
                },
            );
            return;
        }

        let Some(tasks) = self.displayed_tasks.clone() else {
            return;
        };

        // Guard 2: Display informational placeholder if dataset is zero-length
        if tasks.is_empty() {
            ui.colored_label(Color32::GRAY, fl!("no-active"));
            return;
        }

        ui.separator();
        let mut task_table = TaskTable::new(&tasks);
        task_table.show(ui);
        if task_table.clicked() {
            self.temp_task = task_table.clicked_task().clone().unwrap();
            self.state = AppState::Edit;
            info!("Edit Button Pressed: {:?}", self.temp_task);
        }
    }
}
