use eframe::egui;
use egui_plot::{Bar, BarChart, GridInput, Plot, PlotTransform};
use jiff::{ToSpan, Zoned};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::log::{error, info};

pub struct Graph {
    plot_rect: Option<egui::Rect>,
    should_save: Arc<AtomicBool>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            plot_rect: None,
            should_save: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn save_screenshot(&self) {
        self.should_save.store(true, Ordering::SeqCst);
    }
}

impl Graph {
    pub fn show(&mut self, ui: &mut egui::Ui, data: &[(i32, i32, i32, i32)]) -> egui::Response {
        let response = self.draw_plot(ui, data);

        if self.check_save_trigger() {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }

        #[cfg(not(target_arch = "wasm32"))]
        self.handle_screenshot_event(ui.ctx());

        response
    }
}

impl Graph {
    fn check_save_trigger(&self) -> bool {
        self.should_save
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn handle_screenshot_event(&self, ctx: &egui::Context) {
        let Some(rect) = self.plot_rect else { return };

        let screenshot = ctx.input(|i| {
            i.events.iter().find_map(|event| match event {
                egui::Event::Screenshot { image, .. } => Some(Arc::clone(image)),
                _ => None,
            })
        });

        let Some(screenshot_img) = screenshot else {
            return;
        };

        let pixels_per_point = ctx.pixels_per_point();
        std::thread::spawn(move || {
            let Some(mut path) = rfd::FileDialog::new().save_file() else {
                return;
            };
            path.set_extension("png");

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

    /// グラフ全体のメイン描画処理
    fn draw_plot(&mut self, ui: &mut egui::Ui, data: &[(i32, i32, i32, i32)]) -> egui::Response {
        let max_x = if data.is_empty() {
            0.0
        } else {
            (data.len() - 1) as f64
        };
        let max_y = calculate_max_y(data);

        // 各ステータスの積み上げグラフ（BarChart）を生成
        let charts = build_stacked_charts(data);

        // プロット表示の設定を構築
        let task_plot = Plot::new(fl!("task-plot"))
            .y_axis_label(fl!("number-of-tasks"))
            .include_x(-0.5)
            .include_x(max_x + 0.5)
            .include_y(0.0)
            .include_y(max_y * 1.1)
            .x_grid_spacer(grid_spacer)
            .x_axis_formatter(move |mark, _| {
                get_date_by_index(data.len(), mark.value.round() as i64)
                    .map(|d| d.strftime("%y/%m/%d").to_string())
                    .unwrap_or_default()
            })
            .allow_double_click_reset(true);

        let mut transform = None;
        let inner = task_plot.show(ui, |plot_ui| {
            plot_ui.bar_chart(charts.canceled);
            plot_ui.bar_chart(charts.complete);
            plot_ui.bar_chart(charts.wip);
            plot_ui.bar_chart(charts.pending);

            transform = Some(*plot_ui.transform());
        });

        // カスタム凡例を重ねて描画
        if let Some(trans) = transform {
            render_legend(ui, &inner.response, &trans);
        }

        self.plot_rect = Some(inner.response.rect);
        inner.response
    }
}

// --- 独立したヘルパー関数（リファクタリング用） ---

struct StackedCharts {
    pending: BarChart,
    wip: BarChart,
    complete: BarChart,
    canceled: BarChart,
}

/// インデックスから相対的な過去の日付を計算する
fn get_date_by_index(data_len: usize, index: i64) -> Option<jiff::civil::Date> {
    if index >= 0 && index < data_len as i64 {
        let start_date = Zoned::now().date();
        let days_to_subtract = (data_len as i64 - 1) - index;
        Some(start_date - days_to_subtract.days())
    } else {
        None
    }
}

/// グラフ全体の最大Y軸値を計算
fn calculate_max_y(data: &[(i32, i32, i32, i32)]) -> f64 {
    data.iter()
        .map(|(p, w, c, ca)| p + w + c + ca)
        .max()
        .map_or(10.0, f64::from)
}

/// 積み上げデータ用の `BarChart` 群を構築する
fn build_stacked_charts(data: &[(i32, i32, i32, i32)]) -> StackedCharts {
    let mut pending = Vec::new();
    let mut wip = Vec::new();
    let mut complete = Vec::new();
    let mut canceled = Vec::new();

    for (i, d) in data.iter().rev().enumerate() {
        let x = i as f64;
        let date_str = get_date_by_index(data.len(), i as i64)
            .map(|d| d.strftime("%Y/%m/%d").to_string())
            .unwrap_or_default();

        pending.push(
            Bar::new(x, d.0.into())
                .name(format!("{}: {}: {}", date_str, fl!("pending"), d.0))
                .width(0.6),
        );
        wip.push(
            Bar::new(x, d.1.into())
                .name(format!(
                    "{}: {}: {}",
                    date_str,
                    fl!("work-in-progress"),
                    d.1
                ))
                .width(0.6),
        );
        complete.push(
            Bar::new(x, d.2.into())
                .name(format!("{}: {}: {}", date_str, fl!("complete"), d.2))
                .width(0.6),
        );
        canceled.push(
            Bar::new(x, d.3.into())
                .name(format!("{}: {}: {}", date_str, fl!("cancel"), d.3))
                .width(0.6),
        );
    }

    let c_chart = BarChart::new("", canceled).color(egui::Color32::from_rgb(234, 110, 110));
    let o_chart = BarChart::new("", complete)
        .color(egui::Color32::from_rgb(78, 205, 151))
        .stack_on(&[&c_chart]);
    let w_chart = BarChart::new("", wip)
        .color(egui::Color32::from_rgb(246, 160, 84))
        .stack_on(&[&c_chart, &o_chart]);
    let p_chart = BarChart::new("", pending)
        .color(egui::Color32::from_rgb(140, 160, 180))
        .stack_on(&[&c_chart, &o_chart, &w_chart]);

    StackedCharts {
        pending: p_chart,
        wip: w_chart,
        complete: o_chart,
        canceled: c_chart,
    }
}

/// X軸の目盛り間隔をズーム具合に合わせて動的に計算する
#[allow(clippy::needless_pass_by_value)]
fn grid_spacer(input: GridInput) -> Vec<egui_plot::GridMark> {
    let start = input.bounds.0.floor() as i64;
    let end = input.bounds.1.ceil() as i64;
    let visible_width = input.bounds.1 - input.bounds.0;

    let step = if visible_width <= 7.0 {
        1
    } else if visible_width <= 15.0 {
        2
    } else if visible_width <= 45.0 {
        5
    } else if visible_width <= 90.0 {
        10
    } else {
        30
    };

    (start..=end)
        .filter(|&i| i % step == 0)
        .map(|i| egui_plot::GridMark {
            value: i as f64,
            step_size: step as f64,
        })
        .collect()
}

/// グラフ左上にカスタム凡例（カラーチップとテキスト）を描画する
fn render_legend(ui: &egui::Ui, response: &egui::Response, transform: &PlotTransform) {
    let bounds = transform.bounds();

    // mul_add を視覚的にわかりやすい標準的な数式表記に整理
    let plot_left = (bounds.max()[0] - bounds.min()[0]).mul_add(0.03, bounds.min()[0]);
    let plot_top = (bounds.max()[1] - bounds.min()[1]).mul_add(-0.04, bounds.max()[1]);

    let screen_start =
        transform.position_from_point(&egui_plot::PlotPoint::new(plot_left, plot_top));
    let mut current_y = screen_start.y;

    let painter = ui.painter_at(response.rect);
    let font_id = egui::FontId::proportional(12.0);

    let items = [
        (fl!("pending"), egui::Color32::from_rgb(140, 160, 180)),
        (
            fl!("work-in-progress"),
            egui::Color32::from_rgb(246, 160, 84),
        ),
        (fl!("complete"), egui::Color32::from_rgb(78, 205, 151)),
        (fl!("cancel"), egui::Color32::from_rgb(234, 110, 110)),
    ];

    for (text, color) in items {
        let rect = egui::Rect::from_min_size(
            egui::pos2(screen_start.x, current_y),
            egui::vec2(14.0, 14.0),
        );
        painter.rect_filled(rect, 2.0, color);

        painter.text(
            egui::pos2(screen_start.x + 22.0, current_y + 7.0),
            egui::Align2::LEFT_CENTER,
            text,
            font_id.clone(),
            ui.visuals().text_color(),
        );

        current_y += 22.0;
    }
}
