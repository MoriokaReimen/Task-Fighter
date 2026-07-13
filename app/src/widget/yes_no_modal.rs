use egui;

/// ポップアップの状態を管理する構造体
pub struct YesNoModal {
    title: String,
    message: String,
    is_open: bool,
    // IDをインスタンスごとに一意にするためのフィールドを追加
    id: egui::Id,
}

/// ユーザーがどちらを選択したかの戻り値
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ModalResult {
    Yes,
    No,
    None,
}

impl YesNoModal {
    /// 新しいポップアップの作成（一意の識別子を渡す）
    pub fn new(id_source: impl std::hash::Hash + std::fmt::Debug) -> Self {
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
        self.title = title.into();
        self.message = message.into();
    }

    /// ポップアップを描画する（ウィジェット関数）
    pub fn show(&mut self, ctx: &egui::Context) -> ModalResult {
        if !self.is_open {
            return ModalResult::None;
        }

        let mut result = ModalResult::None;

        // 1. egui::Modal を一意のIDで生成
        let _modal = egui::Modal::new(self.id).show(ctx, |ui| {
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
                        if ui.button(fl!("no")).clicked() {
                            result = ModalResult::No;
                            self.is_open = false;
                        }
                        if ui.button(fl!("yes")).clicked() {
                            result = ModalResult::Yes;
                            self.is_open = false;
                        }
                    });
                },
            );
        });

        result
    }
}
