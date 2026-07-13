use egui::{Response, Ui};

#[derive(Debug)]
pub struct StopWatch<'a> {
    start_time: &'a jiff::Zoned,
    total_seconds: i64,
}

impl<'a> StopWatch<'a> {
    pub const fn new(start_time: &'a jiff::Zoned) -> Self {
        Self {
            start_time,
            total_seconds: 0,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) -> Response {
        ui.scope(|ui| {
            let time_diff = self.start_time.duration_since(&jiff::Zoned::now());
            self.total_seconds = time_diff.as_secs().abs();
            let hours = self.total_seconds / 3600;
            let minutes = (self.total_seconds % 3600) / 60;
            let seconds = self.total_seconds % 60;
            let time_str = format!("{hours:02}:{minutes:02}:{seconds:02}");

            let circle_zone_height = ui.available_height() * 0.25;
            let circle_zone_width = ui.available_width();
            let (rect, _response) = ui.allocate_exact_size(
                egui::vec2(circle_zone_width, circle_zone_height),
                egui::Sense::hover(),
            );

            let center = rect.center();
            let circle_radius = circle_zone_height.min(circle_zone_width) * 0.5;
            let painter = ui.painter();

            // 3. ぐるぐる回る点の座標を計算
            let time = ui.input(|i| i.time);
            let angle = time * 3.0;
            let dot_pos =
                center + egui::vec2(angle.cos() as f32, angle.sin() as f32) * circle_radius;

            // 4. 背景の円を描画
            painter.circle_stroke(
                center,
                circle_radius,
                egui::Stroke::new(2.0, egui::Color32::from_gray(60)),
            );

            // 5. ぐるぐる回る点を描画
            painter.circle_filled(dot_pos, 8.0, egui::Color32::LIGHT_BLUE);

            // 6. 中央にテキストを描画
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                time_str,
                egui::FontId::monospace(50.0),
                ui.visuals().text_color(),
            );
        })
        .response
    }

    pub const fn get_total_seconds(&self) -> i64 {
        self.total_seconds
    }
}
