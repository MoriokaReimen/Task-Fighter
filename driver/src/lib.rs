mod connect;
pub use connect::*;
pub use duckdb::Connection;

mod migrations;

mod duckdb_task;
mod task_record;
pub use task_record::*;

mod mail;
pub use mail::*;

mod plot;
pub use plot::*;

mod daily_task_record;
mod duckdb_daily_task;
pub use daily_task_record::*;

mod duckdb_weekly_task;
mod weekly_task_record;
pub use weekly_task_record::*;

mod duckdb_monthly_task;
mod monthly_task_record;
pub use monthly_task_record::*;

mod duckdb_relation;
mod relation_record;
pub use relation_record::*;

mod duckdb_work_time;
mod work_time_record;
pub use work_time_record::*;

mod config_record;
mod duckdb_config;
pub use config_record::*;
