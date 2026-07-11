use crate::{Core, CoreOutput};
use anyhow::Result;
use domain::prelude::*;
use domain::{MonthlyTask, MonthlyTaskFilterFlags, MonthlyTaskOrderFlags, MonthlyTaskSearchFlags};
use domain::{TaskFilterFlags, TaskOrderFlags, TaskSearchFlags};
use jiff::Zoned;
use std::sync::Arc;
use tokio::sync::oneshot::{self};

/// `tokio::sync::Mutex` をブロッキングタスク内で安全に取得・処理するマクロ
macro_rules! spawn_async_db {
    ($self:expr, $output_variant:ident, |$conn:ident| $action:expr) => {{
        let conn = Arc::clone(&$self.conn);
        let (tx, rx) = oneshot::channel();

        // spawn_blocking 内から非同期Mutexにアクセスするためのハンドルを取得
        let handle = $self.runtime.handle().clone();

        $self.runtime.spawn_blocking(move || {
            // ブロッキングスレッド内で非同期Mutexをロックする
            let conn_guard = handle.block_on(async { conn.lock().await });
            let $conn = &*conn_guard;
            let result = $action;

            // スレッドを抜ける前に確実にガードをドロップ（解放）
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
        spawn_async_db!(self, InsertTask, |conn| driver::insert_monthly_task(
            conn,
            &monthly_task
        ))
    }

    fn update_monthly_task(&self, monthly_task: &MonthlyTask) -> Self::AsyncOutput {
        let monthly_task = monthly_task.clone();
        spawn_async_db!(self, UpdateTask, |c| driver::update_monthly_task(
            c,
            &monthly_task
        ))
    }

    fn upsert_monthly_task(&self, monthly_task: &MonthlyTask) -> Self::AsyncOutput {
        let monthly_task = monthly_task.clone();
        spawn_async_db!(self, UpsertTask, |c| driver::upsert_monthly_task(
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
            // 即時実行クロージャを使い、内部で `?` によるクリーンな早期リターンを可能にします
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
                    // 2. 既存のタスク一覧を取得
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
                        // 3. 未登録ならインサートを実行
                        driver::insert_task(c, &task)?;
                    }
                }

                Ok(())
            })();

            // マクロの $action の最終評価値として Result<()> を渡す
            // これにより、マクロ内部の `tx.send(result)` を経由して `CoreOutput::SyncAllMonthlyTask(rx)` へ正しく送信されます
            result
        })
    }
}
