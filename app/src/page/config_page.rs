use crate::i18n::I18n;
use crate::page::{Page, Pages};
use crate::widget::MenuBar;
use crate::work::Work;
use core::prelude::*;
use core::{ColorScheme, Config, Locale};
use egui::{self, Align, Button, Grid, Layout, Ui, vec2};
use tracing::info;

trait ColorSchemeExt {
    fn label(&self) -> String;
}

impl ColorSchemeExt for ColorScheme {
    fn label(&self) -> String {
        match self {
            ColorScheme::LightBlue => fl!("light-blue"),
            ColorScheme::DarkOrange => fl!("dark-orange"),
            ColorScheme::WindowsLight => fl!("windows-light"),
            ColorScheme::WindowsDark => fl!("windows-dark"),
            ColorScheme::Sakura => fl!("sakura"),
            ColorScheme::Violet => fl!("violet"),
            ColorScheme::Chrome => fl!("chrome"),
        }
    }
}

trait LocaleExt {
    fn label(&self) -> String;
}

impl LocaleExt for Locale {
    fn label(&self) -> String {
        match self {
            Locale::System => fl!("system"),
            Locale::English => fl!("english"),
            Locale::Japanese => fl!("japanese"),
            Locale::German => fl!("german"),
            Locale::Chinese => fl!("chinese"),
            Locale::Vietnamese => fl!("vietnamese"),
            Locale::Spanish => fl!("spanish"),
        }
    }
}

pub struct ConfigPage {
    back_page: Pages,
    config: Config,
    menu_bar: MenuBar,
}

impl ConfigPage {
    pub fn new() -> Self {
        Self {
            back_page: Pages::Main,
            menu_bar: MenuBar::new(),
            config: Config::default(),
        }
    }
}

impl Page for ConfigPage {
    fn on_entry(&mut self, work: &mut crate::work::Work) {
        self.back_page = work.last_page;
        self.config = work.core.load_config().expect("Failed to load config");
    }

    fn show(&mut self, ui: &mut egui::Ui, work: &mut Work) {
        self.menu_bar.show(ui, work);

        egui::Panel::bottom("config_bottom_panel").show(ui, |ui: &mut Ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(Button::new(fl!("close")).min_size(vec2(90.0, 28.0)))
                    .clicked()
                {
                    info!("Close Button Pressed");
                    work.next_page = self.back_page;
                }
            });
        });

        egui::CentralPanel::default().show(ui, |ui: &mut Ui| {
            ui.heading(fl!("setting"));
            Grid::new("config_grid")
                .striped(false)
                .num_columns(2)
                .spacing(vec2(16.0, 12.0))
                .show(ui, |ui| {
                    ui.label(fl!("color-scheme-label"));
                    let mut theme_changed = false;
                    egui::ComboBox::from_id_salt("color_scheme_picker")
                        .selected_text(self.config.color_scheme.label())
                        .show_ui(ui, |ui| {
                            theme_changed |= ui
                                .selectable_value(
                                    &mut self.config.color_scheme,
                                    ColorScheme::LightBlue,
                                    &ColorScheme::LightBlue.label(),
                                )
                                .changed();
                            theme_changed |= ui
                                .selectable_value(
                                    &mut self.config.color_scheme,
                                    ColorScheme::DarkOrange,
                                    &ColorScheme::DarkOrange.label(),
                                )
                                .changed();
                            theme_changed |= ui
                                .selectable_value(
                                    &mut self.config.color_scheme,
                                    ColorScheme::WindowsLight,
                                    &ColorScheme::WindowsLight.label(),
                                )
                                .changed();
                            theme_changed |= ui
                                .selectable_value(
                                    &mut self.config.color_scheme,
                                    ColorScheme::WindowsDark,
                                    &ColorScheme::WindowsDark.label(),
                                )
                                .changed();
                            theme_changed |= ui
                                .selectable_value(
                                    &mut self.config.color_scheme,
                                    ColorScheme::Sakura,
                                    &ColorScheme::Sakura.label(),
                                )
                                .changed();
                            theme_changed |= ui
                                .selectable_value(
                                    &mut self.config.color_scheme,
                                    ColorScheme::Violet,
                                    &ColorScheme::Violet.label(),
                                )
                                .changed();
                            theme_changed |= ui
                                .selectable_value(
                                    &mut self.config.color_scheme,
                                    ColorScheme::Chrome,
                                    &ColorScheme::Chrome.label(),
                                )
                                .changed();
                        });
                    if theme_changed {
                        work.config.color_scheme = self.config.color_scheme;
                    }
                    ui.end_row();

                    ui.label(fl!("locale-label"));
                    let mut locale_changed = false;
                    egui::ComboBox::from_id_salt("locale_picker")
                        .selected_text(self.config.locale.label())
                        .show_ui(ui, |ui| {
                            locale_changed |= ui
                                .selectable_value(
                                    &mut self.config.locale,
                                    Locale::System,
                                    &Locale::System.label(),
                                )
                                .changed();
                            locale_changed |= ui
                                .selectable_value(
                                    &mut self.config.locale,
                                    Locale::Japanese,
                                    &Locale::Japanese.label(),
                                )
                                .changed();
                            locale_changed |= ui
                                .selectable_value(
                                    &mut self.config.locale,
                                    Locale::English,
                                    &Locale::English.label(),
                                )
                                .changed();
                            locale_changed |= ui
                                .selectable_value(
                                    &mut self.config.locale,
                                    Locale::Chinese,
                                    &Locale::Chinese.label(),
                                )
                                .changed();
                            locale_changed |= ui
                                .selectable_value(
                                    &mut self.config.locale,
                                    Locale::German,
                                    &Locale::German.label(),
                                )
                                .changed();
                            locale_changed |= ui
                                .selectable_value(
                                    &mut self.config.locale,
                                    Locale::Vietnamese,
                                    &Locale::Vietnamese.label(),
                                )
                                .changed();
                            locale_changed |= ui
                                .selectable_value(
                                    &mut self.config.locale,
                                    Locale::Spanish,
                                    &Locale::Spanish.label(),
                                )
                                .changed();
                        });
                    if locale_changed {
                        work.config.locale = self.config.locale;
                        I18n::global().set_locale_from_config(self.config.locale);
                    }
                    ui.end_row();
                });
        });
    }

    fn on_exit(&mut self, work: &mut crate::work::Work) {
        work.core
            .save_config(&self.config)
            .expect("Failed to save config");
    }
}
