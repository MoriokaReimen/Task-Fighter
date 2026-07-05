use eframe::egui;

pub struct WarningModal {
    title: String,
    message: String,
    is_open: bool,
    id: egui::Id,
}

impl WarningModal {
    pub fn new(id_source: impl std::hash::Hash + std::fmt::Debug) -> Self {
        Self {
            title: String::new(),
            message: String::new(),
            is_open: false,
            id: egui::Id::new(id_source),
        }
    }

    pub fn open(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.is_open = true;
        self.title = title.into();
        self.message = message.into();
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let _modal = egui::Modal::new(self.id).show(ctx, |ui| {
            ui.heading(&self.title);
            ui.add_space(4.0);

            ui.label(&self.message);
            ui.add_space(12.0);
            ui.vertical_centered(|ui| {
                if ui.button("OK").clicked() {
                    ui.close()
                }
            });
        });
    }
}
