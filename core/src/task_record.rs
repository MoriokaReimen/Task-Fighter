use crate::{Core, CoreOutput};
use anyhow::Result;
use domain::prelude::*;
use domain::{Task, TaskFilterFlags, TaskOrderFlags, TaskSearchFlags};
use jiff::{ToSpan, Zoned};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::oneshot::{self};
use uuid::Uuid;

macro_rules! spawn_async_db {
    ($self:expr, $output_variant:ident, |$conn:ident| $action:expr) => {{
        let conn = Arc::clone(&$self.conn);
        let (tx, rx) = oneshot::channel();

        let handle = $self.runtime.handle().clone();

        $self.runtime.spawn_blocking(move || {
            let conn_guard = handle.block_on(async { conn.lock().await });
            let $conn = &*conn_guard;
            let result = $action;
            drop(conn_guard);

            let _ = tx.send(result);
        });

        CoreOutput::$output_variant(rx)
    }};
}

impl TaskRecord for Core {
    type AsyncOutput = CoreOutput;

    fn fetch_one_task(&self, uuid: Uuid) -> Self::AsyncOutput {
        spawn_async_db!(self, FetchOneTask, |conn| {
            driver::fetch_one_task(conn, uuid)
        })
    }

    fn fetch_all_task(
        &self,
        filter_flags: TaskFilterFlags,
        order_flags: TaskOrderFlags,
    ) -> Self::AsyncOutput {
        spawn_async_db!(self, FetchAllTask, |c| {
            driver::fetch_all_task(c, filter_flags, order_flags)
        })
    }

    fn search_task(
        &self,
        pattern: &str,
        search_flags: TaskSearchFlags,
        filter_flags: TaskFilterFlags,
        order_flags: TaskOrderFlags,
    ) -> Self::AsyncOutput {
        let pattern = pattern.to_string();
        spawn_async_db!(self, SearchTask, |c| {
            driver::search_task(c, &pattern, search_flags, filter_flags, order_flags)
        })
    }

    fn insert_task(&self, task: &Task) -> Self::AsyncOutput {
        let task = task.clone();
        spawn_async_db!(self, InsertTask, |conn| driver::insert_task(conn, &task))
    }

    fn update_task(&self, task: &Task) -> Self::AsyncOutput {
        let task = task.clone();
        spawn_async_db!(self, UpdateTask, |c| driver::update_task(c, &task))
    }

    fn upsert_task(&self, task: &Task) -> Self::AsyncOutput {
        let task = task.clone();
        spawn_async_db!(self, UpsertTask, |c| driver::upsert_task(c, &task))
    }

    fn get_plot_data(&self) -> Self::AsyncOutput {
        spawn_async_db!(self, PlotData, |c| {
            let today = Zoned::now().date();
            let start_date = today - 99.days();
            driver::get_plot_data(c, start_date, today)
        })
    }

    fn mail_daily(&self, tasks: &[Task]) -> Self::AsyncOutput {
        let conn = Arc::clone(&self.conn);
        let (tx, rx) = oneshot::channel();
        let tasks = tasks.to_vec();
        let handle = self.runtime.handle().clone();

        self.runtime.spawn_blocking(move || {
            let result = (|| -> Result<()> {
                let today = Zoned::now().date();
                let start_date = today - 99.days();

                let conn_guard = handle.block_on(async { conn.lock().await });
                let data = driver::get_plot_data(&conn_guard, start_date, today)?;
                let config = driver::load_config(&conn_guard)?;
                drop(conn_guard);

                let image_data = driver::export_to_base64(&data)?;
                driver::launch_system_mailer(&tasks, &image_data, &config)?;
                Ok(())
            })();
            let _ = tx.send(result);
        });

        CoreOutput::MailDaily(rx)
    }

    fn export_markdown(&self, output: &Path, tasks: &[Task]) -> Self::AsyncOutput {
        let (tx, rx) = oneshot::channel();
        let tasks = tasks.to_vec();
        let output = output.to_path_buf();

        self.runtime.spawn_blocking(move || {
            let result = (|| -> Result<()> {
                driver::export_markdown(&output, &tasks)?;
                Ok(())
            })();
            let _ = tx.send(result);
        });

        CoreOutput::ExportMarkdown(rx)
    }
}
