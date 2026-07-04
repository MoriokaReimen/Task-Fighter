use crate::{Priority, Task, insert_task};
use anyhow::{Context, Result, bail};
use duckdb::{Connection, params};
use jiff::ToSpan;
use jiff::Zoned;
use jiff::civil::{Date, Weekday};
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
    let year = current_month_first_day.strftime("%Y").to_string();
    let title_with_month = format!("{} for {} {}", periodic_task.title, month_name, year);

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
        let template = include_str!("../assets/config.toml");
        if let Some(parent) = config_path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
            info!("Created directory: {:?}", parent);
        }
        fs::write(config_path, template)?;
        info!("Generated default config file at {:?}", config_path);
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
