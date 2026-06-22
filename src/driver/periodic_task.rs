use crate::driver::{Priority, Task, insert_task};
use anyhow::{Context, Result, bail};
use jiff::ToSpan;
use jiff::Zoned;
use jiff::civil::{Date, Weekday};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub enum Period {
    #[default]
    Daily = 0,
    Weekly = 1,
    Monthly = 2,
}

impl TryFrom<i32> for Period {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Period::Daily),
            1 => Ok(Period::Weekly),
            2 => Ok(Period::Monthly),
            _ => bail!("Invalid period integer state: {}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PeriodicTask {
    pub period: Period,
    pub project: String,
    pub title: String,
    pub detail: String,
    pub priority: Priority,
}

#[derive(Debug, Deserialize)]
struct TaskList {
    tasks: Vec<PeriodicTask>,
}

impl Default for PeriodicTask {
    fn default() -> Self {
        Self {
            period: Period::Daily,
            project: String::new(),
            title: String::new(),
            detail: String::new(),
            priority: Priority::Low,
        }
    }
}

/// Evaluates and initializes standard daily tracking rows if absent for the current calendar date.
fn initialize_daily_task(
    conn: &Connection,
    periodic_task: &PeriodicTask,
    today: Date,
) -> Result<()> {
    let date_str = today.strftime("%Y/%m/%d").to_string();
    let title_with_date = format!("{} for {}", periodic_task.title, date_str);
    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM tasks WHERE title = ?1")
        .context("Failed compiling matrix query statement for periodic routine validation")?;

    let count: i32 = stmt
        .query_row(params![title_with_date], |row| row.get(0))
        .context("Failed executing row validation checks for periodic routines")?;
    if count > 0 {
        return Ok(());
    }

    info!(
        "No daily tracking task detected. Injecting {:?}",
        periodic_task
    );
    insert_task(
        conn,
        &Task {
            project: periodic_task.project.clone(),
            title: title_with_date,
            detail: periodic_task.detail.clone(),
            start_date: today,
            due_date: today,
            priority: periodic_task.priority,
            ..Default::default()
        },
    )
    .context("Failed committing automated daily routine token item")?;

    Ok(())
}

/// Evaluates and initializes weekly milestone tracking boundaries anchored to the current week's Monday state.
fn initialize_weekly_task(
    conn: &Connection,
    periodic_task: &PeriodicTask,
    today: Date,
) -> Result<()> {
    // 2. ⚠️ Jiffに存在しない .yesterday() を、安全な .tomorrow()?.nth_weekday(-1, ...) へ修正
    let current_week_monday = today
        .tomorrow()?
        .nth_weekday(-1, Weekday::Monday)
        .context("Failed calculating current Monday of current week.")?;

    // 3. 日曜日も同様に、明日から過去に遡って最も近い日曜日（今週の日曜日）を取得
    let current_week_sunday = today
        .tomorrow()?
        .nth_weekday(1, Weekday::Sunday)
        .context("Failed calculating current Sunday of current week.")?;

    let month_name = current_week_monday.strftime("%B").to_string();
    let week_of_month = ((current_week_monday.day() - 1) / 7) + 1;
    let title_with_week = format!(
        "{} for {} Week {}",
        periodic_task.title, month_name, week_of_month
    );

    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM tasks WHERE title = ?1")
        .context("Failed compiling matrix query statement for periodic routine validation")?;
    let count: i32 = stmt
        .query_row(params![title_with_week], |row| row.get(0))
        .context("Failed executing row validation checks for periodic routines")?;
    if count > 0 {
        return Ok(());
    }
    info!(
        "No weekly tracking task detected. Injecting {:?}",
        periodic_task
    );
    insert_task(
        conn,
        &Task {
            project: periodic_task.project.clone(),
            title: title_with_week,
            detail: periodic_task.detail.clone(),
            start_date: current_week_monday,
            due_date: current_week_sunday,
            priority: periodic_task.priority,
            ..Default::default()
        },
    )
    .context("Failed committing automated weekly routine token item")?;

    Ok(())
}

/// Evaluates and initializes monthly strategic epic buckets anchored to the first day of the current month.
fn initialize_monthly_task(
    conn: &Connection,
    periodic_task: &PeriodicTask,
    today: Date,
) -> Result<()> {
    let current_month_first_day = today
        .with()
        .day(1)
        .build()
        .context("Failed calculating the first day in current month")?;

    let current_month_final_day = today
        .with()
        .day(1)
        .build()
        .context("Failed calculating the final day in current month")?
        .checked_add(1.months())
        .context("Failed to advance to the next month")?
        .checked_sub(1.days())
        .context("Failed to compute the last day of the current month")?;

    let month_name = current_month_first_day.strftime("%B").to_string();
    let title_with_month = format!("{} for {}", periodic_task.title, month_name);

    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM tasks WHERE title = ?1")
        .context("Failed compiling matrix query statement for periodic routine validation")?;
    let count: i32 = stmt
        .query_row(params![title_with_month], |row| row.get(0))
        .context("Failed executing row validation checks for periodic routines")?;
    if count > 0 {
        return Ok(());
    }
    info!(
        "No monthly tracking task detected. Injecting {:?}",
        periodic_task
    );
    insert_task(
        conn,
        &Task {
            project: periodic_task.project.clone(),
            title: title_with_month,
            detail: periodic_task.detail.clone(),
            start_date: current_month_first_day,
            due_date: current_month_final_day,
            priority: periodic_task.priority,
            ..Default::default()
        },
    )
    .context("Failed committing automated monthly routine token item")?;

    Ok(())
}

/// Dispatches calls down to specific timeframe slice execution subroutines cleanly.
pub fn initialize_periodic_tasks(conn: &Connection) -> Result<()> {
    let config_path = Path::new("./runtime/config.toml");
    if !config_path.is_file() {
        warn!("Config file not found in {:?}", config_path);
        warn!("Skipped periodic task creation.");
        return Ok(());
    }

    let content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read TOML file at: {:?}", config_path))?;
    let data: TaskList = toml::from_str(&content)
        .with_context(|| "Failed to parse PeriodicTask matrix from TOML".to_string())?;

    let today = Zoned::now().date();
    info!("Initialize periodic tasks on date: {:?}", today);

    // 4. ⚠️ 途切れていたマッチングとループ処理のクロージャを完結
    for periodic_task in data.tasks {
        match periodic_task.period {
            Period::Daily => initialize_daily_task(conn, &periodic_task, today)?,
            Period::Weekly => initialize_weekly_task(conn, &periodic_task, today)?,
            Period::Monthly => initialize_monthly_task(conn, &periodic_task, today)?,
        }
    }

    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::Date;

    /// Mock utility to establish a standard transient database table context layout.
    fn setup_mock_db() -> Result<Connection> {
        let conn = Connection::open_in_memory()
            .context("Failed to spin up transient in-memory validation database storage context")?;

        conn.execute(
            "CREATE TABLE tasks (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                active      INTEGER NOT NULL DEFAULT 1,
                status      INTEGER NOT NULL DEFAULT 0,
                project     TEXT NOT NULL,
                title       TEXT NOT NULL,
                detail      TEXT NOT NULL,
                start_date  DATETIME NOT NULL,
                due_date    DATETIME NOT NULL,
                priority    INTEGER NOT NULL DEFAULT 1,
                progress    REAL NOT NULL DEFAULT 0.0,
                time_spent  REAL NOT NULL DEFAULT 0.0
            );",
            [],
        )
        .context("Failed executing transient validation layout table migration sequences")?;

        Ok(conn)
    }

    /// Prepares a safe temporary directory containing a mock config.toml configuration target payload.
    fn write_test_config(content: &str) -> Result<tempfile::TempDir> {
        let temp_dir =
            tempfile::tempdir().context("Failed creating secure storage anchor path context")?;
        let runtime_path = temp_dir.path().join("runtime");
        fs::create_dir_all(&runtime_path)
            .context("Failed creating structural testing subdirectories chains")?;

        let file_path = runtime_path.join("config.toml");
        fs::write(file_path, content).context(
            "Failed flushing temporary mock profile data rows down onto local disk segments",
        )?;

        Ok(temp_dir)
    }

    #[test]
    fn test_initialize_periodic_tasks_missing_config_graceful_skips() -> Result<()> {
        let conn = setup_mock_db()?;

        // Temporarily change directory or ensure no file conflicts are present locally.
        // If config.toml is missing, it logs a warning and returns Ok(()) without failure.
        let _temp_dir = tempfile::tempdir()?;
        let previous_dir = std::env::current_dir()?;
        std::env::set_current_dir(_temp_dir.path())?;

        let result = initialize_periodic_tasks(&conn);
        assert!(result.is_ok());

        // Restore active path context tracking fields cleanly
        std::env::set_current_dir(previous_dir)?;
        Ok(())
    }

    #[test]
    fn test_periodic_tasks_generation_and_idempotency_flow() -> Result<()> {
        let conn = setup_mock_db()?;

        // Define a comprehensive mix of periodic configurations
        let mock_toml = r#"
            [[tasks]]
            period = "Daily"
            project = "DailyRoutine"
            title = "Morning Sync"
            detail = "Review personal backlog tickets stack"
            priority = "Low"

            [[tasks]]
            period = "Weekly"
            project = "WeeklyReview"
            title = "Sprint Planning"
            detail = "Align tactical milestones configurations"
            priority = "Medium"

            [[tasks]]
            period = "Monthly"
            project = "MonthlyOverview"
            title = "Financial Audit"
            detail = "Verify cash flow statements indicators ratios"
            priority = "High"
        "#;

        let temp_dir = write_test_config(mock_toml)?;
        let previous_dir = std::env::current_dir()?;
        std::env::set_current_dir(temp_dir.path())?;

        // 1. Initial invocation loop boundary segment execution
        initialize_periodic_tasks(&conn)
            .context("Failed processing baseline transactional pipeline sweeps")?;

        let mut stmt =
            conn.prepare("SELECT title, project, start_date, due_date FROM tasks ORDER BY id ASC")?;
        let mut rows = stmt.query([])?;

        // Assert Daily Generation Bounds
        let row_d = rows
            .next()?
            .expect("Missing generated daily task element entry metadata");
        let t_d: String = row_d.get(0)?;
        assert!(t_d.starts_with("Morning Sync for "));

        // Assert Weekly Generation Bounds
        let row_w = rows
            .next()?
            .expect("Missing generated weekly task element entry metadata");
        let t_w: String = row_w.get(0)?;
        assert!(t_w.starts_with("Sprint Planning for "));

        // Assert Monthly Generation Bounds
        let row_m = rows
            .next()?
            .expect("Missing generated monthly task element entry metadata");
        let t_m: String = row_m.get(0)?;
        assert!(t_m.starts_with("Financial Audit for "));

        // 2. Second invocation must trigger early return guards bypassing duplicate writes (Idempotency)
        initialize_periodic_tasks(&conn)
            .context("Failed secondary verification pipeline tracking sweeps")?;

        let count: i32 = conn.query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))?;
        assert_eq!(
            count, 3,
            "Idempotency invariant breached: Duplicated periodic entries found inside tables storage."
        );

        std::env::set_current_dir(previous_dir)?;
        Ok(())
    }

    #[test]
    fn test_internal_date_calculation_subroutines() -> Result<()> {
        let conn = setup_mock_db()?;

        let p_task = PeriodicTask {
            period: Period::Weekly,
            project: "Core".to_string(),
            title: "Weekly Baseline Sync".to_string(),
            detail: "Sync status parameters templates".to_string(),
            priority: Priority::Medium,
        };

        // Fix an arbitrary date target context: June 23, 2026 is a Tuesday.
        let mock_tuesday = Date::new(2026, 6, 23).unwrap();

        initialize_weekly_task(&conn, &p_task, mock_tuesday)?;

        let mut stmt =
            conn.prepare("SELECT start_date, due_date FROM tasks WHERE project = 'Core'")?;
        let (start, due): (Date, Date) =
            stmt.query_row([], |row| Ok((row.get(0)?, row.get(1)?)))?;

        // The week's Monday anchor must correctly point back to June 22
        assert_eq!(start, Date::new(2026, 6, 22).unwrap());
        // The week's Sunday anchor must correctly stretch down to June 28
        assert_eq!(due, Date::new(2026, 6, 28).unwrap());

        Ok(())
    }
}
