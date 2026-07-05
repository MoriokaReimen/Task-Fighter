mod task;
pub use task::{Task, TaskPriority, TaskStatus};
pub use task::{TaskFilterFlags, TaskOrderFlags, TaskSearchFlags};

mod database;
pub use database::DuckdbPath;
pub use database::connect;
pub use database::fetch_all_task;
pub use database::fetch_one_task;
pub use database::get_next_id;
pub use database::get_plot_data;
pub use database::insert_task;
pub use database::search_task;
pub use database::update_task;
pub use database::upsert_task;

mod mail;
pub use mail::launch_system_mailer;

mod plot;
pub use plot::export_to_base64;

mod periodic_task;
pub use periodic_task::initialize_periodic_tasks;

pub use duckdb::Connection;

mod duckdb_task;
