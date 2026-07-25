use crate::page::{Page, Pages};
use crate::widget::MenuBar;
use crate::widget::SearchConditionModal;
use crate::widget::TaskTable;
use crate::widget::search_condition_modal::ModalResult;
use crate::work::Work;
use core::prelude::*;
use core::{TaskFilterFlags, TaskOrderFlags};
use egui::{self, Align, Button, Color32, Layout, ScrollArea, Ui, Vec2, vec2};
use tracing::info;

const ACTION_BUTTON_SIZE: Vec2 = vec2(110.0, 28.0);
const SEARCH_BUTTON_SIZE: Vec2 = vec2(80.0, 28.0);
/// Buttons whose label is long enough to need extra width (email report, markdown export).
const WIDE_BUTTON_SIZE: Vec2 = vec2(120.0, 28.0);

/// Action requested by one of the bottom panel buttons.
enum BottomAction {
    None,
    Graph,
    Kanban,
    CreateTask,
    EmailReport,
    ExportMarkdown,
}

/// Action requested from the search control bar (reset, or whatever the modal reported).
enum SearchBarAction {
    Reset,
    Modal(ModalResult),
}

/// Render a fixed-size button and report whether it was clicked.
fn button_clicked(ui: &mut Ui, label: impl Into<egui::WidgetText>, size: Vec2) -> bool {
    ui.add(Button::new(label).min_size(size)).clicked()
}

pub struct MainPage {
    search_condition_modal: SearchConditionModal,
    task_table: TaskTable,
    menu_bar: MenuBar,
}

impl Default for MainPage {
    fn default() -> Self {
        Self::new()
    }
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

    /// Kick off the standard "fetch every active task" request.
    fn fetch_all_tasks(work: &mut Work) {
        let (filter_flag, order_flag) = Self::default_fetch_flags();
        work.outputs
            .push(work.core.fetch_all_task(filter_flag, order_flag));
    }

    fn show_loading_spinner(ui: &mut Ui) {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.push_id("main-page-spinner", |ui| {
                    ui.add(egui::Spinner::new().size(64.0));
                });
            },
        );
    }

    fn render_task_list_content(&mut self, work: &mut Work, ui: &mut Ui) {
        if !work.outputs.is_empty() {
            Self::show_loading_spinner(ui);
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

        if let Some(clicked_task) = self
            .task_table
            .clicked()
            .then(|| self.task_table.clicked_task())
            .flatten()
        {
            work.task = clicked_task;
            work.next_page = Pages::EditTask;
            info!("Edit Button Pressed: {:?}", work.task);
        }
    }

    /// Render the bottom action panel and apply whichever action was clicked.
    fn render_bottom_panel(&self, ui: &mut Ui, work: &mut Work) {
        let (left_action, right_action) = egui::containers::Sides::new().show(
            ui,
            |ui| {
                if button_clicked(ui, fl!("graph"), ACTION_BUTTON_SIZE) {
                    return BottomAction::Graph;
                }
                if button_clicked(ui, fl!("kanban"), ACTION_BUTTON_SIZE) {
                    return BottomAction::Kanban;
                }
                if button_clicked(ui, fl!("email-report"), WIDE_BUTTON_SIZE) {
                    return BottomAction::EmailReport;
                }
                if button_clicked(ui, fl!("export-markdown"), WIDE_BUTTON_SIZE) {
                    return BottomAction::ExportMarkdown;
                }
                BottomAction::None
            },
            |ui| {
                if button_clicked(ui, fl!("create-new"), ACTION_BUTTON_SIZE) {
                    return BottomAction::CreateTask;
                }
                BottomAction::None
            },
        );

        // Only one side can be clicked in a given frame, so whichever side fired wins;
        // when neither did, both are `None` and nothing happens.
        let action = match left_action {
            BottomAction::None => right_action,
            clicked => clicked,
        };
        Self::apply_bottom_action(&action, work);
    }

    fn apply_bottom_action(action: &BottomAction, work: &mut Work) {
        match action {
            BottomAction::None => {}
            BottomAction::Graph => Self::open_graph(work),
            BottomAction::Kanban => work.next_page = Pages::Kanban,
            BottomAction::CreateTask => work.next_page = Pages::CreateTask,
            BottomAction::EmailReport => Self::send_email_report(work),
            BottomAction::ExportMarkdown => Self::export_markdown(work),
        }
    }

    fn open_graph(work: &mut Work) {
        work.next_page = Pages::Graph;
        work.outputs.push(work.core.get_plot_data());
    }

    fn send_email_report(work: &mut Work) {
        info!("Email Report Button Pressed");
        if let Some(tasks) = &work.tasks {
            work.outputs.push(work.core.mail_daily(tasks));
        }
    }

    fn export_markdown(work: &mut Work) {
        info!("Export Markdown Button Pressed");
        let Some(path) = rfd::FileDialog::new()
            .set_title(fl!("export-markdown"))
            .add_filter("Mark Down", &["md"])
            .set_file_name("tasks.md")
            .save_file()
        else {
            return;
        };

        if let Some(tasks) = &work.tasks {
            work.outputs.push(work.core.export_markdown(&path, tasks));
        }
    }

    /// Render the heading, reset/search buttons, and search modal; apply whichever action fired.
    fn render_search_control_bar(&mut self, ui: &mut Ui, work: &mut Work) {
        let action = egui::Sides::new()
            .show(
                ui,
                |ui| {
                    ui.heading(fl!("task-list"));
                },
                |ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if button_clicked(ui, fl!("reset"), SEARCH_BUTTON_SIZE) {
                            return SearchBarAction::Reset;
                        }

                        if button_clicked(ui, fl!("search"), SEARCH_BUTTON_SIZE) {
                            self.search_condition_modal.open();
                        }

                        SearchBarAction::Modal(self.search_condition_modal.show(ui))
                    })
                    .inner
                },
            )
            .1;

        self.apply_search_bar_action(action, work);
    }

    fn apply_search_bar_action(&self, action: SearchBarAction, work: &mut Work) {
        match action {
            SearchBarAction::Reset => {
                info!("Reset Button Pressed");
                Self::fetch_all_tasks(work);
            }
            SearchBarAction::Modal(ModalResult::Search(pattern, filter, order, search)) => {
                work.outputs
                    .push(work.core.search_task(&pattern, search, filter, order));
            }
            SearchBarAction::Modal(_) => {}
        }
    }
}

impl Page for MainPage {
    fn on_entry(&mut self, work: &mut crate::work::Work) {
        info!("Enter to Main Page");
        work.outputs.push(work.core.sync_all_daily_task());
        work.outputs.push(work.core.sync_all_weekly_task());
        work.outputs.push(work.core.sync_all_monthly_task());
        work.tasks = None;
        Self::fetch_all_tasks(work);
    }

    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) {
        if work.outputs.is_empty() && work.tasks.is_none() {
            Self::fetch_all_tasks(work);
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

    fn on_exit(&mut self, _: &mut crate::work::Work) {
        info!("Exit from Main Page");
    }
}
