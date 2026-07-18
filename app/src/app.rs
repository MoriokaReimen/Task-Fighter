use super::style;
use crate::i18n::I18n;
use crate::page::{self, Page, Pages};
use crate::work::Work;
use core::prelude::*;
use core::{CoreOutput, Receiver, TryRecvError};
use egui::Ui;
use jiff::Timestamp;
use std::collections::HashMap;
use tracing::{error, info, warn};

/// Main application state holder.
pub struct App {
    work: Work,
    pages: HashMap<Pages, Box<dyn Page>>,
    last_synched: Timestamp,
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
        pages.insert(Pages::MonthlyMain, Box::new(page::MonthlyMainPage::new()));
        pages.insert(
            Pages::EditMonthlyTask,
            Box::new(page::EditMonthlyTaskPage::new()),
        );
        pages.insert(
            Pages::CreateMonthlyTask,
            Box::new(page::CreateMonthlyTaskPage::new()),
        );
        pages.insert(Pages::Kanban, Box::new(page::KanbanPage::new()));
        pages.insert(Pages::Config, Box::new(page::ConfigPage::new()));

        let mut work = Work::new();
        work.config = work.core.load_config().expect("Failed to load config");
        I18n::global().set_locale_from_config(work.config.locale);
        work.outputs.push(work.core.sync_all_daily_task());
        work.outputs.push(work.core.sync_all_weekly_task());
        work.outputs.push(work.core.sync_all_monthly_task());
        let last_synched = Timestamp::now();

        Self {
            work,
            pages,
            last_synched,
        }
    }
}

impl eframe::App for App {
    /// Main UI update loop called on every frame render.
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        style::set_theme(ui.ctx(), &self.work);
        self.poll_background_tasks();

        /* Invoke on entry handler */
        if self.work.next_page != self.work.last_page {
            if let Some(page) = self.pages.get_mut(&self.work.next_page) {
                info!("{:?} entry handler called", &self.work.next_page);
                page.on_entry(&mut self.work);
            } else {
                warn!("Page not found: {:?}", self.work.next_page);
            }
            self.work.last_page = self.work.next_page;
        }

        /* Show page */
        if let Some(page) = self.pages.get_mut(&self.work.next_page) {
            page.show(ui, &mut self.work);
        } else {
            warn!("Page not found: {:?}", self.work.next_page);
        }

        /* Invoke on exit handler */
        if self.work.next_page != self.work.last_page {
            if let Some(page) = self.pages.get_mut(&self.work.last_page) {
                info!("{:?} exit handler called", &self.work.last_page);
                page.on_exit(&mut self.work);
            } else {
                warn!("Page not found: {:?}", self.work.next_page);
            }
        }

        let synch_interval = jiff::SignedDuration::from_mins(10);
        if Timestamp::now().duration_since(self.last_synched) > synch_interval {
            self.work.outputs.push(self.work.core.sync_all_daily_task());
            self.work
                .outputs
                .push(self.work.core.sync_all_weekly_task());
            self.work
                .outputs
                .push(self.work.core.sync_all_monthly_task());
            self.work.tasks = None;
            self.last_synched = Timestamp::now();
            info!("Data Synched.")
        }
    }
}

impl App {
    fn poll_background_tasks(&mut self) {
        // 1. 一時的に outputs を取り出す
        let outputs = std::mem::take(&mut self.work.outputs);

        // 2. フィルター処理を行い、結果を再代入する
        self.work.outputs = outputs
            .into_iter()
            .filter_map(|mut output| {
                match &mut output {
                    // データ更新を伴うタスク
                    CoreOutput::FetchAllTask(rx) => {
                        Self::check_rx(rx, "Failed to fetch active tasks", |d| {
                            self.work.tasks = Some(d);
                        })
                        .then_some(output)
                    }
                    CoreOutput::SearchTask(rx) => {
                        Self::check_rx(rx, "Search query failed", |d| self.work.tasks = Some(d))
                            .then_some(output)
                    }
                    CoreOutput::FetchAllDailyTask(rx) => {
                        Self::check_rx(rx, "Failed to fetch daily tasks", |d| {
                            self.work.daily_tasks = Some(d);
                        })
                        .then_some(output)
                    }
                    CoreOutput::FetchAllWeeklyTask(rx) => {
                        Self::check_rx(rx, "Failed to fetch weekly tasks", |d| {
                            self.work.weekly_tasks = Some(d);
                        })
                        .then_some(output)
                    }
                    CoreOutput::FetchAllMonthlyTask(rx) => {
                        Self::check_rx(rx, "Failed to fetch monthly tasks", |d| {
                            self.work.monthly_tasks = Some(d);
                        })
                        .then_some(output)
                    }
                    CoreOutput::PlotData(rx) => {
                        Self::check_rx(rx, "Plot Data failed", |d| self.work.plot_data = Some(d))
                            .then_some(output)
                    }

                    // 副作用のみのタスク
                    CoreOutput::InsertTask(rx) => {
                        Self::check_rx(rx, "Failed to insert task", |()| {}).then_some(output)
                    }
                    CoreOutput::UpsertTask(rx) => {
                        Self::check_rx(rx, "Failed to insert task", |()| {}).then_some(output)
                    }
                    CoreOutput::FetchOneTask(rx) => {
                        Self::check_rx(rx, "Failed to fetch task by ID", |_| {}).then_some(output)
                    }
                    CoreOutput::UpdateTask(rx) => {
                        Self::check_rx(rx, "Failed to update task", |()| {}).then_some(output)
                    }
                    CoreOutput::MailDaily(rx) => {
                        Self::check_rx(rx, "Failed to send daily report mail", |()| {})
                            .then_some(output)
                    }
                    CoreOutput::SyncAllDailyTask(rx) => {
                        Self::check_rx(rx, "Failed to sync daily tasks", |()| {}).then_some(output)
                    }
                    CoreOutput::SyncAllWeeklyTask(rx) => {
                        Self::check_rx(rx, "Failed to sync weekly tasks", |()| {}).then_some(output)
                    }
                    CoreOutput::SyncAllMonthlyTask(rx) => {
                        Self::check_rx(rx, "Failed to sync monthly tasks", |()| {})
                            .then_some(output)
                    }
                    other => {
                        warn!("{:?}", other);
                        None
                    }
                }
            })
            .collect();
    }

    fn check_rx<T, E>(
        rx: &mut Receiver<Result<T, E>>,
        err_msg: &str,
        on_success: impl FnOnce(T),
    ) -> bool
    where
        E: std::fmt::Debug,
    {
        match rx.try_recv() {
            Ok(Ok(data)) => {
                on_success(data);
                false // 完了したため残さない
            }
            Ok(Err(e)) => {
                error!("{}: {:?}", err_msg, e);
                false // エラー終了のため残さない
            }
            Err(TryRecvError::Empty) => {
                true // まだ実行中なので残す
            }
            Err(TryRecvError::Closed) => {
                error!("Channel disconnected unexpectedly ({})", err_msg);
                false // 切断されたため残さない
            }
        }
    }
}
