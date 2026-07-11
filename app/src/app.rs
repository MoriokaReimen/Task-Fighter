use super::style;
use crate::page::{self, Page, Pages};
use crate::work::Work;
use core::prelude::*;
use core::{CoreOutput, TryRecvError};
use eframe::egui::Ui;
use std::collections::HashMap;
use tracing::{error, warn};

/// Main application state holder.
pub struct App {
    next_page: Pages,
    work: Work,
    pages: HashMap<Pages, Box<dyn Page>>,
}

impl App {
    /// Initializes application state and applies global UI styling.
    pub fn new(ctx: &egui::Context) -> Self {
        style::setup_style(ctx);
        let mut pages: HashMap<Pages, Box<dyn Page>> = HashMap::new();
        pages.insert(Pages::Main, Box::new(page::MainPage::new()));
        pages.insert(Pages::EditTask, Box::new(page::EditTaskPage::new()));
        pages.insert(Pages::CreateTask, Box::new(page::CreateTaskPage::new()));
        pages.insert(Pages::Timer, Box::new(page::TimerPage::new()));
        pages.insert(Pages::Graph, Box::new(page::GraphPage::new()));
        pages.insert(Pages::DailyMain, Box::new(page::DailyMainPage::new()));
        pages.insert(
            Pages::EditDailyTask,
            Box::new(page::EditDailyTaskPage::new()),
        );
        pages.insert(
            Pages::CreateDailyTask,
            Box::new(page::CreateDailyTaskPage::new()),
        );
        pages.insert(Pages::WeeklyMain, Box::new(page::WeeklyMainPage::new()));
        pages.insert(
            Pages::EditWeeklyTask,
            Box::new(page::EditWeeklyTaskPage::new()),
        );
        pages.insert(
            Pages::CreateWeeklyTask,
            Box::new(page::CreateWeeklyTaskPage::new()),
        );

        let mut work = Work::new();
        work.config = work.core.load_config().expect("Failed to load config");
        work.core.sync_all_daily_task();
        work.core.sync_all_weekly_task();
        work.core.sync_all_monthly_task();

        Self {
            next_page: Pages::Main,
            work,
            pages,
        }
    }
}

impl eframe::App for App {
    /// Main UI update loop called on every frame render.
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        style::set_theme(ui.ctx(), &self.work);
        self.poll_background_tasks();
        if let Some(page) = self.pages.get_mut(&self.next_page) {
            self.next_page = page.show(ui, &mut self.work);
        } else {
            warn!("Page not found: {:?}", self.next_page);
        }
    }
}

impl App {
    fn poll_background_tasks(&mut self) {
        let next_output = match &mut self.work.output {
            CoreOutput::Idle => None,

            CoreOutput::FetchAllTask(rx) => match rx.try_recv() {
                Ok(Ok(tasks)) => {
                    self.work.tasks = Some(tasks);
                    Some(CoreOutput::Idle)
                }
                Ok(Err(e)) => {
                    error!("Failed to fetch active tasks: {:?}", e);
                    Some(CoreOutput::Idle)
                }
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Closed) => {
                    error!("Channel disconnected unexpectedly (FetchActiveTasks)");
                    Some(CoreOutput::Idle)
                }
            },
            CoreOutput::SearchTask(rx) => match rx.try_recv() {
                Ok(Ok(tasks)) => {
                    self.work.tasks = Some(tasks);
                    Some(CoreOutput::Idle)
                }
                Ok(Err(e)) => {
                    warn!("Search query failed: {:?}", e);
                    Some(CoreOutput::Idle)
                }
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Closed) => {
                    error!("Channel disconnected unexpectedly (ScanTasks)");
                    Some(CoreOutput::Idle)
                }
            },
            CoreOutput::FetchAllDailyTask(rx) => match rx.try_recv() {
                Ok(Ok(daily_tasks)) => {
                    self.work.daily_tasks = Some(daily_tasks);
                    Some(CoreOutput::Idle)
                }
                Ok(Err(e)) => {
                    error!("Failed to fetch daily tasks: {:?}", e);
                    Some(CoreOutput::Idle)
                }
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Closed) => {
                    error!("Channel disconnected unexpectedly (FetchActiveTasks)");
                    Some(CoreOutput::Idle)
                }
            },
            CoreOutput::FetchAllWeeklyTask(rx) => match rx.try_recv() {
                Ok(Ok(weekly_tasks)) => {
                    self.work.weekly_tasks = Some(weekly_tasks);
                    Some(CoreOutput::Idle)
                }
                Ok(Err(e)) => {
                    error!("Failed to fetch weekly tasks: {:?}", e);
                    Some(CoreOutput::Idle)
                }
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Closed) => {
                    error!("Channel disconnected unexpectedly (FetchActiveTasks)");
                    Some(CoreOutput::Idle)
                }
            },
            CoreOutput::PlotData(rx) => match rx.try_recv() {
                Ok(Ok(data)) => {
                    self.work.plot_data = Some(data);
                    Some(CoreOutput::Idle)
                }
                Ok(Err(e)) => {
                    warn!("Plot Data failed: {:?}", e);
                    Some(CoreOutput::Idle)
                }
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Closed) => {
                    error!("Channel disconnected unexpectedly (plot_data)");
                    Some(CoreOutput::Idle)
                }
            },

            other_output => {
                macro_rules! handle_rx {
                    ($rx:expr, $err_msg:expr) => {
                        match $rx.try_recv() {
                            Ok(Ok(_)) => Some(CoreOutput::Idle),
                            Ok(Err(e)) => {
                                error!("{}: {:?}", $err_msg, e);
                                Some(CoreOutput::Idle)
                            }
                            Err(TryRecvError::Empty) => None,
                            Err(TryRecvError::Closed) => {
                                error!("Channel disconnected unexpectedly ({})", $err_msg);
                                Some(CoreOutput::Idle)
                            }
                        }
                    };
                }

                match other_output {
                    CoreOutput::InsertTask(rx) => handle_rx!(rx, "Failed to insert task"),
                    CoreOutput::UpsertTask(rx) => handle_rx!(rx, "Failed to insert task"),
                    CoreOutput::FetchAllTask(rx) => handle_rx!(rx, "Failed to fetch all tasks"),
                    CoreOutput::FetchOneTask(rx) => handle_rx!(rx, "Failed to fetch task by ID"),
                    CoreOutput::UpdateTask(rx) => handle_rx!(rx, "Failed to update task"),
                    CoreOutput::MailDaily(rx) => handle_rx!(rx, "Failed to send daily report mail"),
                    CoreOutput::SyncAllDailyTask(rx) => {
                        handle_rx!(rx, "Failed to send daily report mail")
                    }
                    CoreOutput::SyncAllWeeklyTask(rx) => {
                        handle_rx!(rx, "Failed to send daily report mail")
                    }
                    CoreOutput::SyncAllMonthlyTask(rx) => {
                        handle_rx!(rx, "Failed to send daily report mail")
                    }
                    _ => None,
                }
            }
        };

        // Apply state transition if a new output state is determined
        if let Some(next) = next_output {
            self.work.output = next;
        }
    }
}
