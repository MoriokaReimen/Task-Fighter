use crate::driver::{Priority, Task, TaskStatus};
use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::{ImageFormat, open};
use jiff::Zoned;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html};
use std::fmt::Write as _;
use std::io::Cursor;
use std::io::Write as _;
use std::path::Path;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use tempfile::Builder;
use tracing::{error, info};

fn md_to_html(markdown_input: &str) -> String {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"]; // お好みのテーマを選択

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

    // 3. イベントストリームを処理するバッファ
    let mut new_events = Vec::new();
    let mut in_code_block = false;
    let mut current_lang = String::new();
    let mut code_accumulator = String::new();

    for event in parser {
        match event {
            // コードブロックの開始を検知
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                current_lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                    pulldown_cmark::CodeBlockKind::Indented => String::new(),
                };
                code_accumulator.clear();
            }
            // コードブロックの終了を検知：ここでハイライト済みのHTMLを生成
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;

                // 言語の判定（未指定の場合はプレーンテキスト）
                let syntax = ps
                    .find_syntax_by_token(&current_lang)
                    .unwrap_or_else(|| ps.find_syntax_plain_text());

                // syntect を使用してHTML文字列へ変換
                let highlighted_html =
                    highlighted_html_for_string(&code_accumulator, &ps, syntax, theme)
                        .unwrap_or_else(|_| {
                            format!("<pre><code>{}</code></pre>", code_accumulator)
                        });

                // 生成された生HTMLを Event::Html としてストリームに挿入
                new_events.push(Event::Html(highlighted_html.into()));
            }
            // コードブロックの内部テキストを蓄積
            Event::Text(text) if in_code_block => {
                code_accumulator.push_str(&text);
            }
            // それ以外の通常のイベントはそのままスルー
            _ => {
                if !in_code_block {
                    new_events.push(event);
                }
            }
        }
    }

    // 4. カスタマイズしたイベントストリームをHTML文字列にレンダリング
    let mut html_output = String::new();
    html::push_html(&mut html_output, new_events.into_iter());

    html_output
}

pub fn create_mail_html(tasks: &[Task]) -> String {
    let mut html = String::new();

    // タスクの進捗ステータスを集計
    let total_tasks = tasks.len();
    let completed = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Complete)
        .count();
    let in_progress = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::WorkInProgress)
        .count();
    let pending = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Pending)
        .count();
    let canceled = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Canceled)
        .count();

    let date_headline = Zoned::now().date().strftime("%B %d, %Y").to_string();

    // ベースHTML構造とスタイルの定義
    html.push_str(
"<html>\r\n<head>\r\n<meta charset=\"utf-8\">\r\n</head>\r\n\
<body style=\"font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; background-color: #f4f6f8; color: #333333; margin: 0; padding: 20px;\">\r\n\
<div style=\"max-width: 650px; margin: 0 auto; background: #ffffff; padding: 24px; border-radius: 8px; box-shadow: 0 4px 10px rgba(0,0,0,0.05);\">\r\n"
    );

    let _ = write!(html, "<h1>Task Status Report</h1>\r\n");
    let image_path = Path::new("./.plot.png");
    match open(image_path) {
        Ok(image) => {
            // 1. メモリ上に書き出すためのバッファを用意
            let mut buffer = Cursor::new(Vec::new());

            // 2. 画像データをPNGフォーマットとしてバッファに書き込む
            image.write_to(&mut buffer, ImageFormat::Png);

            // 3. バッファからPNG形式のバイト列を取り出してBase64エンコード
            let png_bytes = buffer.into_inner();
            let base64_image = STANDARD.encode(png_bytes);

            let img_src = format!("data:image/png;base64,{}", base64_image);
            let _ = write!(
                html,
                "<div style=\"text-align: center; margin-bottom: 16px;\">\r\n\
            <img src=\"{}\" alt=\"Report Header\" style=\"max-width: 100%; height: auto; border-radius: 4px;\">\r\n\
            </div>\r\n\r\n",
                img_src
            );
        }
        Err(e) => {
            error!("Failed to read file {}", e);
        }
    }

    // --- メインタイトルとサマリーカード ---
    let _ = write!(
        html,
        "<p style=\"color: #666666; font-size: 14px; margin-top: -8px;\">Generated on {}</p>\r\n\r\n\
<div style=\"background: #f8f9fa; border: 1px solid #e1e4e6; border-radius: 6px; padding: 16px; margin-bottom: 24px;\">\r\n\
<h3 style=\"margin: 0 0 12px 0; font-size: 16px; color: #444444;\">📊 Summary</h3>\r\n\
<table style=\"width: 100%; font-size: 14px; border-collapse: collapse;\">\r\n\
<tr><td><strong>Total Tasks:</strong> {}</td></tr>\r\n\
<tr><td><strong>Completed:</strong> {} ✅</td><td><strong>In Progress:</strong> {} 🏃</td></tr>\r\n\
<tr><td><strong>Pending:</strong> {} ⏳</td><td><strong>Canceled:</strong> {} 🚫</td></tr>\r\n\
</table>\r\n\
</div>\r\n\r\n",
        date_headline, total_tasks, completed, in_progress, pending, canceled,
    );

    // --- 各タスクのレンダリング ---
    for task in tasks {
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

        let start_date_str = task.start_date.strftime("%Y/%m/%d").to_string();
        let due_date_str = task.due_date.strftime("%Y/%m/%d").to_string();

        // 個別タスクのカードデザイン
        let _ = write!(
            html,
            "<div style=\"border: 1px solid #e1e4e6; border-radius: 6px; padding: 18px; margin-bottom: 16px; background-color: #ffffff;\">\r\n\
<div style=\"display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 12px;\">\r\n\
<h2 style=\"font-size: 18px; margin: 0; color: #111111;\"><span style=\"color: #888888; font-size: 14px; font-weight: normal; margin-right: 6px;\">#{}</span>{}</h2>\r\n\
</div>\r\n\
<table style=\"width: 100%; font-size: 14px; margin-bottom: 12px; border-collapse: collapse;\">\r\n\
<tr><td style=\"padding: 4px 0; color: #666666; width: 100px;\">Project</td><td style=\"padding: 4px 0;\"><strong>{}</strong></td></tr>\r\n\
<tr><td style=\"padding: 4px 0; color: #666666;\">Priority</td><td style=\"padding: 4px 0; color: {}; font-weight: bold;\">{}</td></tr>\r\n\
<tr><td style=\"padding: 4px 0; color: #666666;\">Status</td><td style=\"padding: 4px 0; color: {}; font-weight: bold;\">{}</td></tr>\r\n\
<tr><td style=\"padding: 4px 0; color: #666666;\">Timeline</td><td style=\"padding: 4px 0; font-family: monospace;\">{} ~ {}</td></tr>\r\n\
<tr><td style=\"padding: 4px 0; color: #666666;\">Time Spent</td><td style=\"padding: 4px 0;\">{} hrs</td></tr>\r\n\
<tr>\r\n\
<td style=\"padding: 4px 0; color: #666666;\">Progress</td>\r\n\
<td style=\"padding: 4px 0; vertical-align: middle;\">\r\n\
<progress value=\"{}\" max=\"100\" style=\"width: 120px; height: 12px; margin-right: 8px; vertical-align: middle;\"></progress>\r\n\
<span style=\"font-weight: bold; vertical-align: middle;\">{}%</span>\r\n\
</td>\r\n\
</tr>\r\n\
</table>\r\n\
<div style=\"background: #f8f9fa; border-left: 3px solid #cbd5e1; padding: 10px 14px; font-size: 14px; color: #4a5568;\">", // 💡 white-space: pre-wrap; を削除
            task.id,
            task.title,
            task.project,
            priority_color,
            priority_text,
            status_color,
            status_text,
            start_date_str,
            due_date_str,
            task.time_spent,
            task.progress,
            task.progress
        );

        // 詳細の流し込み
        if task.detail.trim().is_empty() {
            html.push_str("<span style=\"color: #a0aec0; font-style: italic;\">No additional details provided.</span>");
        } else {
            html.push_str(&md_to_html(&task.detail));
        }

        html.push_str("</div>\r\n</div>\r\n\r\n");
    }

    // フッターの閉鎖
    html.push_str("</div>\r\n</body>\r\n</html>");
    html.replace("\r\n", "\n").replace("\n", "\r\n")
}

pub fn launch_system_mailer(tasks: &[Task]) -> Result<()> {
    let html_text = create_mail_html(tasks);
    let raw_subject = Zoned::now()
        .date()
        .strftime("%Y/%m/%d Task Status Report")
        .to_string();

    let suffix = ".eml";

    let mut temp_file = Builder::new()
        .suffix(suffix)
        .tempfile_in(".")
        .context("Failed to allocate transient local storage space for email payload context")?;

    let mut eml_content = String::new();
    let _ = write!(eml_content, "Subject: {}\r\n", raw_subject);
    eml_content.push_str("X-Unsent: 1\r\n");
    eml_content.push_str("MIME-Version: 1.0\r\n");
    // ✨ 変更箇所: Content-Type を text/html に指定します
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::{Priority, Task, TaskStatus};
    use jiff::Zoned;

    // ヘルパー関数: テスト用のダミータスク群を生成
    fn create_mock_tasks() -> Vec<Task> {
        vec![
            Task {
                id: 1,
                active: true,
                title: "Implement DuckDB storage".to_string(),
                project: "Task-Fighter".to_string(),
                priority: Priority::High,
                status: TaskStatus::WorkInProgress,
                start_date: Zoned::now().into(),
                due_date: Zoned::now().into(),
                progress: 70.0,
                time_spent: 4.5,
                detail: "Need to replace raw file logic with duckdb embedded backend.".to_string(),
            },
            Task {
                id: 2,
                active: true,
                title: "Refactor Credits Widget".to_string(),
                project: "Task-Fighter".to_string(),
                priority: Priority::Low,
                status: TaskStatus::Complete,
                start_date: Zoned::now().into(),
                due_date: Zoned::now().into(),
                progress: 100.0,
                time_spent: 2.0,
                detail: "".to_string(), // 空の詳細文の挙動確認用
            },
        ]
    }

    #[test]
    fn test_create_mail_html_summary_aggregation() {
        let tasks = create_mock_tasks();
        let html = create_mail_html(&tasks);

        // 1. サマリーの集計数がHTML文字列に正しく反映されているか検証
        assert!(html.contains("Total Tasks:</strong> 2"));
        assert!(html.contains("Completed:</strong> 1 ✅"));
        assert!(html.contains("In Progress:</strong> 1 🏃"));
        assert!(html.contains("Pending:</strong> 0 ⏳"));
    }

    #[test]
    fn test_create_mail_html_content_rendering() {
        let tasks = create_mock_tasks();
        let html = create_mail_html(&tasks);

        // 2. 個別のタスクデータがHTML構造内に埋め込まれているか検証
        assert!(html.contains("#1</span>Implement DuckDB storage"));
        assert!(html.contains("🔴 High"));
        assert!(html.contains("Work In Progress 🏃"));
        assert!(html.contains("70%"));
        assert!(html.contains("Need to replace raw file logic"));

        // 3. 詳細が空のタスクに対するフォールバック表示の検証
        assert!(html.contains("No additional details provided."));
    }

    #[test]
    fn test_create_mail_html_empty_state() {
        // 4. エッジケース: タスクがゼロ件のときの動作検証
        let empty_tasks: Vec<Task> = vec![];
        let html = create_mail_html(&empty_tasks);

        assert!(html.contains("Total Tasks:</strong> 0"));
        assert!(html.contains("<html>"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn test_create_mail_html_crlf_format() {
        let tasks = create_mock_tasks();
        let html = create_mail_html(&tasks);

        // 1. 主要なHTML構造が問題なく含まれているか確認
        assert!(html.contains("</html>"));

        // 2. 【修正】単独の \n （直前に \r がない \n）が文字列内に1箇所も存在しないか検証
        let bytes = html.as_bytes();
        let mut has_isolated_lf = false;

        for i in 0..bytes.len() {
            if bytes[i] == b'\n' {
                // \n の直前(i-1) が \r でなければ、それは不正な単独の LF
                if i == 0 || bytes[i - 1] != b'\r' {
                    has_isolated_lf = true;
                    break;
                }
            }
        }

        assert!(
            !has_isolated_lf,
            "HTML output contains isolated LF (\\n) instead of CRLF (\\r\\n)."
        );
    }
}
