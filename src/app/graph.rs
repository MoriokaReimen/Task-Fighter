use crate::fl;
use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use std::sync::{Arc, Mutex};
use tracing::log::{error, info};

pub struct Graph {
    // グラフの描画領域（位置とサイズ）を記録する
    plot_rect: Option<egui::Rect>,
    // ファイル保存ダイアログを開くフラグ
    should_save: Arc<Mutex<bool>>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            plot_rect: None,
            should_save: Arc::new(Mutex::new(false)),
        }
    }

    /// 「グラフだけ」を画像として保存する要求を出すメソッド
    pub fn save_screenshot(&self) {
        if let Ok(mut guard) = self.should_save.lock() {
            *guard = true;
        }
    }
}

impl Graph {
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        // 1. グラフの描画
        let response = self.draw_plot(ui);

        // 2. 外部からのスクリーンショット要求があれば、eguiに撮影コマンドを送信
        if self.check_save_trigger() {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }

        // 3. 発行されたスクリーンショットイベントのキャッチと保存処理
        #[cfg(not(target_arch = "wasm32"))]
        self.handle_screenshot_event(ui.ctx());

        response
    }

    /// コントロールUI（保存ボタンなど）を表示するメソッド
    pub fn show_controls(&self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui.button(fl!("save-graph"));
        if response.clicked() {
            self.save_screenshot();
        }
        response
    }
}

// --- 内部補助関数（プライベートメソッド） ---
impl Graph {
    /// グラフの定義と描画を行い、描画領域を記録する
    fn draw_plot(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let my_plot = Plot::new("My Plot").legend(Legend::default());
        let graph: Vec<[f64; 2]> = vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0]];

        let inner = my_plot.show(ui, |plot_ui| {
            plot_ui.line(Line::new("curve", PlotPoints::from(graph)));
        });

        self.plot_rect = Some(inner.response.rect);
        inner.response
    }

    /// 保存要求フラグが立っているか確認し、立っていればリセットして true を返す
    fn check_save_trigger(&self) -> bool {
        let Ok(mut guard) = self.should_save.lock() else {
            return false;
        };
        if *guard {
            *guard = false;
            return true;
        }
        false
    }

    /// eguiのイベント配列からScreenshotイベントを検知して非同期保存する
    #[cfg(not(target_arch = "wasm32"))]
    fn handle_screenshot_event(&self, ctx: &egui::Context) {
        // 早期リターン：グラフの描画領域が未決定なら何もしない
        let Some(rect) = self.plot_rect else { return };

        // イベントからスクリーンショットデータを検索
        let screenshot = ctx.input(|i| {
            i.events.iter().find_map(|event| match event {
                egui::Event::Screenshot { image, .. } => Some(Arc::clone(image)),
                _ => None,
            })
        });

        // 早期リターン：今フレームでスクリーンショットイベントがなければ終了
        let Some(screenshot_img) = screenshot else {
            return;
        };

        // 条件が揃ったので保存処理を別スレッドで実行
        let pixels_per_point = ctx.pixels_per_point();
        std::thread::spawn(move || {
            let Some(mut path) = rfd::FileDialog::new().save_file() else {
                return;
            };
            path.set_extension("png");

            // グラフの領域だけを切り抜いて保存
            let plot_image = screenshot_img.region(&rect, Some(pixels_per_point));
            let result = image::save_buffer(
                &path,
                plot_image.as_raw(),
                plot_image.width() as u32,
                plot_image.height() as u32,
                image::ColorType::Rgba8,
            );

            match result {
                Ok(()) => info!("Image saved to {}", path.display()),
                Err(err) => error!("Failed to save image: {err}"),
            }
        });
    }
}
