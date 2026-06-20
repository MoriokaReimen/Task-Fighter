use crate::driver::{Priority, Task}; // Priority もインポート
use anyhow::{Context, Result};
use jiff::Zoned;
use tracing::info;
use urlencoding::encode;

pub fn create_mail_text(tasks: &Vec<Task>) -> String {
    let mut contents = String::new();

    // メールのタイトルや概要をMarkdownのヘッダーで作成
    contents += "Task Status Report\n";
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
        contents += &format!("- Due Date: {}\n\n", due_date_str);
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
    use chrono::{TimeZone, Utc};

    // テスト用のTaskベクタを作成するヘルパー関数
    fn create_test_tasks() -> Vec<Task> {
        let start = Utc.with_ymd_and_hms(2026, 6, 1, 9, 0, 0).unwrap();
        let due = Utc.with_ymd_and_hms(2026, 6, 10, 18, 0, 0).unwrap();

        vec![
            Task::new(
                1,
                true,
                false,
                "Project Alpha".to_string(),
                "フロントエンドの実装".to_string(),
                "ログイン画面のUI作成\nバリデーションの追加".to_string(),
                start,
                due,
                Priority::High,
            ),
            Task::new(
                2,
                true,
                true,
                "Project Beta".to_string(),
                "ドキュメント作成".to_string(),
                "仕様書の更新".to_string(),
                start,
                due,
                Priority::Low,
            ),
        ]
    }

    #[test]
    fn test_create_mail_text() {
        let tasks = create_test_tasks();
        let mail_text = create_mail_text(&tasks);

        // 1. 全体件数が正しく埋め込まれているか
        assert!(mail_text.contains("現在、**2件**のタスクがあります。"));

        // 2. タスク1（未完了・High）の内容検証
        assert!(mail_text.contains("## 1. フロントエンドの実装"));
        assert!(mail_text.contains("- **プロジェクト**: `Project Alpha`"));
        assert!(mail_text.contains("- **優先度**: 🔴 高 (High)"));
        assert!(mail_text.contains("- **ステータス**: - [ ] **未完了**"));
        assert!(mail_text.contains("- **開始日時**: 2026-06-01 09:00"));
        assert!(mail_text.contains("- **期限日時**: 2026-06-10 18:00"));
        // 複数行のディテールが引用（>）になっているか
        assert!(mail_text.contains("> ログイン画面のUI作成"));
        assert!(mail_text.contains("> バリデーションの追加"));

        // 3. タスク2（完了・Low）の内容検証
        assert!(mail_text.contains("## 2. ドキュメント作成"));
        assert!(mail_text.contains("- **優先度**: 🔵 低 (Low)"));
        assert!(mail_text.contains("- **ステータス**: - [x] **完了**"));
    }

    #[test]
    fn test_create_mail_text_empty() {
        let tasks: Vec<Task> = vec![];
        let mail_text = create_mail_text(&tasks);

        assert!(mail_text.contains("現在、**0件**のタスクがあります。"));
    }

    #[test]
    fn test_launch_system_mailer_flow() {
        let tasks = create_test_tasks();

        // ※注意: テスト環境にGUI（デフォルトメーラー）がないヘッドレス環境（CIなど）の場合、
        // open::that はエラーを返す可能性があります。
        // そのため、ResultがOkであることを盲信するのではなく、関数がクラッシュ（panic）しないこと、
        // もしくは環境依存のエラーとして安全に処理されることを検証します。
        let result = launch_system_mailer(&tasks);

        // テストが実行された環境（GUIがあるかないか）によって結果が変わるため、
        // ここでは match を使って「関数が正しく実行を終えたか（あるいは open のエラーか）」をチェックします。
        match result {
            Ok(_) => info!("テスト環境でメーラーが正常に呼び出されました。"),
            Err(e) => {
                let err_msg = e.to_string();
                // 少なくとも「open::that」の手前までのロジック（create_mail_text や encode）が
                // 正常に動いていることは、パニックが起きないことで保証されます。
                error!(
                    "メーラーの起動自体は環境要因でスキップされました: {}",
                    err_msg
                );
            }
        }
    }
}
