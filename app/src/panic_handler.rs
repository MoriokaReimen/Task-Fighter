use egui::{self, Color32, RichText};
use tracing::error;

#[derive(Clone, Debug)]
pub struct AppInfo {
    pub name: &'static str,
    pub additional_text: &'static str,
    pub links: Vec<Link>,
}

#[derive(Clone, Debug)]
pub struct Link {
    pub label: &'static str,
    pub url: &'static str,
}

/// Builds a plain-text summary of the panic, suitable for copying into a
/// bug report.
pub fn details(
    panic_payload_display: Option<&String>,
    panic_formatted: &str,
    app_info: &AppInfo,
) -> String {
    let payload_display = panic_payload_display
        .map(String::as_str)
        .unwrap_or("[PAYLOAD IS NOT A STRING]");
    let name = app_info.name;
    let pkg_name = env!("CARGO_PKG_NAME");
    let pkg_version = env!("CARGO_PKG_VERSION");

    [
        format!("**Panic report from {payload_display}**"),
        name.to_string(),
        format!("Package name: `{pkg_name}`\nVersion: `{pkg_version}`"),
        format!("Panic info:\n```\n{panic_formatted}\n```"),
    ]
    .join("\n\n")
}

/// Shows a small native window describing the panic, and blocks until
/// the user closes it.
pub fn show_gui_egui(
    panic_payload_display: Option<String>,
    panic_formatted: String,
    app_info: AppInfo,
) {
    eframe::run_ui_native(
        "Crash report",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_maximize_button(false)
                .with_always_on_top()
                .with_inner_size([512.0, 256.0]),
            ..Default::default()
        },
        move |ui, _frame| {
            render_panel(
                ui,
                panic_payload_display.as_ref(),
                &panic_formatted,
                &app_info,
            );
        },
    )
    .unwrap();
}

fn render_panel(
    ui: &mut egui::Ui,
    panic_payload_display: Option<&String>,
    panic_formatted: &str,
    app_info: &AppInfo,
) {
    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(ui.available_width());
            render_header(ui, panic_payload_display, panic_formatted, app_info);
        });
    });
}

fn render_header(
    ui: &mut egui::Ui,
    panic_payload_display: Option<&String>,
    panic_formatted: &str,
    app_info: &AppInfo,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("⚠").size(48.0).color(Color32::RED));
        ui.add_space(16.0);
        ui.vertical(|ui| {
            render_body(ui, panic_payload_display, panic_formatted, app_info);
        });
    });
}

fn render_body(
    ui: &mut egui::Ui,
    panic_payload_display: Option<&String>,
    panic_formatted: &str,
    app_info: &AppInfo,
) {
    ui.heading(format!("{} crashed", app_info.name));
    ui.add_space(8.0);
    ui.label(app_info.additional_text);

    render_reason(ui, panic_payload_display);
    ui.add_space(8.0);

    render_action_buttons(ui, panic_payload_display, panic_formatted, app_info);
    ui.add_space(8.0);

    render_links(ui, &app_info.links);
    ui.add_space(16.0);

    render_package_meta(ui);

    ui.collapsing("Developer information", |ui| {
        ui.monospace(panic_formatted);
    });
}

fn render_reason(ui: &mut egui::Ui, payload: Option<&String>) {
    let Some(payload) = payload else { return };
    ui.horizontal_wrapped(|ui| {
        ui.strong("Reason:");
        ui.monospace(payload);
    });
}

fn render_action_buttons(
    ui: &mut egui::Ui,
    panic_payload_display: Option<&String>,
    panic_formatted: &str,
    app_info: &AppInfo,
) {
    ui.horizontal_wrapped(|ui| {
        if ui.button("📋 Copy details").clicked() {
            let text = details(panic_payload_display, panic_formatted, app_info);
            ui.ctx().copy_text(text);
        }
    });
}

fn render_links(ui: &mut egui::Ui, links: &[Link]) {
    ui.horizontal_wrapped(|ui| {
        let mut iter = links.iter();
        if let Some(link) = iter.next() {
            open_link_if_clicked(ui, link);
        }
        for link in iter {
            ui.separator();
            open_link_if_clicked(ui, link);
        }
    });
}

fn open_link_if_clicked(ui: &mut egui::Ui, link: &Link) {
    if ui.link(link.label).clicked() {
        ui.ctx().open_url(egui::OpenUrl {
            url: link.url.to_owned(),
            new_tab: true,
        });
    }
}

fn render_package_meta(ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.strong("Package name:");
        ui.monospace(env!("CARGO_PKG_NAME"));
    });
    ui.horizontal_wrapped(|ui| {
        ui.strong("Version:");
        ui.label(env!("CARGO_PKG_VERSION"));
    });
}

/// Extracts a human-readable panic message, if the payload is a `&str` or
/// `String` (the two common cases for `std::panic!`).
fn extract_payload_display(payload: &(dyn std::any::Any + Send)) -> Option<String> {
    payload
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
}

/// Installs a panic hook that logs the panic and shows a native crash
/// report window with the given app info.
pub fn register(app_info: AppInfo) {
    std::panic::set_hook(Box::new(move |panic_info| {
        handle_panic(panic_info, &app_info);
    }));
}

fn handle_panic(panic_info: &std::panic::PanicHookInfo, app_info: &AppInfo) {
    let panic_formatted = format!("{panic_info:#?}");
    let panic_payload_display = extract_payload_display(panic_info.payload());

    error!("The app panicked.");
    error!("Panic info: {panic_formatted}");

    match &panic_payload_display {
        Some(payload) => println!("Panic payload: {payload}"),
        None => println!("Panic payload doesn't implement `Display`"),
    }

    show_gui_egui(panic_payload_display, panic_formatted, app_info.clone());
}
