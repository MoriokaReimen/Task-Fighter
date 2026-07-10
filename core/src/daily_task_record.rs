use crate::{Core, CoreOutput};
use anyhow::Result;
use domain::prelude::*;
use domain::{DailyTask, DailyTaskFilterFlags, DailyTaskOrderFlags, DailyTaskSearchFlags};
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

impl DailyTaskRecord for Core {
    type AsyncOutput = CoreOutput;

    fn get_next_daily_task_id(&self) -> Result<i32> {
        self.runtime.block_on(async {
            let conn = self.conn.lock().await;
            driver::get_next_daily_task_id(&conn)
        })
    }

    fn fetch_one_daily_task(&self, id: i32) -> Self::AsyncOutput {
        spawn_async_db!(self, FetchOneDailyTask, |conn| {
            driver::fetch_one_daily_task(conn, id)
        })
    }

    fn fetch_all_daily_task(
        &self,
        filter_flags: DailyTaskFilterFlags,
        order_flags: DailyTaskOrderFlags,
    ) -> Self::AsyncOutput {
        spawn_async_db!(self, FetchAllDailyTask, |c| {
            driver::fetch_all_daily_task(c, filter_flags, order_flags)
        })
    }

    fn search_daily_task(
        &self,
        pattern: &str,
        search_flags: DailyTaskSearchFlags,
        filter_flags: DailyTaskFilterFlags,
        order_flags: DailyTaskOrderFlags,
    ) -> Self::AsyncOutput {
        let pattern = pattern.to_string();
        spawn_async_db!(self, SearchDailyTask, |c| {
            driver::search_daily_task(c, &pattern, search_flags, filter_flags, order_flags)
        })
    }

    fn insert_daily_task(&self, daily_task: &DailyTask) -> Self::AsyncOutput {
        let daily_task = daily_task.clone();
        spawn_async_db!(self, InsertTask, |conn| driver::insert_daily_task(
            conn,
            &daily_task
        ))
    }

    fn update_daily_task(&self, daily_task: &DailyTask) -> Self::AsyncOutput {
        let daily_task = daily_task.clone();
        spawn_async_db!(self, UpdateTask, |c| driver::update_daily_task(
            c,
            &daily_task
        ))
    }

    fn upsert_daily_task(&self, daily_task: &DailyTask) -> Self::AsyncOutput {
        let daily_task = daily_task.clone();
        spawn_async_db!(self, UpsertTask, |c| driver::upsert_daily_task(
            c,
            &daily_task
        ))
    }

    fn delete_daily_task(&self, id: i32) -> Self::AsyncOutput {
        spawn_async_db!(self, DeleteDailyTask, |c| {
            driver::delete_daily_task(c, id)
        })
    }

    fn sync_all_daily_task(&self) -> Self::AsyncOutput {
        spawn_async_db!(self, SyncAllDailyTask, |c| {
            // 即時実行クロージャを使い、内部で `?` によるクリーンな早期リターンを可能にします
            let result: Result<()> = (|| {
                let filter_flags = DailyTaskFilterFlags::All;
                let order_flags = DailyTaskOrderFlags::default();
                let daily_tasks = driver::fetch_all_daily_task(c, filter_flags, order_flags)?;
                let today = Zoned::now().date();

                for daily_task in daily_tasks {
                    let task = daily_task.create_task(&today)?;
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
                    let is_already_exists = !existing_tasks.is_empty();

                    if !is_already_exists {
                        // 3. 未登録ならインサートを実行
                        driver::insert_task(c, &task)?;
                    }
                }

                Ok(())
            })();

            // マクロの $action の最終評価値として Result<()> を渡す
            // これにより、マクロ内部の `tx.send(result)` を経由して `CoreOutput::SyncAllDailyTask(rx)` へ正しく送信されます
            result
        })
    }
}
