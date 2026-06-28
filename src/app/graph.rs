use crate::fl;
use eframe::egui;
use egui_plot::{Bar, BarChart, Legend, Line, Plot, PlotPoints, GridMark, GridInput};
use jiff::{ToSpan, Zoned};
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
    pub fn show(&mut self, ui: &mut egui::Ui, data: &Vec<(i32, i32, i32, i32)>) -> egui::Response {
        // 1. グラフの描画
        let response = self.draw_plot(ui, data);

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
}

// --- 内部補助関数（プライベートメソッド） ---
impl Graph {
    fn draw_plot(&mut self, ui: &mut egui::Ui, data: &Vec<(i32, i32, i32, i32)>) -> egui::Response {
        // 1. x_formatter の引数を GridMark に修正
        let x_formatter = move |mark: egui_plot::GridMark,
                                _range: &std::ops::RangeInclusive<f64>| {
            let index = mark.value.round() as i64;

            // ループ側で .rev() しているため、index 0 は「データの一番後ろ（過去）」になります。
            // 今日の日付から (データ数 - 1 - index) 日を引くことで、カレンダー通りの正しい日付になります。
            let start_date = Zoned::now().date();
            let days_to_subtract = (data.len() as i64 - 1) - index;

            if index >= 0 && index < data.len() as i64 {
                let date = start_date - days_to_subtract.days(); // 過去へ引き算
                date.strftime("%Y/%m/%d").to_string()
            } else {
                "".to_string()
            }
        };

        let my_plot = Plot::new("Task Plot")
            .legend(Legend::default())
            .clamp_grid(true)
            .x_grid_spacer(move |input: GridInput| {
                let mut ticks = Vec::new();
                let start = input.bounds.0.floor() as i64;
                let end = input.bounds.1.ceil() as i64;
                for i in start..=end {
                    // 2. egui_plot::GridTick を GridMark に修正
                    ticks.push(egui_plot::GridMark {
                        value: i as f64,
                        step_size: input.base_step_size,
                    });
                }
                ticks
            })
            .x_axis_formatter(x_formatter);

        // ステータスごとの棒（Bar）を格納するベクタを準備
        let mut pending_bars = Vec::new();
        let mut wip_bars = Vec::new();
        let mut complete_bars = Vec::new();
        let mut canceled_bars = Vec::new();

        // 過去30日分のデータを古い順（.rev()）にループ処理
        for (i, d) in data.iter().rev().enumerate() {
            let x = i as f64;

            // グラフのホバー時（ツールチップ）に表示する正しい日付を計算
            let start_date = Zoned::now().date();
            let days_to_subtract = (data.len() as i64 - 1) - i as i64;
            let current_date = start_date - days_to_subtract.days();
            let date_str = current_date.strftime("%Y/%m/%d").to_string();

            pending_bars.push(
                Bar::new(x, d.0 as f64)
                    .name(format!("{}: Pending: {}", date_str, d.0))
                    .width(0.6),
            );

            wip_bars.push(
                Bar::new(x, d.1 as f64)
                    .name(format!("{}: WIP: {}", date_str, d.1))
                    .width(0.6),
            );

            complete_bars.push(
                Bar::new(x, d.2 as f64)
                    .name(format!("{}: Complete: {}", date_str, d.2))
                    .width(0.6),
            );

            canceled_bars.push(
                Bar::new(x, d.3 as f64)
                    .name(format!("{}: Canceled: {}", date_str, d.3))
                    .width(0.6),
            );
        }

        // 各ステータスの BarChart を生成し、色を設定
        let chart_pending = BarChart::new("Pending", pending_bars)
            .color(egui::Color32::from_rgb(230, 126, 34));

        let chart_wip = BarChart::new("WIP", wip_bars)
            .color(egui::Color32::from_rgb(52, 152, 219));

        let chart_complete = BarChart::new("Complete", complete_bars)
            .color(egui::Color32::from_rgb(46, 204, 113));

        let chart_canceled = BarChart::new("Canceled", canceled_bars)
            .color(egui::Color32::from_rgb(149, 165, 166));

        // 積み上げの順番を設定
        let chart_complete = chart_complete.stack_on(&[&chart_canceled]);
        let chart_wip = chart_wip.stack_on(&[&chart_canceled, &chart_complete]);
        let chart_pending = chart_pending.stack_on(&[&chart_canceled, &chart_complete, &chart_wip]);

        // プロットを表示して描画
        let inner = my_plot.show(ui, |plot_ui| {
            plot_ui.bar_chart(chart_canceled);
            plot_ui.bar_chart(chart_complete);
            plot_ui.bar_chart(chart_wip);
            plot_ui.bar_chart(chart_pending);
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
