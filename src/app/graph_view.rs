use super::main_app::{App, AppState};
use crate::driver::Task;
use crate::fl;
use eframe::egui::{self, Align, Button, Layout, vec2};
use egui::Ui;
use rand::RngExt; // 💡 rand 0.8 用のRngトレイトをインポート
use tracing::info;

impl App {
    /// Renders the task editing view inside a dedicated panel setup.
    pub fn graph_view(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let mut should_close = false;

        // --- Bottom Action Bar ---
        egui::Panel::bottom("bottom_panel").show_inside(ui, |ui: &mut Ui| {
            // Right-to-left layout places buttons from rightmost to leftmost
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Cancel Button Action
                if ui
                    .add(Button::new(fl!("close")).min_size(vec2(90.0, 28.0)))
                    .clicked()
                {
                    info!("Close Button Pressed");
                    should_close = true;
                }
                self.graph.show_controls(ui);
            });
        });

        if should_close {
            self.temp_task = Task::default();
            self.state = AppState::Default;
            self.displayed_tasks = None;
        }

        egui::CentralPanel::default().show_inside(ui, |ui: &mut Ui| {
            ui.heading(fl!("graph"));
            self.graph.show(ui);
        });
    }
}

fn generate_random_30_days() -> Vec<f64> {
    // 💡 修正4: rand 0.8 の thread_rng() に戻します
    let mut rng = rand::rng();
    (0..30).map(|_| rng.random_range(10.0..=100.0)).collect()
}
