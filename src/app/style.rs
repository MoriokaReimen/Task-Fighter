use eframe::egui;
use egui::{Color32, CornerRadius, Stroke};

pub fn setup_style(ctx: &egui::Context) {
    /* Tweak font and size */
    ctx.set_pixels_per_point(1.5);
    setup_custom_fonts(ctx);
    /* Tweak spacing */
    ctx.global_style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.window_margin = egui::Margin::same(10);
    });

    /* Tweak Color */
    ctx.set_theme(egui::Theme::Dark);
    let visuals = egui::Visuals::light();
    ctx.set_visuals(visuals);
    let visuals = egui::Visuals::dark();
    ctx.set_visuals(visuals);
}

fn get_visuals(visuals: &mut egui::Visuals) {
    // --- 基本設定 ---
    visuals.dark_mode = true;
    // 水色に馴染む、少し青みがかったクリーンな白
    visuals.override_text_color = Some(Color32::from_rgb(0, 192, 220));
    // 控えめな水色グレー
    visuals.weak_text_color = Some(Color32::from_rgb(130, 150, 160));
    visuals.weak_text_alpha = 0.6;
    visuals.disabled_alpha = 0.5;

    // --- 背景・ウィンドウ設定 ---
    // メインパネル：深海のようなダークアクアネイビー
    visuals.panel_fill = Color32::from_rgb(16, 24, 30);
    // ウィンドウ：少し明るいミッドナイトブルー
    visuals.window_fill = Color32::from_rgb(24, 34, 42);
    // ストライプや薄い背景：透明感のある水色グレー
    visuals.faint_bg_color = Color32::from_rgb(30, 44, 54);
    // テキストエリアなどのくぼんだ背景：引き締まった濃紺
    visuals.extreme_bg_color = Color32::from_rgb(10, 16, 20);

    // --- テキスト・コード設定 ---
    visuals.text_edit_bg_color = Some(Color32::from_rgb(10, 16, 20));
    visuals.code_bg_color = Color32::from_rgb(20, 30, 38);

    // --- アクセントカラー ---
    visuals.hyperlink_color = Color32::from_rgb(0, 210, 255); // 明るいクリアな水色
    visuals.warn_fg_color = Color32::from_rgb(240, 180, 50); // 警告：少し寒色に合うゴールド
    visuals.error_fg_color = Color32::from_rgb(255, 90, 120); // エラー：コーラルピンク

    // --- 形状・枠線・影 ---
    visuals.window_corner_radius = CornerRadius::same(6);
    visuals.menu_corner_radius = CornerRadius::same(4);
    // 境界線：氷のような薄い青
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(40, 60, 75));
    visuals.window_highlight_topmost = true;

    // --- ガジェット・UIの振る舞い ---
    visuals.button_frame = true;
    visuals.collapsing_header_frame = false;
    visuals.indent_has_left_vline = true;
    visuals.striped = true;
    visuals.slider_trailing_fill = true;
    visuals.image_loading_spinners = true;

    visuals.resize_corner_size = 10.0;
    visuals.clip_rect_margin = 3.0;
    visuals.interact_cursor = None;

    // --- コンポーネントの状態別カラー (widgets) ---
    let text_color = Color32::from_rgb(225, 240, 245);

    // 1. 通常時 (Inactive) - 落ち着いた水色グレー
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(32, 46, 56);
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(26, 38, 48);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(50, 72, 88));
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_color);

    // 2. ホバー時 (Hovered) - 美しく発光するサイアンブルー
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(42, 64, 80);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(0, 140, 180);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0, 210, 255));
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);

    // 3. クリック・アクティブ時 (Active) - 鮮やかなネオン水色
    visuals.widgets.active.bg_fill = Color32::from_rgb(0, 170, 220);
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(0, 170, 220);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(140, 240, 255));
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);

    // 4. 選択状態 (Selection) - テキストハイライトなど
    visuals.selection.bg_fill = Color32::from_rgb(0, 100, 140);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(160, 235, 255));
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
