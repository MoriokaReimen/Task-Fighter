use super::main_app::{App, AppState};
use crate::fl;
use crate::stop_watch::StopWatch;
use eframe::egui::{RichText, ScrollArea, TextEdit, Ui};
use std::time::Duration;

impl App {
    pub fn time_view(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        // 毎フレーム再描画を要求（アニメーションを滑らかにするため）
        ui.ctx().request_repaint_after(Duration::from_millis(20));

        egui::CentralPanel::default().show_inside(ui, |ui: &mut Ui| {
            let working_on = fl!("working_on");
            ui.heading(format!("{} {}", working_on, self.temp_task.title));

            let mut stop_watch = StopWatch::new(&self.start_time);
            stop_watch.show(ui);

            // 💡 ボタンが大きくなる（高さ 50px）ので、下のマージンを 80px に広げて被りを防ぐ
            ui.label(RichText::new(fl!("details"))); // ラベルを少し大きめに
            let bottom_margin = 80.0;
            let available_height = (ui.available_height() - bottom_margin).max(100.0);
            let available_size = egui::vec2(ui.available_width(), available_height);
            ScrollArea::vertical()
                .max_height(available_height)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add_sized(
                        available_size,
                        TextEdit::multiline(&mut self.temp_task.detail)
                            .desired_width(ui.available_width()),
                    );
                });
            egui::Area::new(egui::Id::new("close_button_area"))
                .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -20.0))
                .show(ui.ctx(), |ui| {
                    // 💡 Areaの内部幅ではなく、画面全体の横幅（screen_rect）を基準にする
                    let content_width = ui.ctx().content_rect().width();
                    let button_width = (content_width - 40.0).max(100.0); // マイナスにならないよう安全弁(max)も追加
                    let button_size = egui::vec2(button_width, 50.0);

                    // RichText でボタンの文字自体も大きく、太字にする
                    let button_text = RichText::new(fl!("stop_working"))
                        .size(20.0)
                        .strong()
                        .color(ui.visuals().text_color());

                    // add_sized でサイズを指定してボタンを配置
                    if ui
                        .add_sized(button_size, egui::Button::new(button_text))
                        .clicked()
                    {
                        self.state = AppState::Edit;
                        self.temp_task
                            .accumulate_time(stop_watch.get_total_seconds());
                    }
                });
        });
    }
}
