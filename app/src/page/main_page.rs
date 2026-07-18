use crate::page::{Page, Pages};
use crate::widget::MenuBar;
use crate::widget::SearchConditionModal;
use crate::widget::TaskTable;
use crate::widget::search_condition_modal::ModalResult;
use crate::work::Work;
use core::prelude::*;
use core::{TaskFilterFlags, TaskOrderFlags};
use egui::{self, Align, Button, Color32, Layout, ScrollArea, Ui, vec2};
use tracing::info;

pub struct MainPage {
    search_condition_modal: SearchConditionModal,
    task_table: TaskTable,
    menu_bar: MenuBar,
}

impl MainPage {
    pub fn new() -> Self {
        Self {
            search_condition_modal: SearchConditionModal::new("main_page_search_condition"),
            menu_bar: MenuBar::new(),
            task_table: TaskTable::new(),
        }
    }

    fn default_fetch_flags() -> (TaskFilterFlags, TaskOrderFlags) {
        let filter_flag = TaskFilterFlags::All & !TaskFilterFlags::Inactive;
        let order_flag = TaskOrderFlags::OrderByPriority
            | TaskOrderFlags::OrderByDueDate
            | TaskOrderFlags::Reversed;
        (filter_flag, order_flag)
    }

    fn render_task_list_content(&mut self, work: &mut Work, ui: &mut Ui) {
        if !work.outputs.is_empty() {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.push_id("main-page-spinner", |ui| {
                        ui.add(egui::Spinner::new().size(64.0));
                    });
                },
            );
            return;
        }

        let Some(ref tasks) = work.tasks else {
            return;
        };

        if tasks.is_empty() {
            ui.colored_label(Color32::GRAY, fl!("no-active"));
            return;
        }

        ui.separator();
        self.task_table.show(ui, tasks);

        if self.task_table.clicked() {
            if let Some(clicked_task) = self.task_table.clicked_task() {
                work.task = clicked_task;
                work.next_page = Pages::EditTask;
                info!("Edit Button Pressed: {:?}", work.task);
            }
        }
    }

    /// 下部アクションパネルの描画と遷移・副作用の処理
    fn render_bottom_panel(&self, ui: &mut Ui, work: &mut Work) {
        let mut clicked_graph = false;
        let mut clicked_create = false;
        let mut clicked_email = false;

        egui::containers::Sides::new().show(
            ui,
            |ui| {
                if ui
                    .add(Button::new(fl!("graph")).min_size(vec2(110.0, 28.0)))
                    .clicked()
                {
                    clicked_graph = true;
                }
                if ui
                    .add(Button::new(fl!("kanban")).min_size(vec2(110.0, 28.0)))
                    .clicked()
                {
                    work.next_page = Pages::Kanban;
                }
            },
            |ui| {
                if ui
                    .add(Button::new(fl!("create-new")).min_size(vec2(110.0, 28.0)))
                    .clicked()
                {
                    clicked_create = true;
                }

                if ui
                    .add(Button::new(fl!("email-report")).min_size(vec2(120.0, 28.0)))
                    .clicked()
                {
                    clicked_email = true;
                }
            },
        );

        // --- クロージャの実行がすべて終わった後（Sides::show の外）で副作用を処理する ---
        if clicked_graph {
            work.next_page = Pages::Graph;
            work.outputs.push(work.core.get_plot_data());
        }

        if clicked_create {
            work.next_page = Pages::CreateTask;
        }

        if clicked_email {
            info!("Email Report Button Pressed");
            if let Some(ref tasks) = work.tasks {
                work.outputs.push(work.core.mail_daily(tasks));
            }
        }
    }

    fn render_search_control_bar(&mut self, ui: &mut Ui, work: &mut Work) {
        egui::Sides::new().show(
            ui,
            |ui| {
                ui.heading(fl!("task-list"));
            },
            |ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(Button::new(fl!("reset")).min_size(vec2(80.0, 28.0)))
                        .clicked()
                    {
                        info!("Reset Button Pressed");
                        let (filter_flag, order_flag) = Self::default_fetch_flags();
                        work.outputs
                            .push(work.core.fetch_all_task(filter_flag, order_flag));
                    }

                    if ui
                        .add(Button::new(fl!("search")).min_size(vec2(80.0, 28.0)))
                        .clicked()
                    {
                        self.search_condition_modal.open();
                    }

                    if let ModalResult::Search(pattern, filter, order, search) =
                        self.search_condition_modal.show(ui)
                    {
                        work.outputs
                            .push(work.core.search_task(&pattern, search, filter, order));
                    }
                });
            },
        );
    }
}

impl Page for MainPage {
    fn on_entry(&mut self, work: &mut crate::work::Work) {
        work.outputs.push(work.core.sync_all_daily_task());
        work.outputs.push(work.core.sync_all_weekly_task());
        work.outputs.push(work.core.sync_all_monthly_task());
        work.tasks = None;
    }

    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) {
        if work.outputs.is_empty() && work.tasks.is_none() {
            let (filter_flag, order_flag) = Self::default_fetch_flags();
            work.outputs
                .push(work.core.fetch_all_task(filter_flag, order_flag));
        }

        self.menu_bar.show(ui, work);

        egui::Panel::bottom("bottom_panel").show(ui, |ui| {
            self.render_bottom_panel(ui, work);
        });

        egui::CentralPanel::default().show(ui, |ui| {
            self.render_search_control_bar(ui, work);
            ScrollArea::vertical()
                .id_salt("main-page-scroll")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                        self.render_task_list_content(work, ui);
                    });
                });
        });
    }

    fn on_exit(&mut self, _: &mut crate::work::Work) {}
}
