use crate::driver::{Priority, Task, TaskStatus};
use anyhow::{Context, Result};
use jiff::Zoned;
use std::fmt::Write as _;
use std::io::Write as _;
use tempfile::Builder;
use tracing::info;

pub fn create_mail_text(tasks: &[Task]) -> String {
    let mut contents = String::new();

    let date_headline = Zoned::now()
        .date()
        .strftime("%Y/%m/%d Task Status Report\n")
        .to_string();

    let _ = write!(
        contents,
        "{date_headline}\
         ===========================================================================\n\
         There are currently {} tasks.\n\n",
        tasks.len()
    );

    for task in tasks {
        let status_gfm = match task.status {
            TaskStatus::Pending => "Pending",
            TaskStatus::WorkInProgress => "Work In Progress",
            TaskStatus::Complete => "Complete",
        };

        let priority_str = match task.priority {
            Priority::High => "🔴 High",
            Priority::Medium => "🟡 Medium",
            Priority::Low => "🔵 Low",
        };

        let start_date_str = task.start_date.strftime("%Y/%m/%d").to_string();
        let due_date_str = task.due_date.strftime("%Y/%m/%d").to_string();

        let _ = write!(
            contents,
            "Task #{}. {}\n\
             ---------------------------------------------------------------------------\n\
             - Project: {}\n\
             - Priority: {}\n\
             - Status: {}\n\
             - Start Date: {}\n\
             - Due Date: {}\n\
             - Progress: {}%\n\
             - Time Spent: {} hrs\n\n\
             # Details\n",
            task.id,
            task.title,
            task.project,
            priority_str,
            status_gfm,
            start_date_str,
            due_date_str,
            task.progress,
            task.time_spent
        );

        for line in task.detail.lines() {
            let _ = writeln!(contents, "{}", line);
        }

        contents.push_str("\n\n");
    }

    contents
}

pub fn launch_system_mailer(tasks: &[Task]) -> Result<()> {
    let body_text = create_mail_text(tasks);
    let raw_subject = Zoned::now()
        .date()
        .strftime("%Y/%m/%d Task Status Report")
        .to_string();

    let mut temp_file = Builder::new()
        .suffix(".eml")
        .tempfile()
        .context("Failed to allocate transient local storage space for email payload context")?;

    // Windows (Outlook等) のために、改行コードをすべて \r\n (CRLF) に統一します
    let mut eml_content = String::new();
    let _ = write!(eml_content, "Subject: {}\r\n", raw_subject);
    eml_content.push_str("MIME-Version: 1.0\r\n");
    eml_content.push_str("Content-Type: text/plain; charset=utf-8\r\n");
    eml_content.push_str("Content-Transfer-Encoding: 8bit\r\n\r\n");

    // 本文の改行コードも一念のため CRLF に変換しておくとより安全です
    let body_crlf = body_text.replace("\r\n", "\n").replace("\n", "\r\n");
    eml_content.push_str(&body_crlf);

    temp_file.write_all(eml_content.as_bytes()).context(
        "Failed writing compiled structural email buffers down to local transient path descriptors",
    )?;

    temp_file.flush().context(
        "Failed flushing operating system write streams buffers onto block devices segments",
    )?;

    // 先にファイルを永続化（自動削除を解除）し、ファイルハンドルを確実に閉じます
    // Windowsではファイルが開いたままだと、他のプロセス（メーラー）がファイルを開けないことがあります
    let (file, path) = temp_file.keep().context(
        "Failed safeguarding local file persistence integrity boundary configurations locks",
    )?;

    // 明示的にドロップしてファイルロックを解除
    drop(file);

    info!(
        "Synchronized continuous email file sequence artifact: {:?}",
        path
    );

    // 完全にフリーになったファイルパスをシステムに渡す
    open::that(&path).context(
        "Failed invoking native desktop standard protocol handlers to stream target file content",
    )?;

    info!("Dispatched active call stack processing to localized system default client interface.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::Date;

    fn create_mock_task(id: i32, title: &str, status: TaskStatus, priority: Priority) -> Task {
        Task {
            id,
            active: true,
            title: title.to_string(),
            project: "Test Project".to_string(),
            status,
            priority,
            start_date: Date::new(2026, 6, 22)
                .expect("Failed evaluating mock boundary baseline start calendar date state"),
            due_date: Date::new(2026, 6, 25)
                .expect("Failed evaluating mock boundary baseline closure calendar date state"),
            progress: 50.0,
            time_spent: 4.5,
            detail: "Line 1\nLine 2".to_string(),
        }
    }

    #[test]
    fn test_create_mail_text_empty() {
        let tasks: Vec<Task> = vec![];
        let result = create_mail_text(&tasks);
        assert!(result.contains("There are currently 0 tasks."));
    }

    #[test]
    fn test_create_mail_text_with_tasks() {
        let tasks = vec![
            create_mock_task(
                1,
                "Fix Critical Bug",
                TaskStatus::WorkInProgress,
                Priority::High,
            ),
            create_mock_task(
                2,
                "Update Documentation",
                TaskStatus::Complete,
                Priority::Low,
            ),
        ];

        let result = create_mail_text(&tasks);

        assert!(result.contains("There are currently 2 tasks."));

        assert!(result.contains("Task #1. Fix Critical Bug"));
        assert!(result.contains("- Priority: 🔴 High"));
        assert!(result.contains("- Status: Work In Progress"));
        assert!(result.contains("- Progress: 50%"));
        assert!(result.contains("- Time Spent: 4.5 hrs"));

        assert!(result.contains("Task #2. Update Documentation"));
        assert!(result.contains("- Priority: 🔵 Low"));
        assert!(result.contains("- Status: Complete"));

        assert!(result.contains("Line 1\nLine 2"));
    }
}
