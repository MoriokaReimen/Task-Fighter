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
            Self::LightBlue => fl!("light-blue"),
            Self::DarkOrange => fl!("dark-orange"),
            Self::WindowsLight => fl!("windows-light"),
            Self::WindowsDark => fl!("windows-dark"),
            Self::Sakura => fl!("sakura"),
            Self::Violet => fl!("violet"),
            Self::Chrome => fl!("chrome"),
        }
    }
}

const COLOR_SCHEMES: [ColorScheme; 7] = [
    ColorScheme::LightBlue,
    ColorScheme::DarkOrange,
    ColorScheme::WindowsLight,
    ColorScheme::WindowsDark,
    ColorScheme::Sakura,
    ColorScheme::Violet,
    ColorScheme::Chrome,
];

trait LocaleExt {
    fn label(&self) -> String;
}

impl LocaleExt for Locale {
    fn label(&self) -> String {
        match self {
            Self::System => fl!("system"),
            Self::English => fl!("english"),
            Self::Japanese => fl!("japanese"),
            Self::German => fl!("german"),
            Self::Chinese => fl!("chinese"),
            Self::Vietnamese => fl!("vietnamese"),
            Self::Spanish => fl!("spanish"),
        }
    }
}

const LOCALE_OPTIONS: [Locale; 7] = [
    Locale::System,
    Locale::Japanese,
    Locale::English,
    Locale::Chinese,
    Locale::German,
    Locale::Vietnamese,
    Locale::Spanish,
];

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
        info!("Enter to Config Page");
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
                            for scheme in COLOR_SCHEMES {
                                theme_changed |= ui
                                    .selectable_value(
                                        &mut self.config.color_scheme,
                                        scheme,
                                        scheme.label(),
                                    )
                                    .changed();
                            }
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
                            for locale in LOCALE_OPTIONS {
                                locale_changed |= ui
                                    .selectable_value(
                                        &mut self.config.locale,
                                        locale,
                                        locale.label(),
                                    )
                                    .changed();
                            }
                        });
                    if locale_changed {
                        work.config.locale = self.config.locale;
                        I18n::global().set_locale_from_config(self.config.locale);
                    }
                    ui.end_row();

                    ui.label(fl!("email-locale-label"));
                    let mut email_locale_changed = false;
                    egui::ComboBox::from_id_salt("email_locale_picker")
                        .selected_text(self.config.email_locale.label())
                        .show_ui(ui, |ui| {
                            for email_locale in LOCALE_OPTIONS {
                                email_locale_changed |= ui
                                    .selectable_value(
                                        &mut self.config.email_locale,
                                        email_locale,
                                        email_locale.label(),
                                    )
                                    .changed();
                            }
                        });
                    if email_locale_changed {
                        work.config.email_locale = self.config.email_locale;
                    }
                    ui.end_row();
                });
        });
    }

    fn on_exit(&mut self, work: &mut crate::work::Work) {
        info!("Exit from Config Page");
        work.core
            .save_config(&self.config)
            .expect("Failed to save config");
    }
}
