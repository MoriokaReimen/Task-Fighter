use crate::work::Work;
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
}

/// Defines and applies the dark cyan/aqua custom theme.
pub fn set_theme(ctx: &egui::Context, work: &Work) {
    match work.config.color_scheme {
        _ => set_dark_orange(ctx),
    }
}

fn set_dark_orange(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // --- Base Settings (Terminal in the Dark Abyss) ---
    visuals.dark_mode = true;
    visuals.override_text_color = Some(Color32::from_rgb(245, 230, 215)); // Luminous cream white text for readability
    visuals.weak_text_color = Some(Color32::from_rgb(150, 100, 60)); // Muted amber-brown for subtext
    visuals.weak_text_alpha = 0.8;
    visuals.disabled_alpha = 0.35;

    // --- Backgrounds & Panels (Deep Incinerator Void - NO GRAY) ---
    visuals.panel_fill = Color32::from_rgb(20, 12, 5); // Main canvas: Ultra-dark espresso/burnt wood void
    visuals.window_fill = Color32::from_rgb(32, 20, 8); // Popups/Windows: Very deep mahogany shadow
    visuals.faint_bg_color = Color32::from_rgb(45, 28, 12); // Zebra rows: Subdued dark copper
    visuals.extreme_bg_color = Color32::from_rgb(12, 6, 2); // Recessed fields: Pitch-black amber floor

    // --- Input & Code Fields ---
    visuals.text_edit_bg_color = Some(Color32::from_rgb(12, 6, 2));
    visuals.code_bg_color = Color32::from_rgb(25, 15, 5);

    // --- Accent & Semantic Colors ---
    visuals.hyperlink_color = Color32::from_rgb(255, 180, 80); // Bright warm amber links
    visuals.warn_fg_color = Color32::from_rgb(255, 215, 0); // Warning: Searing yellow laser
    visuals.error_fg_color = Color32::from_rgb(255, 70, 70); // Error: Neurotoxin vent active red

    // --- Geometry, Strokes & Borders ---
    visuals.window_corner_radius = CornerRadius::same(4); // Crisp industrial corners
    visuals.menu_corner_radius = CornerRadius::same(2);
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(80, 50, 25)); // Deep copper outlines
    visuals.window_highlight_topmost = true;

    // --- Gadgets & UI Behavior ---
    visuals.button_frame = true;
    visuals.collapsing_header_frame = false;
    visuals.indent_has_left_vline = true;
    visuals.striped = true;
    visuals.slider_trailing_fill = true; // Slider trail will burn with orange energy
    visuals.image_loading_spinners = true;

    visuals.resize_corner_size = 8.0;
    visuals.clip_rect_margin = 2.0;
    visuals.interact_cursor = None;

    // --- Widget State Colors (Luminous Aperture Orange Buttons) ---
    let button_text_dark = Color32::from_rgb(25, 12, 0); // Dark contrast text for inside bright orange buttons

    // 1. Inactive State (Standard Glowing Orange Buttons)
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(235, 110, 15); // Pure, solid Aperture Safety Orange
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(180, 80, 5); // Under-layers: slightly darker warm orange
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(255, 140, 40)); // Distinct outer energy rim
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.3, button_text_dark); // Sharp dark text over the orange block

    // 2. Hovered State (Overloaded Optical Core Glow)
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(255, 145, 30); // Lighter, brilliant fiery orange
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(255, 160, 50);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, Color32::from_rgb(255, 210, 150)); // High-intensity heat rim
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, button_text_dark);

    // 3. Active State (Incinerator Purge / Maximum Brightness)
    visuals.widgets.active.bg_fill = Color32::from_rgb(255, 190, 80); // White-hot molten saffron glow
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(255, 190, 80);
    visuals.widgets.active.bg_stroke = Stroke::new(2.0, Color32::WHITE);
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::from_rgb(40, 15, 0));

    // 4. Selection State (Deep Incandescence Highlight)
    visuals.selection.bg_fill = Color32::from_rgb(140, 55, 0); // Deeply saturated orange text background
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(255, 160, 60));

    ctx.set_visuals(visuals);
}


fn set_windows_light(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();

    // --- Base Settings (Windows 11 Light High Contrast) ---
    visuals.dark_mode = false;
    visuals.override_text_color = Some(Color32::from_rgb(23, 23, 23)); // Erecutチャコールブラック (Segoe UI text)
    visuals.weak_text_color = Some(Color32::from_rgb(100, 100, 100)); // Secondary text gray
    visuals.weak_text_alpha = 0.85;
    visuals.disabled_alpha = 0.36;

    // --- Backgrounds & Panels (Mica Light & Acrylic Cards) ---
    visuals.panel_fill = Color32::from_rgb(243, 243, 243); // Main window canvas (Mica Light base)
    visuals.window_fill = Color32::from_rgb(255, 255, 255); // Flyouts / Dialogs: Pure white cards
    visuals.faint_bg_color = Color32::from_rgb(238, 238, 238); // List items / Hover item slots / Zebra rows
    visuals.extreme_bg_color = Color32::from_rgb(255, 255, 255); // Input text fields (White recessed)

    // --- Input & Code Fields ---
    visuals.text_edit_bg_color = Some(Color32::from_rgb(255, 255, 255));
    visuals.code_bg_color = Color32::from_rgb(245, 245, 245);

    // --- Accent & Semantic Colors (Windows Light System Accents) ---
    visuals.hyperlink_color = Color32::from_rgb(0, 90, 158); // Deep Windows Link Blue for light backgrounds
    visuals.warn_fg_color = Color32::from_rgb(159, 118, 0); // Warning: High-contrast Dark Amber
    visuals.error_fg_color = Color32::from_rgb(196, 43, 28); // Error: Windows Light Red Indicator

    // --- Geometry, Strokes & Borders (Windows 11 Standard Radii) ---
    visuals.window_corner_radius = CornerRadius::same(8); // Soft rounded window boundaries
    visuals.menu_corner_radius = CornerRadius::same(4);
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(218, 218, 218)); // Soft boundary outline
    visuals.window_highlight_topmost = true;

    // --- Gadgets & UI Behavior ---
    visuals.button_frame = true;
    visuals.collapsing_header_frame = false;
    visuals.indent_has_left_vline = true;
    visuals.striped = true;
    visuals.slider_trailing_fill = true; // Active blue track indicator
    visuals.image_loading_spinners = true;

    visuals.resize_corner_size = 8.0;
    visuals.clip_rect_margin = 2.0;
    visuals.interact_cursor = None;

    // --- Widget State Colors (Segoe Fluent Light Controls) ---
    let text_color = Color32::from_rgb(23, 23, 23);

    // 1. Inactive State (Default/Idle - Subtle Solid White/Gray Button)
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(254, 254, 254); 
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(249, 249, 249);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(204, 204, 204)); // Subtle bottom border reflection
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_color);

    // 2. Hovered State (Pointer-over Highlight)
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(245, 245, 245); // Light smoky tint
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(0, 120, 212); // Vivid Windows Accent Blue
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(166, 166, 166));
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::from_rgb(0, 0, 0));

    // 3. Active State (Pressed Toggle / Click State)
    visuals.widgets.active.bg_fill = Color32::from_rgb(235, 235, 235);
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(0, 90, 158); // Pressed Deep Accent Blue
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(140, 140, 140));
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::from_rgb(0, 0, 0));

    // 4. Selection State (Standard Windows Selection Highlighting)
    visuals.selection.bg_fill = Color32::from_rgb(0, 120, 212); // Standard Selection Blue
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(180, 220, 255));

    ctx.set_visuals(visuals);
}


fn set_windows_dark(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // --- Base Settings (Windows 11 Modern Contrast) ---
    visuals.dark_mode = true;
    visuals.override_text_color = Some(Color32::from_rgb(255, 255, 255)); // Pure White for high readability
    visuals.weak_text_color = Some(Color32::from_rgb(160, 160, 160)); // Segoe UI Muted Gray
    visuals.weak_text_alpha = 0.78;
    visuals.disabled_alpha = 0.36;

    // --- Backgrounds & Panels (Mica Material & Slate Backgrounds) ---
    visuals.panel_fill = Color32::from_rgb(32, 32, 32); // Main background: Windows 11 Mica base
    visuals.window_fill = Color32::from_rgb(44, 44, 44); // Flyout / Dialog Windows: layered card background
    visuals.faint_bg_color = Color32::from_rgb(40, 40, 40); // List view items / Zebra rows
    visuals.extreme_bg_color = Color32::from_rgb(26, 26, 26); // Text fields / Recessed elements

    // --- Input & Code Fields ---
    visuals.text_edit_bg_color = Some(Color32::from_rgb(26, 26, 26));
    visuals.code_bg_color = Color32::from_rgb(28, 28, 28);

    // --- Accent & Semantic Colors (Windows System Accents) ---
    visuals.hyperlink_color = Color32::from_rgb(96, 205, 255); // Windows Segoe Light Blue Link
    visuals.warn_fg_color = Color32::from_rgb(255, 185, 0); // Warning: Windows Safety Gold
    visuals.error_fg_color = Color32::from_rgb(255, 153, 164); // Error: Soft Critical Red

    // --- Geometry, Strokes & Borders (Windows 11 Standard Radii) ---
    visuals.window_corner_radius = CornerRadius::same(8); // Windows 11 standard rounded windows
    visuals.menu_corner_radius = CornerRadius::same(4); // Context menus & popups
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(60, 60, 60)); // Clean border outline
    visuals.window_highlight_topmost = true;

    // --- Gadgets & UI Behavior ---
    visuals.button_frame = true;
    visuals.collapsing_header_frame = false;
    visuals.indent_has_left_vline = true;
    visuals.striped = true;
    visuals.slider_trailing_fill = true; // Shows the blue accent track length
    visuals.image_loading_spinners = true;

    visuals.resize_corner_size = 8.0;
    visuals.clip_rect_margin = 2.0;
    visuals.interact_cursor = None;

    // --- Widget State Colors (Segoe Fluent Controls) ---
    let text_color = Color32::from_rgb(255, 255, 255);

    // 1. Inactive State (Default Command / Rest State)
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 45); // Standard Rest Button
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(38, 38, 38);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(55, 55, 55));
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_color);

    // 2. Hovered State (Fluent Pointer-over Highlight)
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(50, 50, 50); // Slightly lighter gray
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(0, 120, 212); // Windows Accent Blue (Rest/Hover)
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(110, 110, 110)); // Subtle border highlight
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);

    // 3. Active State (Pressed / Click Injection)
    visuals.widgets.active.bg_fill = Color32::from_rgb(60, 60, 60);
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(0, 90, 158); // Deep pressed Windows Accent Blue
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(140, 140, 140));
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);

    // 4. Selection State (Text Selection Highlight)
    visuals.selection.bg_fill = Color32::from_rgb(0, 120, 212); // Universal Windows Selection Blue
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(140, 200, 255));

    ctx.set_visuals(visuals);
}

fn set_sakura(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // --- Base Settings (Bright & Cute) ---
    visuals.dark_mode = true;
    visuals.override_text_color = Some(Color32::from_rgb(255, 240, 245)); // Lavender blush: pristine bright white-pink
    visuals.weak_text_color = Some(Color32::from_rgb(230, 175, 195)); // Bright rose pastel gray
    visuals.weak_text_alpha = 0.8; // Higher opacity for more brightness
    visuals.disabled_alpha = 0.5;

    // --- Backgrounds & Panels (Translucent Berry & Bright Rose Shadow) ---
    visuals.panel_fill = Color32::from_rgb(46, 32, 38); // Main panel: soft mauve rose
    visuals.window_fill = Color32::from_rgb(60, 42, 50); // Windows: medium frosted cherry
    visuals.faint_bg_color = Color32::from_rgb(76, 52, 64); // Stripe rows: light cotton candy gray
    visuals.extreme_bg_color = Color32::from_rgb(32, 20, 26); // Recessed fields: deep strawberry syrup

    // --- Input & Code Fields ---
    visuals.text_edit_bg_color = Some(Color32::from_rgb(32, 20, 26));
    visuals.code_bg_color = Color32::from_rgb(66, 44, 54);

    // --- Accent & Semantic Colors (High-Intensity Pastel Pink & Coral) ---
    visuals.hyperlink_color = Color32::from_rgb(255, 140, 200); // Super vibrant pastel pink
    visuals.warn_fg_color = Color32::from_rgb(255, 190, 100); // Warning: bright peach gold
    visuals.error_fg_color = Color32::from_rgb(255, 100, 130); // Error: bright soft crimson

    // --- Geometry, Strokes & Borders ---
    visuals.window_corner_radius = CornerRadius::same(8); // Softer, rounder edges for a cute aesthetic
    visuals.menu_corner_radius = CornerRadius::same(5);
    visuals.window_stroke = Stroke::new(1.5, Color32::from_rgb(130, 85, 105)); // Defined rose-quartz rim
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
    let text_color = Color32::from_rgb(255, 245, 250);

    // 1. Inactive State (Default/Idle - Creamy Berry Puff)
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(82, 55, 68); // Lighter pinkish-gray button base
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(68, 45, 56);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(150, 100, 125));
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_color);

    // 2. Hovered State (Bright Electric Bubblegum)
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(105, 70, 88);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(255, 105, 180); // Hot bubblegum pink backlight
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, Color32::from_rgb(255, 180, 220)); // Brilliant glowing border
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);

    // 3. Active State (Pop Neon Candy Flash)
    visuals.widgets.active.bg_fill = Color32::from_rgb(255, 60, 150);
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(255, 60, 150);
    visuals.widgets.active.bg_stroke = Stroke::new(1.5, Color32::from_rgb(255, 220, 240)); // High-gloss neon edge
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);

    // 4. Selection State (Bright Orchid Highlight)
    visuals.selection.bg_fill = Color32::from_rgb(160, 60, 110);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(255, 180, 220));

    ctx.set_visuals(visuals);
}

fn set_violet(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // --- Base Settings ---
    visuals.dark_mode = true;
    visuals.override_text_color = Some(Color32::from_rgb(230, 215, 255)); // Soft violet-tinted white
    visuals.weak_text_color = Some(Color32::from_rgb(145, 130, 165)); // Mystic slate purple
    visuals.weak_text_alpha = 0.6;
    visuals.disabled_alpha = 0.4;

    // --- Backgrounds & Panels (Deep Amethyst & Night Shade) ---
    visuals.panel_fill = Color32::from_rgb(18, 14, 26); // Main panel: deep space violet
    visuals.window_fill = Color32::from_rgb(26, 20, 36); // Windows: midnight amethyst
    visuals.faint_bg_color = Color32::from_rgb(36, 28, 50); // Stripe rows: subtle lavender gray
    visuals.extreme_bg_color = Color32::from_rgb(12, 8, 18); // Recessed fields: abyssal purple

    // --- Input & Code Fields ---
    visuals.text_edit_bg_color = Some(Color32::from_rgb(12, 8, 18));
    visuals.code_bg_color = Color32::from_rgb(24, 18, 32);

    // --- Accent & Semantic Colors (Neon Electric Violet) ---
    visuals.hyperlink_color = Color32::from_rgb(180, 100, 255); // Radiant neon violet
    visuals.warn_fg_color = Color32::from_rgb(240, 160, 40); // Warning: contrasting amber
    visuals.error_fg_color = Color32::from_rgb(255, 80, 160); // Error: hot electric magenta

    // --- Geometry, Strokes & Borders ---
    visuals.window_corner_radius = CornerRadius::same(6); // Elegant, smooth rounded corners
    visuals.menu_corner_radius = CornerRadius::same(4);
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(55, 42, 80)); // Velvet nebula border
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
    let text_color = Color32::from_rgb(245, 240, 255); // Crystal lavender text

    // 1. Inactive State (Default/Idle Muted Velvet)
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(38, 28, 54);
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(30, 22, 44);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(70, 50, 95));
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_color);

    // 2. Hovered State (Luminous Violet Glow)
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(52, 38, 74);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(130, 60, 220); // Vivid purple backlight
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(190, 120, 255)); // Glowing edge
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);

    // 3. Active State (Electric Purple Pulse)
    visuals.widgets.active.bg_fill = Color32::from_rgb(150, 70, 250);
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(150, 70, 250);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(220, 180, 255)); // Peak neon aura
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);

    // 4. Selection State (Nebula Backlight)
    visuals.selection.bg_fill = Color32::from_rgb(80, 40, 130);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(200, 150, 255));

    ctx.set_visuals(visuals);
}



fn set_chrome(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // --- Base Settings ---
    visuals.dark_mode = true;
    visuals.override_text_color = Some(Color32::from_rgb(220, 225, 230)); // Polished silver-white
    visuals.weak_text_color = Some(Color32::from_rgb(130, 135, 140)); // Muted steel gray
    visuals.weak_text_alpha = 0.6;
    visuals.disabled_alpha = 0.4;

    // --- Backgrounds & Panels (Deep Industrial Metallocenes) ---
    visuals.panel_fill = Color32::from_rgb(20, 22, 24); // Main panel: dark gunmetal
    visuals.window_fill = Color32::from_rgb(28, 30, 34); // Windows: heavy steel plates
    visuals.faint_bg_color = Color32::from_rgb(36, 40, 44); // Stripe rows: brushed metallic gray
    visuals.extreme_bg_color = Color32::from_rgb(12, 13, 15); // Recessed fields: deep cast iron

    // --- Input & Code Fields ---
    visuals.text_edit_bg_color = Some(Color32::from_rgb(12, 13, 15));
    visuals.code_bg_color = Color32::from_rgb(24, 26, 28);

    // --- Accent & Semantic Colors (High-contrast Machined Accents) ---
    visuals.hyperlink_color = Color32::from_rgb(100, 180, 255); // Laser electric blue
    visuals.warn_fg_color = Color32::from_rgb(230, 160, 40); // Warning: industrial amber
    visuals.error_fg_color = Color32::from_rgb(240, 70, 80); // Error: hot metallic crimson

    // --- Geometry, Strokes & Borders ---
    visuals.window_corner_radius = CornerRadius::same(4); // Sharper corners for a machined look
    visuals.menu_corner_radius = CornerRadius::same(3);
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(65, 70, 80)); // Chrome rim border
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
    let text_color = Color32::from_rgb(240, 242, 245); // Pure silver text

    // 1. Inactive State (Default/Idle Metallic Finish)
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(40, 44, 50); // Aluminum block feel
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(32, 36, 40);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(75, 80, 90)); // Beveled edge
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_color);

    // 2. Hovered State (Specular Highlight Reflection)
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(55, 60, 70); // Polished glow
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(80, 90, 100); 
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(200, 210, 225)); // Chrome spec reflectivity
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);

    // 3. Active State (Laser Indicator / Pressed Reflection)
    visuals.widgets.active.bg_fill = Color32::from_rgb(120, 130, 145);
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(140, 150, 165);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(240, 245, 255)); // Super-bright reflection
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);

    // 4. Selection State (Industrial Blue Backlight Highlight)
    visuals.selection.bg_fill = Color32::from_rgb(50, 75, 100);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(150, 200, 255));

    ctx.set_visuals(visuals);
}

fn set_light_blue(ctx: &egui::Context) {
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
        egui::FontData::from_static(include_bytes!("../assets/NotoSansJP-Regular.ttf")).into(),
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
