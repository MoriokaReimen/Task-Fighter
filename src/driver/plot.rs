use anyhow::Result; // UnusedのContextを削除
use jiff::{ToSpan, Zoned};
use plotters::prelude::*;
use std::path::Path;

/// データを積み上げ棒グラフとして解析し、指定されたパスにPNGとして保存する
pub fn export_to_png<P: AsRef<Path>>(output_path: P, data: &[(i32, i32, i32, i32)]) -> Result<()> {
    // 1. 全体の背景を少し落ち着いた薄いグレーに
    let root = BitMapBackend::new(&output_path, (800, 500)).into_drawing_area();
    root.fill(&RGBColor(245, 247, 250))?;

    if data.is_empty() {
        return Ok(());
    }

    // Y軸の最大値を計算
    let max_y = data
        .iter()
        .map(|(p, w, c, ca)| p + w + c + ca)
        .max()
        .unwrap_or(10);

    let y_max_with_margin = (max_y as f32 * 1.15) as i32; // 凡例の被りを防ぐため15%の余裕

    // 2. グラフのレイアウト設定（X軸を f32 の範囲にすることで隙間計算を可能にする）
    let mut chart = ChartBuilder::on(&root)
        .margin(30)
        .x_label_area_size(40)
        .y_label_area_size(50)
        // build_ranged から build_cartesian_2d に変更し、X軸を f32 にキャスト
        .build_cartesian_2d(0.0..data.len() as f32, 0..y_max_with_margin)?;

    // グリッド線のスタイリッシュ化（点線の作成方法を修正）
    let _grid_style = ShapeStyle {
        color: RGBAColor(220, 225, 230, 1.0),
        filled: false,
        stroke_width: 1,
    };

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
        .axis_style(ShapeStyle::from(&RGBColor(180, 190, 200)).stroke_width(1)) // 修正: stroke_width
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

    // 3. 洗練されたカラーパレット（モダン・フラットデザイン）
    let color_pending = RGBColor(140, 160, 180); // 落ち着いたグレーブルー
    let color_wip = RGBColor(246, 160, 84); // 柔らかなオレンジ
    let color_complete = RGBColor(78, 205, 151); // さわやかなミントグリーン
    let color_canceled = RGBColor(234, 110, 110); // 優しい赤・コーラル

    // 4. 積み上げグラフのダミーシリーズ登録（凡例用）
    let dummy_styles = [
        ("Pending", color_pending),
        ("WIP", color_wip),
        ("Complete", color_complete),
        ("Canceled", color_canceled),
    ];
    for (label, color) in dummy_styles {
        // ダミー座標の型を (f32, i32) に統一
        chart
            .draw_series(std::iter::once(EmptyElement::at((0.0, 0))))?
            .label(label)
            .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 15, y + 5)], color.filled()));
    }

    // 5. 積み上げ棒グラフの描画
    let bar_margin = 0.15; // 棒と棒の間の隙間

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

            // X軸(f32) と Y軸(i32) の組み合わせで矩形を描画
            chart.draw_series(std::iter::once(Rectangle::new(
                [(x_start, bottom), (x_end, top)],
                color.filled(),
            )))?;
            Ok(())
        };

        // 積み上げ順: Pending -> WIP -> Complete -> Canceled
        draw_bar(d.0, color_pending)?;
        draw_bar(d.1, color_wip)?;
        draw_bar(d.2, color_complete)?;
        draw_bar(d.3, color_canceled)?;
    }

    // 6. 凡例（Legend）のスタイリングと描画
    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft) // 右上に配置
        .background_style(WHITE.mix(0.9)) // やや透過した白背景
        .border_style(RGBColor(210, 215, 220)) // 薄い境界線
        .draw()?;

    // バッファをファイルに書き込み
    root.present()?;
    Ok(())
}
