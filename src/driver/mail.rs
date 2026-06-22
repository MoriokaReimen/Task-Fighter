use crate::driver::{Priority, Task, TaskStatus}; // Priority もインポート
use anyhow::{Context, Result};
use jiff::Zoned;
use std::io::Write;
use tempfile::Builder;
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

pub fn launch_system_mailer_via_eml(tasks: &Vec<Task>) -> Result<()> {
    // 1. メール本文と件名の生成
    let body_text = create_mail_text(tasks);
    let raw_subject = Zoned::now()
        .date()
        .strftime("%Y/%m/%d Task Status Report")
        .to_string();

    // 2. 一時ファイル (.eml) の作成
    // NamedTempFile を使うことで、ファイルパスを維持しつつ、
    // 変数 `_temp_file` がスコープを外れたときに自動削除されるようにします。
    let mut temp_file = Builder::new()
        .suffix(".eml")
        .tempfile()
        .context("一時ファイルの作成に失敗しました")?;

    // 3. EMLフォーマット（MIME）に従ってデータを書き込み
    // Windowsの標準メーラーなどで文字化けを防ぐため、UTF-8であることを明示します。
    let mut eml_content = String::new();
    eml_content.push_str(&format!("Subject: {}\n", raw_subject));
    eml_content.push_str("MIME-Version: 1.0\n");
    eml_content.push_str("Content-Type: text/plain; charset=utf-8\n");
    eml_content.push_str("Content-Transfer-Encoding: 8bit\n");
    eml_content.push('\n'); // ヘッダーと本文を区切る空行
    eml_content.push_str(&body_text);

    temp_file
        .write_all(eml_content.as_bytes())
        .context("一時ファイルへの書き込みに失敗しました")?;

    // データをディスクに確実にフラッシュする
    temp_file
        .flush()
        .context("ファイルのフラッシュに失敗しました")?;

    // 4. 一時ファイルのパスを取得
    let file_path = temp_file.path();

    info!("EMLファイルを生成しました: {:?}", file_path);

    // 5. OS標準のアプリケーション（メーラー）で開く
    // 注意: 多くのOSでは、開いたメーラーがファイルを読み込むまで
    // しばらく時間がかかるため、open::that が完了した直後にプログラムを終了すると、
    // 一時ファイルが先に削除されてしまいメーラー側で「ファイルが見つかりません」となることがあります。
    // そのため、一時ファイルの自動削除を無効化して永続化するか、少し待機処理を入れます。

    // ここでは確実にメーラーにファイルを渡すため、一時ファイルを自動削除させずに残す（永続化）
    // もしくは、特定の「一時保存ディレクトリ」に普通のファイルとして出力する方法が安全です。
    let (_file, path) = temp_file
        .keep()
        .context("一時ファイルの永続化に失敗しました")?;

    open::that(&path).context("デフォルトのメーラーでEMLファイルを開けませんでした")?;

    info!("システムメーラーを起動しました。");
    Ok(())
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
    // 擬似的な Task 構造体の定義（実際の構造体のフィールドに合わせて適宜調整してください）
    // ※ jiff クレートの Zoned や CivilDate を想定しています
    use jiff::civil::Date;

    // テスト用のTaskインスタンスを生成するヘルパー関数
    fn create_mock_task(id: i32, title: &str, status: TaskStatus, priority: Priority) -> Task {
        Task {
            id,
            active: true,
            title: title.to_string(),
            project: "Test Project".to_string(),
            status: status,
            priority: priority,
            // jiff の日付型 (2026/06/22 を仮定)
            start_date: Date::new(2026, 6, 22).expect("REASON"),
            due_date: Date::new(2026, 6, 25).expect("REASON"),
            progress: 50.0,
            time_spent: 4.5,
            detail: "Line 1\nLine 2".to_string(),
        }
    }

    #[test]
    fn test_create_mail_text_empty() {
        let tasks: Vec<Task> = vec![];
        let result = create_mail_text(&tasks);

        // タスク数が0件の場合のヘッダーチェック
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

        // 1. 全体のタスク数カウントの検証
        assert!(result.contains("There are currently 2 tasks."));

        // 2. タスク#1 のステータス、優先度、詳細の変換検証
        assert!(result.contains("Task #1. Fix Critical Bug"));
        assert!(result.contains("- Priority: 🔴 High"));
        assert!(result.contains("- Status: Work In Progress"));
        assert!(result.contains("- Progress: 50%"));
        assert!(result.contains("- Time Spent: 4.5 hrs"));

        // 3. タスク#2 のステータス、優先度の変換検証
        assert!(result.contains("Task #2. Update Documentation"));
        assert!(result.contains("- Priority: 🔵 Low"));
        assert!(result.contains("- Status: Complete"));

        // 4. 複数行の Details が正しくパースされているか検証
        assert!(result.contains("Line 1\nLine 2"));
    }
}
