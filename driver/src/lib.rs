mod task_database;
pub use task_database::fetch_all_task;
pub use task_database::fetch_one_task;
pub use task_database::get_next_task_id;
pub use task_database::get_plot_data;
pub use task_database::insert_task;
pub use task_database::search_task;
pub use task_database::update_task;
pub use task_database::upsert_task;

mod mail;
pub use mail::launch_system_mailer;

mod plot;
pub use plot::export_to_base64;

mod periodic_task;
pub use periodic_task::initialize_periodic_tasks;

pub use duckdb::Connection;

mod duckdb_task;

mod daily_task_database;
mod duckdb_daily_task;
pub use daily_task_database::delete_daily_task;
pub use daily_task_database::fetch_all_daily_task;
pub use daily_task_database::fetch_one_daily_task;
pub use daily_task_database::get_next_daily_task_id;
pub use daily_task_database::insert_daily_task;
pub use daily_task_database::search_daily_task;
pub use daily_task_database::update_daily_task;
pub use daily_task_database::upsert_daily_task;

mod connect;
pub use connect::DuckdbPath;
pub use connect::connect;
