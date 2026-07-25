use anyhow::{Context, Result};
use domain::{Task, TaskPriority, TaskStatus};
use jiff::Zoned;
use minijinja::{Environment, context};
use serde::Serialize;
use std::fs;
use std::path::Path;
use tracing::info;

#[derive(Serialize, Default)]
struct TemplateSummary {
    total: usize,
    completed: usize,
    in_progress: usize,
    pending: usize,
    canceled: usize,
}

impl TemplateSummary {
    fn from_tasks(tasks: &[Task]) -> Self {
        tasks.iter().fold(
            Self {
                total: tasks.len(),
                ..Self::default()
            },
            |mut summary, task| {
                match task.status {
                    TaskStatus::Complete => summary.completed += 1,
                    TaskStatus::WorkInProgress => summary.in_progress += 1,
                    TaskStatus::Pending => summary.pending += 1,
                    TaskStatus::Canceled => summary.canceled += 1,
                }
                summary
            },
        )
    }
}

#[derive(Serialize)]
struct TemplateTask {
    id: i32,
    title: String,
    project: String,
    priority_text: &'static str,
    priority_color: &'static str,
    status_text: &'static str,
    status_color: &'static str,
    start_date: String,
    due_date: String,
    time_spent: f64,
    progress: f64,
    detail: String,
}

/// ステータスに対応する表示テキストと色を返す。
const fn status_display(status: TaskStatus) -> (&'static str, &'static str) {
    match status {
        TaskStatus::Pending => ("Pending ⏳", "#6c757d"),
        TaskStatus::WorkInProgress => ("Work In Progress 🏃", "#007bff"),
        TaskStatus::Complete => ("Complete ✅", "#28a745"),
        TaskStatus::Canceled => ("Canceled 🚫", "#ff5733"),
    }
}

/// 優先度に対応する表示テキストと色を返す。
const fn priority_display(priority: TaskPriority) -> (&'static str, &'static str) {
    match priority {
        TaskPriority::High => ("🔴 High", "#dc3545"),
        TaskPriority::Medium => ("🟡 Medium", "#ffc107"),
        TaskPriority::Low => ("🔵 Low", "#20c997"),
    }
}

const NO_DETAIL_TEXT: &str = "**No additional details provided.**";
const DATE_FORMAT: &str = "%Y/%m/%d";
const HEADLINE_DATE_FORMAT: &str = "%B %d, %Y";

impl From<&Task> for TemplateTask {
    fn from(task: &Task) -> Self {
        let (status_text, status_color) = status_display(task.status);
        let (priority_text, priority_color) = priority_display(task.priority);
        let detail = if task.detail.trim().is_empty() {
            NO_DETAIL_TEXT.to_string()
        } else {
            task.detail.clone()
        };

        Self {
            id: task.id,
            title: task.title.clone(),
            project: task.project.clone(),
            priority_text,
            priority_color,
            status_text,
            status_color,
            start_date: task.start_date.strftime(DATE_FORMAT).to_string(),
            due_date: task.due_date.strftime(DATE_FORMAT).to_string(),
            time_spent: f64::from(task.time_spent),
            progress: f64::from(task.progress),
            detail,
        }
    }
}

/// 改行コードをCRLFに統一する。
fn to_crlf(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\n', "\r\n")
}

pub fn create_markdown(tasks: &[Task]) -> Result<String> {
    let summary = TemplateSummary::from_tasks(tasks);
    let date_headline = Zoned::now()
        .date()
        .strftime(HEADLINE_DATE_FORMAT)
        .to_string();
    let template_tasks: Vec<TemplateTask> = tasks.iter().map(TemplateTask::from).collect();

    let mut env = Environment::new();
    env.add_template("markdown", include_str!("../assets/markdown.md"))
        .context("failed to register markdown template")?;
    let tmpl = env
        .get_template("markdown")
        .context("failed to load markdown template")?;

    let rendered = tmpl
        .render(context! {
            date_headline => date_headline,
            summary => summary,
            tasks => template_tasks,
        })
        .context("failed to render markdown template")?;

    Ok(to_crlf(&rendered))
}

pub fn export_markdown(output: &Path, tasks: &[Task]) -> Result<()> {
    info!("Dump {} tasks to file {}.", tasks.len(), output.display());
    let md_text = create_markdown(tasks)?;
    fs::write(output, md_text)
        .with_context(|| format!("failed to write markdown file: {}", output.display()))?;
    Ok(())
}
