use eframe::egui;

pub struct AboutModal {
    is_open: bool,
    id: egui::Id,
}

impl AboutModal {
    pub fn new() -> Self {
        Self {
            is_open: false,
            id: egui::Id::new("about-modal"),
        }
    }

    pub const fn open(&mut self) {
        self.is_open = true;
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let _modal = egui::Modal::new(self.id).show(ctx, |ui| {
            ui.heading(fl!("about-task-fighter"));
            ui.add_space(4.0);
            let git_sha = env!("VERGEN_GIT_SHA");
            let git_tag = env!("VERGEN_GIT_DESCRIBE");

            ui.label("Author: MoriokaReimen".to_string());
            ui.hyperlink_to(
                "Github Repository",
                "https://github.com/MoriokaReimen/Task-Fighter",
            );
            ui.label("License: Apache License 2.0".to_string());
            ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
            ui.label(format!("Git Commit SHA: {git_sha}"));
            ui.label(format!("Git Tag/Describe: {git_tag}"));

            ui.add_space(12.0);

            egui::Sides::new().show(
                ui,
                |_ui| {},
                |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(fl!("close")).clicked() {
                            self.is_open = false;
                        }
                    });
                },
            );
        });

        ();
    }
}
