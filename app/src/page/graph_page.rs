use crate::page::{Page, Pages};
use crate::widget::Graph;
use crate::widget::MenuBar;
use crate::work::Work;
use core::Task;
use egui::Ui;
use egui::{self, Align, Button, Layout, vec2};
use tracing::info;

pub struct GraphPage {
    graph: Graph,
    menu_bar: MenuBar,
}

impl GraphPage {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            menu_bar: MenuBar::new(),
        }
    }

    fn bottom_panel(&self, ui: &mut egui::Ui, work: &mut Work) {
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
            work.next_page = Pages::Main;
            work.tasks = None;
        }
    }

    fn central_panel(&mut self, ui: &mut egui::Ui, work: &mut Work) {
        egui::CentralPanel::default().show(ui, |ui: &mut Ui| {
            ui.heading(fl!("task-plot"));
            if !work.outputs.is_empty() || work.plot_data.is_none() {
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
    }
}

impl Page for GraphPage {
    fn on_entry(&mut self, _: &mut crate::work::Work) {}

    /// Renders the task editing page inside a dedicated panel setup.
    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) {
        self.menu_bar.show(ui, work);
        self.bottom_panel(ui, work);
        self.central_panel(ui, work);
    }
    fn on_exit(&mut self, _: &mut crate::work::Work) {}
}
