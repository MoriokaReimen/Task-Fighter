use anyhow::{Context, Result};
use domain::{Task, TaskPriority, TaskStatus};
use jiff::Zoned;
use minijinja::{Environment, context};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html};
use serde::Serialize;
use std::fmt::Write as _;
use std::fs::OpenOptions;
use std::io::Write as _;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

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
                TaskPriority::High => ("🔴 High", "#dc3545"),
                TaskPriority::Medium => ("🟡 Medium", "#ffc107"),
                TaskPriority::Low => ("🔵 Low", "#20c997"),
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

    let mut temp_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./task_report.eml")
        .context("Failed to task_report.eml")?;

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

    drop(temp_file);

    open::that("./task_report.eml")
        .context("Failed invoking native desktop standard protocol handlers")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{Task, TaskPriority, TaskStatus};
    use jiff::civil::Date;
    use std::fs;

    // テスト用のダミータスクを作成するヘルパー
    fn create_test_task(id: i32, status: TaskStatus, priority: TaskPriority, detail: &str) -> Task {
        Task {
            id,
            active: true,
            status,
            project: "TestProject".to_string(),
            title: format!("Task {}", id),
            detail: detail.to_string(),
            start_date: Date::new(2026, 7, 1).unwrap(),
            due_date: Date::new(2026, 7, 31).unwrap(),
            priority,
            progress: 45.0,
            time_spent: 8.5,
            entry_date: Date::new(2026, 7, 1).unwrap(),
            end_date: None,
        }
    }

    #[test]
    fn test_md_to_html_conversion() {
        // 1. Markdown 変換（コードブロック以外）の検証
        let markdown = "# Hello\nThis is **bold** text.";
        let html = md_to_html(markdown);

        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<strong>bold</strong>"));

        // 2. コードブロックとシンタックスハイライトの検証
        let markdown_code = "```rust\nfn main() {}\n```";
        let html_code = md_to_html(markdown_code);

        // syntect によるハイライト結果（preやspanタグ）が含まれているか確認
        assert!(html_code.contains("<pre"));
        assert!(html_code.contains("main"));
    }

    #[test]
    fn test_create_mail_html_summary_and_fallback() {
        // さまざまなステータスのタスクを準備
        let tasks = vec![
            create_test_task(
                1,
                TaskStatus::Complete,
                TaskPriority::High,
                "Done everything",
            ),
            create_test_task(
                2,
                TaskStatus::WorkInProgress,
                TaskPriority::Medium,
                "Working hard",
            ),
            create_test_task(3, TaskStatus::Pending, TaskPriority::Low, ""), // 詳細空っぽ
            create_test_task(4, TaskStatus::Canceled, TaskPriority::Low, "Dropped"),
        ];

        let image_data_mock = "data:image/png;base64,ABC...";
        let html_result = create_mail_html(&tasks, image_data_mock);

        // 改行コードがすべて CRLF (\r\n) に統一されているか確認
        assert!(html_result.contains("\r\n"));
        assert!(!html_result.contains("\n\n")); // 連続する単一の \n が残っていないか

        // 詳細が空の場合のフォールバック表示の検証
        assert!(html_result.contains("No additional details provided."));

        // 優先度やステータスのテキスト/カラーマッピングが埋め込まれているか検証
        assert!(html_result.contains("🔴 High"));
        assert!(html_result.contains("Complete ✅"));
        assert!(html_result.contains("#28a745")); // Complete のカラーコード
    }

    #[test]
    #[ignore] // CI環境でのメーラー誤起動を防ぐため、通常テストからは除外（cargo test -- --ignored で実行可能）
    fn test_launch_system_mailer_output() {
        let tasks = vec![create_test_task(
            1,
            TaskStatus::WorkInProgress,
            TaskPriority::High,
            "Critical implementation",
        )];
        let image_data_mock = "data:image/png;base64,XYZ...";
        let eml_path = "./task_report.eml";

        // 既存の古いテストファイルを削除しておく
        if fs::metadata(eml_path).is_ok() {
            let _ = fs::remove_file(eml_path);
        }

        // 実行（環境によって open::that がエラーを返す可能性があるため、Resultの成否のみ、あるいはファイル生成を主目的とする）
        let result = launch_system_mailer(&tasks, image_data_mock);

        // ファイルが正しく生成されたか検証
        assert!(fs::metadata(eml_path).is_ok(), "eml file should be created");

        // 生成された EML ファイルのヘッダー内容を検証
        let eml_content = fs::read_to_string(eml_path).unwrap();
        assert!(eml_content.contains("Subject:"));
        assert!(eml_content.contains("X-Unsent: 1"));
        assert!(eml_content.contains("Content-Type: text/html; charset=utf-8"));
        assert!(eml_content.contains("Critical implementation")); // 本文の内容

        // テスト環境のクリーンアップ（生成したファイルを削除）
        let _ = fs::remove_file(eml_path);

        // open::that がデスクトップ環境のないCIなどでコケても、ファイル生成自体が成功していればOKとする場合はアサーションを調整
        if let Err(ref e) = result {
            println!(
                "Note: mailer invoked successfully but open::that failed (expected in headless env): {}",
                e
            );
        }
    }
}
