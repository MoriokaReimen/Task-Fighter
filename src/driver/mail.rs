use crate::driver::{Priority, Task, TaskStatus}; // Priority もインポート
use anyhow::{Context, Result};
use jiff::Zoned;
use tracing::info;
use urlencoding::encode;

pub fn create_mail_text(tasks: &Vec<Task>) -> String {
    let mut contents = String::new();

    // メールのタイトルや概要をMarkdownのヘッダーで作成
    contents += &Zoned::now()
        .date()
        .strftime("%Y/%m/%d Task Status Report\n")
        .to_string();
    contents += "===========================================================================\n";
    contents += &format!("There are currently {} tasks.\n\n", tasks.len());

    for task in tasks {
        // Markdownのタスクリスト（- [ ] や - [x]）を活用
        let status_gfm = match task.status {
            TaskStatus::Pending => "Pending",
            TaskStatus::WorkInProgress => "Work In Progress",
            TaskStatus::Complete => "Complete",
        };

        // 優先度を分かりやすい文字列（絵文字付き）に変換
        let priority_str = match task.priority {
            Priority::High => "🔴 High",
            Priority::Medium => "🟡 Medium",
            Priority::Low => "🔵 Low",
        };

        // 日付を読みやすいフォーマット（YYYY-MM-DD HH:MM）に変換
        let start_date_str = task.start_date.strftime("%Y/%m/%d").to_string();
        let due_date_str = task.due_date.strftime("%Y/%m/%d").to_string();

        // ## 見出しでタスクタイトルを、引用枠（>）で詳細を表現
        contents += &format!("Task #{}. {}\n", task.id, task.title);
        contents += "---------------------------------------------------------------------------\n";
        contents += &format!("- Project: {}\n", task.project);
        contents += &format!("- Priority: {}\n", priority_str);
        contents += &format!("- Status: {}\n", status_gfm);
        contents += &format!("- Start Date: {}\n", start_date_str);
        contents += &format!("- Due Date: {}\n", due_date_str);
        contents += &format!("- Progress: {}%\n", task.progress);
        contents += &format!("- Time Spent: {} hrs\n\n", task.time_spent);
        contents += "# Details\n";

        for line in task.detail.lines() {
            contents += &format!("{}\n", line);
        }

        contents += "\n\n";
    }

    contents
}

/// 生成したMarkdownテキストを本文にセットして、OS標準のメーラーを起動する
pub fn launch_system_mailer(tasks: &Vec<Task>) -> Result<()> {
    // 1. メール本文のMarkdownを生成
    let body_text = create_mail_text(tasks);

    // 2. 件名と本文をURLエンコード（スペースや改行、# などをURLセーフにするため）
    //
    let raw_subject = Zoned::now()
        .date()
        .strftime("%Y/%m/%d Task Status Report")
        .to_string();
    let subject = encode(&raw_subject);
    let body = encode(&body_text);

    // 3. mailto URLを組み立て
    let mailto_url = format!("mailto:?subject={}&body={}", subject, body);

    // 4. OS標準のブラウザやメーラーでURLを開く
    open::that(&mailto_url).context("デフォルトのメーラーを起動できませんでした")?;

    info!("システムメーラーを起動しました。");
    info!("{}", mailto_url);
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    // テスト用に必要な型を定義（実際のcrate::driverの構造に合わせて調整してください）
    use crate::driver::{Priority, Task, TaskStatus};
    use jiff::civil::Date;

    // ヘルパー関数: テスト用のTaskインスタンスを生成する
    fn create_mock_task(id: i32, title: &str, priority: Priority, status: TaskStatus) -> Task {
        // jiff::Zoned または検証可能な日付型を生成
        // ここでは仮に Zoned::now() もしくは特定のCivil日付から変換していると想定
        let start_date = Date::new(2026, 6, 1).unwrap();
        let due_date = Date::new(2026, 6, 30).unwrap();

        Task {
            id,
            active: true,
            title: title.to_string(),
            project: "Project Alpha".to_string(),
            priority,
            status,
            start_date,
            due_date,
            progress: 50.0,
            time_spent: 4.5,
            detail: "This is a detail line 1.\nThis is a detail line 2.".to_string(),
        }
    }

    #[test]
    fn test_create_mail_text_empty() {
        let tasks: Vec<Task> = vec![];
        let result = create_mail_text(&tasks);

        // タスクが0件の場合のヘッダーと件数カウントの検証
        assert!(result.contains(
            "==========================================================================="
        ));
        assert!(result.contains("There are currently 0 tasks."));
    }

    #[test]
    fn test_create_mail_text_with_tasks() {
        let tasks = vec![
            create_mock_task(
                1,
                "Fix critical bug",
                Priority::High,
                TaskStatus::WorkInProgress,
            ),
            create_mock_task(
                2,
                "Write documentation",
                Priority::Low,
                TaskStatus::Complete,
            ),
        ];

        let result = create_mail_text(&tasks);

        // 1. 全体件数のチェック
        assert!(result.contains("There are currently 2 tasks."));

        // 2. タスク1（High / WorkInProgress）の出力チェック
        assert!(result.contains("Task #1. Fix critical bug"));
        assert!(result.contains("- Priority: 🔴 High"));
        assert!(result.contains("- Status: Work In Progress"));
        assert!(result.contains("- Start Date: 2026/06/01"));
        assert!(result.contains("- Due Date: 2026/06/30"));
        assert!(result.contains("- Progress: 50%"));
        assert!(result.contains("- Time Spent: 4.5 hrs"));

        // 詳細（複数行）の展開チェック
        assert!(result.contains("This is a detail line 1."));
        assert!(result.contains("This is a detail line 2."));

        // 3. タスク2（Low / Complete）の出力チェック
        assert!(result.contains("Task #2. Write documentation"));
        assert!(result.contains("- Priority: 🔵 Low"));
        assert!(result.contains("- Status: Complete"));
    }
}
