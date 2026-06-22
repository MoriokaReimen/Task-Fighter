use eframe::egui;
use egui::{Color32, CornerRadius, Stroke};

/// Global setup for application style, including scale, fonts, and spacing.
pub fn setup_style(ctx: &egui::Context) {
    // Adjust scale and load custom fonts
    ctx.set_pixels_per_point(1.5);
    setup_custom_fonts(ctx);

    // Fine-tune global spacing layout
    ctx.global_style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.window_margin = egui::Margin::same(10);
    });

    // Apply the custom visual theme
    set_theme(ctx);
}

/// Defines and applies the dark cyan/aqua custom theme.
pub fn set_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // --- Base Settings ---
    visuals.dark_mode = true;
    visuals.override_text_color = Some(Color32::from_rgb(0, 192, 220)); // Clean aqua white
    visuals.weak_text_color = Some(Color32::from_rgb(130, 150, 160)); // Soft slate gray
    visuals.weak_text_alpha = 0.6;
    visuals.disabled_alpha = 0.5;

    // --- Backgrounds & Panels ---
    visuals.panel_fill = Color32::from_rgb(16, 24, 30); // Main panel: deep aqua navy
    visuals.window_fill = Color32::from_rgb(24, 34, 42); // Windows: midnight blue
    visuals.faint_bg_color = Color32::from_rgb(30, 44, 54); // Zebra stripes / subtle backgrounds
    visuals.extreme_bg_color = Color32::from_rgb(10, 16, 20); // Recessed backgrounds (e.g., text edits)

    // --- Input & Code Fields ---
    visuals.text_edit_bg_color = Some(Color32::from_rgb(10, 16, 20));
    visuals.code_bg_color = Color32::from_rgb(20, 30, 38);

    // --- Accent & Semantic Colors ---
    visuals.hyperlink_color = Color32::from_rgb(0, 210, 255); // Vibrant clear cyan
    visuals.warn_fg_color = Color32::from_rgb(240, 180, 50); // Warning: muted amber gold
    visuals.error_fg_color = Color32::from_rgb(255, 90, 120); // Error: soft coral pink

    // --- Geometry, Strokes & Borders ---
    visuals.window_corner_radius = CornerRadius::same(6);
    visuals.menu_corner_radius = CornerRadius::same(4);
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(40, 60, 75)); // Ice-blue border
    visuals.window_highlight_topmost = true;

    // --- Gadgets & UI Behavior ---
    visuals.button_frame = true;
    visuals.collapsing_header_frame = false;
    visuals.indent_has_left_vline = true;
    visuals.striped = true;
    visuals.slider_trailing_fill = true;
    visuals.image_loading_spinners = true;

    visuals.resize_corner_size = 10.0;
    visuals.clip_rect_margin = 3.0;
    visuals.interact_cursor = None;

    // --- Widget State Colors ---
    let text_color = Color32::from_rgb(225, 240, 245);

    // 1. Inactive State (Default/Idle)
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(32, 46, 56);
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(26, 38, 48);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(50, 72, 88));
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_color);

    // 2. Hovered State (Mouseover glowing effect)
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(42, 64, 80);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(0, 140, 180);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0, 210, 255));
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);

    // 3. Active State (Click / Pressed neon effect)
    visuals.widgets.active.bg_fill = Color32::from_rgb(0, 170, 220);
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(0, 170, 220);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(140, 240, 255));
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);

    // 4. Selection State (Text highlights)
    visuals.selection.bg_fill = Color32::from_rgb(0, 100, 140);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(160, 235, 255));

    ctx.set_visuals(visuals);
}

/// Loads Noto Sans JP and registers it as the primary fallback font.
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Insert Japanese TrueType Font data
    fonts.font_data.insert(
        "japanese_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../../asset/NotoSansJP-Regular.ttf")).into(),
    );

    // Set Japanese font as top priority for Proportional text
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "japanese_font".to_owned());

    // Set Japanese font as top priority for Monospace text (code fields)
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "japanese_font".to_owned());

    ctx.set_fonts(fonts);
}
