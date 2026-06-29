use super::main_app::{App, AppState};
use crate::core::CoreOutput;
use crate::driver::Task;
use crate::fl;
use eframe::egui::{self, Align, Button, Layout, vec2};
use egui::Ui;
// 💡 rand 0.8 用のRngトレイトをインポート
use tracing::info;

impl App {
    /// Renders the task editing view inside a dedicated panel setup.
    pub fn graph_view(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        self.bottom_panel(ui);
        self.central_panel(ui);
    }

    fn bottom_panel(&mut self, ui: &mut egui::Ui) {
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

                if ui
                    .add(Button::new(fl!("save-graph")).min_size(vec2(180.0, 28.0)))
                    .clicked()
                {
                    info!("Save Screenshot Button Pressed");
                    self.graph.save_screenshot();
                }
            });
        });

        if should_close {
            self.temp_task = Task::default();
            self.state = AppState::Default;
            self.displayed_tasks = None;
        }
    }

    fn central_panel(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui: &mut Ui| {
            ui.heading(fl!("task-plot"));
            if !matches!(self.output, CoreOutput::Idle) || self.plot_data.is_none() {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.add(egui::Spinner::new().size(64.0));
                    },
                );
                return;
            }
            if let Some(data) = &self.plot_data {
                self.graph.show(ui, data);
            }
        });
    }
}
