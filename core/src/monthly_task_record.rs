use crate::{Core, CoreOutput};
use anyhow::Result;
use domain::prelude::*;
use domain::{MonthlyTask, MonthlyTaskFilterFlags, MonthlyTaskOrderFlags, MonthlyTaskSearchFlags};
use domain::{TaskFilterFlags, TaskOrderFlags, TaskSearchFlags};
use jiff::Zoned;
use std::sync::Arc;
use tokio::sync::oneshot::{self};

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

impl MonthlyTaskRecord for Core {
    type AsyncOutput = CoreOutput;

    fn get_next_monthly_task_id(&self) -> Result<i32> {
        self.runtime.block_on(async {
            let conn = self.conn.lock().await;
            driver::get_next_monthly_task_id(&conn)
        })
    }

    fn fetch_one_monthly_task(&self, id: i32) -> Self::AsyncOutput {
        spawn_async_db!(self, FetchOneMonthlyTask, |conn| {
            driver::fetch_one_monthly_task(conn, id)
        })
    }

    fn fetch_all_monthly_task(
        &self,
        filter_flags: MonthlyTaskFilterFlags,
        order_flags: MonthlyTaskOrderFlags,
    ) -> Self::AsyncOutput {
        spawn_async_db!(self, FetchAllMonthlyTask, |c| {
            driver::fetch_all_monthly_task(c, filter_flags, order_flags)
        })
    }

    fn search_monthly_task(
        &self,
        pattern: &str,
        search_flags: MonthlyTaskSearchFlags,
        filter_flags: MonthlyTaskFilterFlags,
        order_flags: MonthlyTaskOrderFlags,
    ) -> Self::AsyncOutput {
        let pattern = pattern.to_string();
        spawn_async_db!(self, SearchMonthlyTask, |c| {
            driver::search_monthly_task(c, &pattern, search_flags, filter_flags, order_flags)
        })
    }

    fn insert_monthly_task(&self, monthly_task: &MonthlyTask) -> Self::AsyncOutput {
        let monthly_task = monthly_task.clone();
        spawn_async_db!(self, InsertMonthlyTask, |conn| driver::insert_monthly_task(
            conn,
            &monthly_task
        ))
    }

    fn update_monthly_task(&self, monthly_task: &MonthlyTask) -> Self::AsyncOutput {
        let monthly_task = monthly_task.clone();
        spawn_async_db!(self, UpdateMonthlyTask, |c| driver::update_monthly_task(
            c,
            &monthly_task
        ))
    }

    fn upsert_monthly_task(&self, monthly_task: &MonthlyTask) -> Self::AsyncOutput {
        let monthly_task = monthly_task.clone();
        spawn_async_db!(self, UpsertMonthlyTask, |c| driver::upsert_monthly_task(
            c,
            &monthly_task
        ))
    }

    fn delete_monthly_task(&self, id: i32) -> Self::AsyncOutput {
        spawn_async_db!(self, DeleteMonthlyTask, |c| {
            driver::delete_monthly_task(c, id)
        })
    }

    fn sync_all_monthly_task(&self) -> Self::AsyncOutput {
        spawn_async_db!(self, SyncAllMonthlyTask, |c| {
            let result: Result<()> = (|| {
                let filter_flags = MonthlyTaskFilterFlags::All;
                let order_flags = MonthlyTaskOrderFlags::default();
                let monthly_tasks = driver::fetch_all_monthly_task(c, filter_flags, order_flags)?;
                let today = Zoned::now().date();

                for monthly_task in monthly_tasks {
                    if !monthly_task.active {
                        continue;
                    }
                    let task = monthly_task.create_task(&today)?;
                    let filter_flags = TaskFilterFlags::All;
                    let order_flags = TaskOrderFlags::default();
                    let search_flags = TaskSearchFlags::default();
                    let existing_tasks = driver::search_task(
                        c,
                        &task.title,
                        search_flags,
                        filter_flags,
                        order_flags,
                    )?;

                    if existing_tasks.is_empty() {
                        driver::insert_task(c, &task)?;
                    }
                }

                Ok(())
            })();

            result
        })
    }
}
