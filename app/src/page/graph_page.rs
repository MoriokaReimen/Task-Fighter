use crate::page::{Page, Pages};
use crate::widget::Graph;
use crate::work::Work;
use core::{CoreOutput, Task};
use eframe::egui::{self, Align, Button, Layout, vec2};
use egui::Ui;
use tracing::info;

pub struct GraphPage {
    graph: Graph,
}

impl GraphPage {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
        }
    }

    fn bottom_panel(&self, ui: &mut egui::Ui, work: &mut Work) -> Pages {
        let mut next_page = Pages::Graph;
        let mut should_close = false;

        // --- Bottom Action Bar ---
        egui::Panel::bottom("bottom_panel").show(ui, |ui: &mut Ui| {
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
            work.task = Task::default();
            next_page = Pages::Main;
            work.tasks = None;
        }

        next_page
    }

    fn central_panel(&mut self, ui: &mut egui::Ui, work: &Work) -> Pages {
        let next_page = Pages::Graph;

        egui::CentralPanel::default().show(ui, |ui: &mut Ui| {
            ui.heading(fl!("task-plot"));
            if !matches!(work.output, CoreOutput::Idle) || work.plot_data.is_none() {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.add(egui::Spinner::new().size(64.0));
                    },
                );
                return;
            }
            if let Some(data) = &work.plot_data {
                self.graph.show(ui, data);
            }
        });

        next_page
    }
}

impl Page for GraphPage {
    /// Renders the task editing page inside a dedicated panel setup.
    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) -> Pages {
        let mut next_page = Pages::Graph;

        let bottom_res = self.bottom_panel(ui, work);
        if bottom_res != Pages::Graph {
            next_page = bottom_res;
        }

        // central_panel の結果が Graph 以外なら更新
        let central_res = self.central_panel(ui, work);
        if central_res != Pages::Graph {
            next_page = central_res;
        }

        next_page
    }
}
