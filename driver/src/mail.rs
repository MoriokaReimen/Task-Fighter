use crate::{Priority, Task, TaskStatus};
use anyhow::{Context, Result};
use jiff::Zoned;
use minijinja::{Environment, context};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html};
use serde::Serialize;
use std::fmt::Write as _;
use std::io::Write as _;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use tempfile::Builder;
use tracing::info;

// 1. テンプレートに渡すためのシリアライズ可能なデータ構造を定義
#[derive(Serialize)]
struct TemplateSummary {
    total: usize,
    completed: usize,
    in_progress: usize,
    pending: usize,
    canceled: usize,
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
    detail_html: String, // 変換済みのHTMLをここに格納
}

fn md_to_html(markdown_input: &str) -> String {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    options.insert(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);
    options.insert(Options::ENABLE_OLD_FOOTNOTES);
    options.insert(Options::ENABLE_MATH);
    options.insert(Options::ENABLE_GFM);
    let parser = Parser::new_ext(markdown_input, options);

    let mut new_events = Vec::new();
    let mut in_code_block = false;
    let mut current_lang = String::new();
    let mut code_accumulator = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                current_lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                    pulldown_cmark::CodeBlockKind::Indented => String::new(),
                };
                code_accumulator.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                let syntax = ps
                    .find_syntax_by_token(&current_lang)
                    .unwrap_or_else(|| ps.find_syntax_plain_text());

                let highlighted_html =
                    highlighted_html_for_string(&code_accumulator, &ps, syntax, theme)
                        .unwrap_or_else(|_| {
                            format!("<pre><code>{}</code></pre>", code_accumulator)
                        });

                new_events.push(Event::Html(highlighted_html.into()));
            }
            Event::Text(text) if in_code_block => {
                code_accumulator.push_str(&text);
            }
            _ => {
                if !in_code_block {
                    new_events.push(event);
                }
            }
        }
    }

    let mut html_output = String::new();
    html::push_html(&mut html_output, new_events.into_iter());
    html_output
}

pub fn create_mail_html(tasks: &[Task], image_data: &str) -> String {
    // サマリーの集計
    let summary = TemplateSummary {
        total: tasks.len(),
        completed: tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Complete)
            .count(),
        in_progress: tasks
            .iter()
            .filter(|t| t.status == TaskStatus::WorkInProgress)
            .count(),
        pending: tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .count(),
        canceled: tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Canceled)
            .count(),
    };

    let date_headline = Zoned::now().date().strftime("%B %d, %Y").to_string();

    // テンプレート用のタスクデータ配列へ変換
    let template_tasks: Vec<TemplateTask> = tasks
        .iter()
        .map(|task| {
            let (status_text, status_color) = match task.status {
                TaskStatus::Pending => ("Pending ⏳", "#6c757d"),
                TaskStatus::WorkInProgress => ("Work In Progress 🏃", "#007bff"),
                TaskStatus::Complete => ("Complete ✅", "#28a745"),
                TaskStatus::Canceled => ("Canceled 🚫", "#ff5733"),
            };

            let (priority_text, priority_color) = match task.priority {
                Priority::High => ("🔴 High", "#dc3545"),
                Priority::Medium => ("🟡 Medium", "#ffc107"),
                Priority::Low => ("🔵 Low", "#20c997"),
            };

            let detail_html = if task.detail.trim().is_empty() {
                "<span style=\"color: #a0aec0; font-style: italic;\">No additional details provided.</span>".to_string()
            } else {
                md_to_html(&task.detail)
            };

            TemplateTask {
                id: task.id,
                title: task.title.clone(),
                project: task.project.clone(),
                priority_text,
                priority_color,
                status_text,
                status_color,
                start_date: task.start_date.strftime("%Y/%m/%d").to_string(),
                due_date: task.due_date.strftime("%Y/%m/%d").to_string(),
                time_spent: task.time_spent as f64,
                progress: task.progress as f64,
                detail_html,
            }
        })
        .collect();

    // minijinja の環境セットアップ（文字列インクルードでテンプレートを登録）
    let mut env = Environment::new();
    env.add_template("mail", include_str!("../assets/mail.html"))
        .expect("Failed to compile template");

    let tmpl = env.get_template("mail").expect("Failed to get template");

    // レンダリングを実行
    let rendered = tmpl
        .render(context! {
            image_data => image_data,
            date_headline => date_headline,
            summary => summary,
            tasks => template_tasks,
        })
        .expect("Failed to render template");

    // テスト要件を満たすため、すべての改行を確実に CRLF 形式に統一
    rendered.replace("\r\n", "\n").replace("\n", "\r\n")
}

pub fn launch_system_mailer(tasks: &[Task], image_data: &str) -> Result<()> {
    let html_text = create_mail_html(tasks, image_data);
    let raw_subject = Zoned::now()
        .date()
        .strftime("%Y/%m/%d Task Status Report")
        .to_string();

    let suffix = ".eml";

    let mut temp_file = Builder::new()
        .suffix(suffix)
        .tempfile()
        .context("Failed to allocate transient local storage space for email payload context")?;

    let mut eml_content = String::new();
    let _ = write!(eml_content, "Subject: {}\r\n", raw_subject);
    eml_content.push_str("X-Unsent: 1\r\n");
    eml_content.push_str("MIME-Version: 1.0\r\n");
    eml_content.push_str("Content-Type: text/html; charset=utf-8\r\n");
    eml_content.push_str("Content-Transfer-Encoding: 8bit\r\n\r\n");

    let body_crlf = html_text.replace("\r\n", "\n").replace("\n", "\r\n");
    eml_content.push_str(&body_crlf);

    temp_file
        .write_all(eml_content.as_bytes())
        .context("Failed writing compiled structural email buffers")?;
    temp_file
        .flush()
        .context("Failed flushing operating system write streams buffers")?;

    let (file, path) = temp_file
        .keep()
        .context("Failed safeguarding local file persistence integrity")?;
    drop(file);

    info!(
        "Synchronized continuous email file sequence artifact: {:?}",
        path
    );
    open::that(&path).context("Failed invoking native desktop standard protocol handlers")?;

    Ok(())
}

// 既存の #[cfg(test)] mod tests { ... } は変更なしでそのまま動作します
