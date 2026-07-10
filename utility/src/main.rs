use anyhow::{Context, Result};
use domain::{TaskPriority, TaskStatus};
use duckdb::{Connection, params};
use jiff::civil::Date;

/// `end_dateも含めたランダムなN件のタスクを生成してデータベースに挿入する`
pub fn generate_random_tasks(conn: &Connection, n: usize) -> Result<()> {
    let projects = [
        "RustProject",
        "Frontend",
        "Backend",
        "DevOps",
        "Marketing",
        "Research",
    ];
    let details = [
        "割り当てられたチケットの確認と修正対応を行う。",
        "コードの可読性向上のため、関数を分割して整理する。",
        "次のスプリントに向けた技術的な実現可能性を調査する。",
        "フロントエンドからの要求に基づき、Swaggerを更新して実装する。",
    ];
    let statuses = [
        TaskStatus::Pending,
        TaskStatus::WorkInProgress,
        TaskStatus::Complete,
        TaskStatus::Canceled,
    ];
    let priorities = [TaskPriority::Low, TaskPriority::Medium, TaskPriority::High];

    println!("{n} 件のランダムタスク（end_date対応）を生成中...");

    let sql = "INSERT INTO tasks (active, status, project, title, detail, start_date, due_date, priority, progress, time_spent, entry_date, end_date) 
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)";

    for i in 0..n {
        // 💡 メソッド呼び出しによるトレイト判定エラーを避けるため、ランダムなインデックスを数値型で取得します
        let project_idx = rand::random_range(0..projects.len());
        let project = projects[project_idx].to_string();

        let title_idx = rand::random_range(0..projects.len());
        let title = format!("{} #{}", projects[title_idx], i + 1);

        let detail_idx = rand::random_range(0..details.len());
        let detail = details[detail_idx].to_string();

        let status_idx = rand::random_range(0..statuses.len());
        let status = statuses[status_idx];

        let priority_idx = rand::random_range(0..priorities.len());
        let priority = priorities[priority_idx];

        let start_day = rand::random_range(1..=28);
        let start_month = rand::random_range(1..=5);
        let start_date = Date::new(2026, start_month, start_day).unwrap();

        let duration = rand::random_range(1..=14);
        let due_date = start_date
            .checked_add(jiff::ToSpan::days(duration))
            .unwrap();

        let entry_date = start_date;

        let progress = match status {
            TaskStatus::Pending => 0.0,
            TaskStatus::Complete => 100.0,
            TaskStatus::Canceled => rand::random_range(0.0..50.0),
            TaskStatus::WorkInProgress => rand::random_range(5.0..95.0),
        };
        let time_spent = if progress > 0.0 {
            rand::random_range(0.5..24.0)
        } else {
            0.0
        };

        let end_date_str: Option<String> =
            if status == TaskStatus::Complete || status == TaskStatus::Canceled {
                let days_to_complete = rand::random_range(1..=(duration + 2));
                let end_date = start_date
                    .checked_add(jiff::ToSpan::days(days_to_complete))
                    .unwrap();
                Some(end_date.to_string())
            } else {
                None
            };

        let active = status != TaskStatus::Complete && status != TaskStatus::Canceled;

        conn.execute(
            sql,
            params![
                active,
                status as i32,
                project,
                title,
                detail,
                start_date.to_string(),
                due_date.to_string(),
                priority as i32,
                progress,
                time_spent,
                entry_date.to_string(),
                end_date_str,
            ],
        )
        .context(format!(
            "Failed to insert random task with end_date at index {i}"
        ))?;
    }

    println!("正常に end_date を含む {n} 件のタスクデータが生成されました。");
    Ok(())
}

fn main() -> Result<()> {
    let path = driver::DuckdbPath::InDirectory("./runtime".into());
    let conn = driver::connect(&path)?;

    let num_records = 100;
    generate_random_tasks(&conn, num_records)?;

    Ok(())
}
