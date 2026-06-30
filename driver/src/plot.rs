use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::ImageEncoder;
use jiff::{ToSpan, Zoned};
use plotters::prelude::*; // Base64用のインポートを追加

/// データを積み上げ棒グラフとして解析し、Base64エンコードされたデータURI文字列を返す
pub fn export_to_base64(data: &[(i32, i32, i32, i32)]) -> Result<String> {
    // 1. ファイルではなく、メモリ上のVec<u8>バッファに直接書き込む準備
    let mut image_buffer = vec![0; 800 * 500 * 3]; // 幅800, 高さ500, RGB(3バイト)

    // スコープを分けることで、rootを確実にドロップ（present）させてバッファに書き込ませる
    {
        let root = BitMapBackend::with_buffer(&mut image_buffer, (800, 500)).into_drawing_area();
        root.fill(&RGBColor(245, 247, 250))?;

        if data.is_empty() {
            return Ok(String::new());
        }

        // Y軸の最大値を計算
        let max_y = data
            .iter()
            .map(|(p, w, c, ca)| p + w + c + ca)
            .max()
            .unwrap_or(10);

        let y_max_with_margin = (max_y as f32 * 1.15) as i32; // 凡例の被りを防ぐため15%の余裕

        // 2. グラフのレイアウト設定
        let mut chart = ChartBuilder::on(&root)
            .margin(30)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(0.0..data.len() as f32, 0..y_max_with_margin)?;

        chart
            .configure_mesh()
            .light_line_style(WHITE) // 補助線を消してすっきりと
            .y_desc("Number of Tasks")
            .x_desc("Date")
            .axis_desc_style(
                ("sans-serif", 14)
                    .into_font()
                    .color(&RGBColor(100, 110, 120)),
            )
            .axis_style(ShapeStyle::from(&RGBColor(180, 190, 200)).stroke_width(1))
            .label_style(("sans-serif", 12).into_font().color(&RGBColor(80, 90, 100)))
            // X軸の数値を日付文字列に変換
            .x_label_formatter(&|&idx_f32| {
                let idx = idx_f32 as usize;
                let start_date = Zoned::now().date();
                let days_to_subtract = (data.len() as i64 - 1) - idx as i64;
                if idx < data.len() {
                    let current_date = start_date - days_to_subtract.days();
                    current_date.strftime("%m/%d").to_string()
                } else {
                    "".to_string()
                }
            })
            .draw()?;

        // 3. 洗練されたカラーパレット
        let color_pending = RGBColor(140, 160, 180);
        let color_wip = RGBColor(246, 160, 84);
        let color_complete = RGBColor(78, 205, 151);
        let color_canceled = RGBColor(234, 110, 110);

        // 4. 積み上げグラフのダミーシリーズ登録（凡例用）
        let dummy_styles = [
            ("Pending", color_pending),
            ("WIP", color_wip),
            ("Complete", color_complete),
            ("Canceled", color_canceled),
        ];
        for (label, color) in dummy_styles {
            chart
                .draw_series(std::iter::once(EmptyElement::at((0.0, 0))))?
                .label(label)
                .legend(move |(x, y)| {
                    Rectangle::new([(x, y - 5), (x + 15, y + 5)], color.filled())
                });
        }

        // 5. 積み上げ棒グラフの描画
        let bar_margin = 0.15;

        for (i, d) in data.iter().rev().enumerate() {
            let mut current_y = 0;
            let x_start = i as f32 + bar_margin;
            let x_end = (i + 1) as f32 - bar_margin;

            let mut draw_bar = |val: i32, color: RGBColor| -> Result<()> {
                if val == 0 {
                    return Ok(());
                }
                let bottom = current_y;
                let top = current_y + val;
                current_y = top;

                chart.draw_series(std::iter::once(Rectangle::new(
                    [(x_start, bottom), (x_end, top)],
                    color.filled(),
                )))?;
                Ok(())
            };

            draw_bar(d.0, color_pending)?;
            draw_bar(d.1, color_wip)?;
            draw_bar(d.2, color_complete)?;
            draw_bar(d.3, color_canceled)?;
        }

        // 6. 凡例（Legend）のスタイリングと描画
        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::UpperLeft)
            .background_style(WHITE.mix(0.9))
            .border_style(RGBColor(210, 215, 220))
            .draw()?;

        root.present()?;
    } // ここで root がドロップされ、image_buffer にRGBの生データが確定する

    let mut png_buffer = std::io::Cursor::new(Vec::new());

    // 修正：new().encode(...) ではなく、PngEncoder::new(...) の後に
    // ImageEncoder トレイトのメソッド、または以下のように直接エンコード関数を呼び出す
    image::codecs::png::PngEncoder::new(&mut png_buffer).write_image(
        &image_buffer,
        800,
        500,
        image::ExtendedColorType::Rgb8,
    )?;

    // 8. PNGバイナリをBase64文字列にエンコードしてデータURIスキームを付与
    let base64_str = STANDARD.encode(png_buffer.into_inner());
    Ok(format!("data:image/png;base64,{}", base64_str))
}
