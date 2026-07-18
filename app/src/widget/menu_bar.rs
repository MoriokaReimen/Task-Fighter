use crate::page::Pages;
use crate::widget::AboutModal;
use crate::work::Work;
use egui::{self, Ui};
use tracing::info;

pub struct MenuBar {
    about_modal: AboutModal,
}

impl MenuBar {
    pub fn new() -> Self {
        Self {
            about_modal: AboutModal::new(),
        }
    }

    pub fn show(&mut self, ui: &mut Ui, work: &mut Work) {
        egui::Panel::top("top_menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button(fl!("menu"), |ui| {
                    if ui.button(fl!("edit-task")).clicked() {
                        info!("Switch to Main Page");
                        work.next_page = Pages::Main;
                    }
                    if ui.button(fl!("edit-daily-task")).clicked() {
                        info!("Switch to Daily Main Page");
                        work.next_page = Pages::DailyMain;
                    }
                    if ui.button(fl!("edit-weekly-task")).clicked() {
                        info!("Switch to Weekly Main Page");
                        work.next_page = Pages::WeeklyMain;
                    }
                    if ui.button(fl!("edit-monthly-task")).clicked() {
                        info!("Switch to Monthly Main Page");
                        work.next_page = Pages::MonthlyMain;
                    }
                    if ui.button(fl!("setting")).clicked() {
                        work.next_page = Pages::Config;
                    }
                    if ui.button(fl!("about")).clicked() {
                        self.about_modal.open();
                    }
                    if ui.button(fl!("quit")).clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });
        let ctx = ui.ctx();
        self.about_modal.show(ctx);
    }
}
