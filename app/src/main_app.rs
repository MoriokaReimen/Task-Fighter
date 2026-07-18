use crate::app::App;
use anyhow::Result;

/// Generates window configuration and initializes the application icon.
fn get_frame_option() -> Result<eframe::NativeOptions> {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon_bytes)?.to_rgba8();
    let (width, height) = image.dimensions();
    let rgba_pixels = image.into_raw();

    let icon_data = egui::IconData {
        rgba: rgba_pixels,
        width,
        height,
    };
    let initial_size = egui::vec2(1280.0, 832.0);
    Ok(eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Task Fighter")
            .with_icon(icon_data)
            .with_inner_size(initial_size)
            .with_min_inner_size(initial_size),
        ..Default::default()
    })
}

/// Main entry point for launching the native GUI application.
pub fn start_app() -> Result<()> {
    let native_options = get_frame_option()?;

    eframe::run_native(
        "Task Fighter",
        native_options,
        Box::new(|cc| {
            let app: Box<dyn eframe::App> = Box::new(App::new(&cc.egui_ctx));
            Ok(app)
        }),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {e:?}"))?;

    Ok(())
}
