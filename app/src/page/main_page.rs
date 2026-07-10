use crate::page::{Page, Pages};
use crate::widget::SearchConditionModal;
use crate::widget::TaskTable;
use crate::widget::search_condition_modal::ModalResult;
use crate::work::Work;
use core::prelude::*;
use core::{CoreOutput, TaskFilterFlags, TaskOrderFlags};
use eframe::egui::{self, Align, Button, Color32, Layout, ScrollArea, Ui, vec2};
use tracing::{error, info};

pub struct MainPage {
    search_condition_modal: SearchConditionModal,
}

impl MainPage {
    pub fn new() -> Self {
        Self {
            search_condition_modal: SearchConditionModal::new("main_page_search_condition"),
        }
    }

    /// Extracted helper to process and render list entries or empty state placeholders.
    fn render_task_list_content(work: &mut Work, ui: &mut Ui) -> Pages {
        // 【修正】初期値は「画面遷移しない（Mainのまま）」にする
        let mut next_page = Pages::Main;

        if !matches!(work.output, CoreOutput::Idle) {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.add(egui::Spinner::new().size(64.0));
                },
            );
            return next_page;
        }

        let Some(tasks) = work.tasks.clone() else {
            return next_page;
        };

        // Guard 2: Display informational placeholder if dataset is zero-length
        if tasks.is_empty() {
            ui.colored_label(Color32::GRAY, fl!("no-active"));
            return next_page;
        }

        ui.separator();
        let mut task_table = TaskTable::new(&tasks);
        task_table.show(ui);
        if task_table.clicked() {
            work.task = task_table.clicked_task().unwrap();
            // タスクがクリックされた時だけ、編集画面へ遷移させる
            next_page = Pages::EditTask;
            info!("Edit Button Pressed: {:?}", work.task);
        }

        next_page
    }
}

impl Page for MainPage {
    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) -> Pages {
        // 【修正】デフォルトは現在のページ（Main）にとどまる
        let mut next_page = Pages::Main;

        // Trigger automatic tasks fetch if system is idle and no tasks are stored yet
        if matches!(work.output, CoreOutput::Idle) && work.tasks.is_none() {
            let filter_flag = TaskFilterFlags::All & !TaskFilterFlags::Inactive;
            let order_flag = TaskOrderFlags::OrderByPriority
                | TaskOrderFlags::OrderByDueDate
                | TaskOrderFlags::Reversed;
            work.output = work.core.fetch_all_task(filter_flag, order_flag);
        }

        // --- Bottom Action Panel ---
        egui::Panel::bottom("bottom_panel").show(ui, |ui: &mut Ui| {
            let mut go_to_graph = false;
            let mut go_to_create = false;

            egui::containers::Sides::new().show(
                ui,
                |ui| {
                    if ui
                        .add(Button::new(fl!("graph")).min_size(vec2(110.0, 28.0)))
                        .clicked()
                    {
                        go_to_graph = true;
                    }
                },
                |ui| {
                    if ui
                        .add(Button::new(fl!("create-new")).min_size(vec2(110.0, 28.0)))
                        .clicked()
                    {
                        go_to_create = true;
                    }
                    if ui
                        .add(Button::new(fl!("email-report")).min_size(vec2(120.0, 28.0)))
                        .clicked()
                    {
                        info!("Email Report Button Pressed");
                        if let Some(ref tasks) = work.tasks {
                            work.output = work.core.mail_daily(tasks);
                        }
                    }
                },
            );

            if go_to_graph {
                next_page = Pages::Graph;
                work.output = work.core.get_plot_data();
            }

            if go_to_create {
                next_page = Pages::CreateTask;
                if let Ok(id) = work.core.get_next_task_id() {
                    work.task.id = id;
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
                    work.output = work.core.fetch_all_task(filter_flag, order_flag);
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
                    work.output = work.core.search_task(
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
                        // 【修正】リスト側で画面遷移（EditTaskなど）が発生した場合のみ上書きする。
                        // すでに下部パネル等で遷移先が指定されている場合は、上書きをスキップする。
                        let list_next_page = Self::render_task_list_content(work, ui);
                        if matches!(next_page, Pages::Main) {
                            next_page = list_next_page;
                        }
                    });
                });
        });

        next_page
    }
}
