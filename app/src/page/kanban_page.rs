use crate::page::{Page, Pages};
use crate::widget::KanbanArea;
use crate::work::Work;
use egui::{self, Align, Button, Color32, Layout, ScrollArea, Ui, vec2};
use tracing::{error, info};

pub struct KanbanPage {
    kanban_area: KanbanArea,
}

impl KanbanPage {
    pub fn new() -> Self {
        Self {
            kanban_area: KanbanArea::default(),
        }
    }

    fn render_top_panel(&mut self, ui: &mut Ui, work: &mut Work, next_page: &mut Pages) {
        egui::Panel::top("kanban_top_panel").show(ui, |ui| {
            ui.heading(fl!("kanban"));
        });
    }

    fn render_centeral_panel(&mut self, ui: &mut Ui, work: &mut Work, next_page: &mut Pages) {
        egui::CentralPanel::default().show(ui, |ui| {
            self.kanban_area.show(ui);
        });
    }

    fn render_bottom_panel(&mut self, ui: &mut Ui, work: &mut Work, next_page: &mut Pages) {
        egui::Panel::bottom("kanban_bottom_panel").show(ui, |ui| {
            egui::containers::Sides::new().show(
                ui,
                |ui| {},
                |ui| {
                    if ui
                        .add(Button::new(fl!("back")).min_size(vec2(120.0, 28.0)))
                        .clicked()
                    {
                        *next_page = Pages::Main;
                    }
                },
            );
        });
    }
}

impl Page for KanbanPage {
    fn on_entry(&mut self, work: &mut crate::work::Work) {}

    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) {
        let mut next_page = Pages::Kanban;
        self.render_top_panel(ui, work, &mut next_page);
        self.render_bottom_panel(ui, work, &mut next_page);
        self.render_centeral_panel(ui, work, &mut next_page);

        work.next_page = next_page;
    }

    fn on_exit(&mut self, work: &mut crate::work::Work) {}
}
