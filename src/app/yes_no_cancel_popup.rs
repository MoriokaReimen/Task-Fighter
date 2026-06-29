use crate::fl;
use eframe::egui;

/// ポップアップの状態を管理する構造体
pub struct YesNoCancelPopup {
    title: String,
    message: String,
    is_open: bool,
    // IDをインスタンスごとに一意にするためのフィールドを追加
    id: egui::Id,
}

/// ユーザーがどちらを選択したかの戻り値
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PopupResult {
    Yes,
    No,
    Cancel,
    None,
}

impl YesNoCancelPopup {
    /// 新しいポップアップの作成（一意の識別子を渡す）
    pub fn new(id_source: impl std::hash::Hash) -> Self {
        Self {
            title: String::new(),
            message: String::new(),
            is_open: false,
            id: egui::Id::new(id_source),
        }
    }

    /// ポップアップを開く
    pub fn open(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.is_open = true;
        // impl Into<String> を使うことで、&str でも String でも柔軟に受け取れる（不要なアロケーションを防ぐ）
        self.title = title.into();
        self.message = message.into();
    }

    /// ポップアップを描画する（ウィジェット関数）
    pub fn show(&mut self, ctx: &egui::Context) -> PopupResult {
        if !self.is_open {
            return PopupResult::None;
        }

        let mut result = PopupResult::None;

        // 1. egui::Modal を一意のIDで生成
        let modal = egui::Modal::new(self.id).show(ctx, |ui| {
            // タイトルをモーダルのヘッダーとして表示
            ui.heading(&self.title);
            ui.add_space(4.0);

            ui.label(&self.message);
            ui.add_space(12.0);

            // 右下にボタンを配置（Sidesの不要なクロージャを省略してシンプルに）
            egui::Sides::new().show(
                ui,
                |_ui| {}, // 左側は空
                |ui| {
                    ui.horizontal(|ui| {
                        // 2. ボタンクリック時に結果を格納し、直接モーダルを閉じるフラグを立てる
                        if ui.button(fl!("cancel")).clicked() {
                            result = PopupResult::Cancel;
                            self.is_open = false;
                        }
                        if ui.button(fl!("no")).clicked() {
                            result = PopupResult::No;
                            self.is_open = false;
                        }
                        if ui.button(fl!("yes")).clicked() {
                            result = PopupResult::Yes;
                            self.is_open = false;
                        }
                    });
                },
            );
        });

        // 3. モーダルの外側（暗背景）がクリックされた、またはエスケープキーが押された場合のハンドリング
        if modal.should_close() {
            self.is_open = false;
        }

        result
    }
}
