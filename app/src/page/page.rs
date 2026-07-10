use crate::work::Work;

pub trait Page {
    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) -> Pages;
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Pages {
    #[default]
    Main, // メイン（デフォルト）画面
    EditTask,   // 編集画面
    CreateTask, // 作成画面
    Graph,      // グラフ画面
    Timer,      // タイマー（時間計測）画面
}
