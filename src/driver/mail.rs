use crate::driver::{Priority, Task}; // Priority もインポート
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
        let status_gfm = if task.done {
            "☑ Complete"
        } else {
            "☐ Incomplete"
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
        contents += &format!("- Time Spent: {}%\n\n", task.time_spent);
        contents += "# Details\n";

        // 詳細が複数行になっても崩れないよう引用スタイルに
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
    use jiff::civil::date;

    // テスト用のダミータスクを生成するヘルパー関数
    fn create_test_task(id: i32, title: &str, priority: Priority, done: bool) -> Task {
        Task {
            id,
            active: true,
            done,
            project: "TestProject".to_string(),
            title: title.to_string(),
            detail: "LINE 1\nLINE 2".to_string(),
            // テスト結果が固定されるよう、固定の日付を設定
            start_date: date(2026, 6, 21),
            due_date: date(2026, 6, 30),
            priority,
            progress: 0.0,
            time_spent: 0.0,
        }
    }

    #[test]
    fn test_create_mail_text_empty() {
        let tasks: Vec<Task> = vec![];
        let result = create_mail_text(&tasks);

        // タスクが0件の場合のヘッダーチェック
        assert!(result.contains("There are currently 0 tasks."));
    }

    #[test]
    fn test_create_mail_text_with_tasks() {
        let tasks = vec![
            create_test_task(1, "未完了タスク", Priority::High, false),
            create_test_task(2, "完了タスク", Priority::Low, true),
        ];

        let result = create_mail_text(&tasks);

        // 1. 全体件数のチェック
        assert!(result.contains("There are currently 2 tasks."));

        // 2. 1件目（未完了・優先度高）の出力チェック
        assert!(result.contains("Task #1. 未完了タスク"));
        assert!(result.contains("- Priority: 🔴 High"));
        assert!(result.contains("- Status: ☐ Incomplete"));
        assert!(result.contains("- Start Date: 2026/06/21"));
        assert!(result.contains("- Due Date: 2026/06/30"));

        // 3. 2件目（完了・優先度低）の出力チェック
        assert!(result.contains("Task #2. 完了タスク"));
        assert!(result.contains("- Priority: 🔵 Low"));
        assert!(result.contains("- Status: ☑ Complete"));

        // 4. 詳細（複数行）が正しく展開されているかチェック
        assert!(result.contains("LINE 1"));
        assert!(result.contains("LINE 2"));
    }

    #[test]
    fn test_create_mail_text_priority_medium() {
        let tasks = vec![create_test_task(1, "中優先度", Priority::Medium, false)];
        let result = create_mail_text(&tasks);

        // 優先度 Medium の絵文字チェック
        assert!(result.contains("- Priority: 🟡 Medium"));
    }
}
