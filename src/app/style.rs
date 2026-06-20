use eframe::egui;

pub fn setup_style(ctx: &egui::Context) {
    ctx.set_pixels_per_point(1.5);
    setup_custom_fonts(ctx);
    ctx.global_style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.window_margin = egui::Margin::same(10);
    });
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // ✅ 修正点: from_static で読み込んだデータを直接挿入するだけでOK
    fonts.font_data.insert(
        "japanese_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../../asset/NotoSansJP-Regular.ttf")).into(),
    );

    // デフォルトのプロポーショナルフォント（通常のテキスト）に日本語を設定
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "japanese_font".to_owned());

    // デフォルトの等幅フォント（コード等）にも日本語を設定
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "japanese_font".to_owned());

    // 設定をコンテキストに反映
    ctx.set_fonts(fonts);
}
