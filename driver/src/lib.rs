mod data;
pub use data::Priority;
pub use data::Task;
pub use data::TaskStatus;

mod database;
pub use database::connect;
pub use database::count_tasks_by_date;
pub use database::fetch_active_tasks;
pub use database::get_next_id;
pub use database::insert_task;
pub use database::scan_tasks;
pub use database::update_task;
pub use database::upsert_task;

mod mail;
pub use mail::launch_system_mailer;

mod plot;
pub use plot::export_to_base64;

mod periodic_task;
pub use periodic_task::initialize_periodic_tasks;

pub use duckdb::Connection;

mod duckdb_data;
