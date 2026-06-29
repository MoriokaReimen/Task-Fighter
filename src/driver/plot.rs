use jiff::{ToSpan, Zoned};
use plotters::prelude::*;
use std::path::Path;

/// データを積み上げ棒グラフとして解析し、指定されたパスにPNGとして保存する
pub fn export_to_png<P: AsRef<Path>>(
    output_path: P,
    data: &[(i32, i32, i32, i32)],
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. 画像の出力先とサイズ（幅800px, 高さ500px）を指定
    let root = BitMapBackend::new(&output_path, (800, 500)).into_drawing_area();
    root.fill(&WHITE)?;

    if data.is_empty() {
        return Ok(());
    }

    // Y軸の最大値を計算（積み上げた合計の最大値）
    let max_y = data
        .iter()
        .map(|(p, w, c, ca)| p + w + c + ca)
        .max()
        .unwrap_or(10) as i32;
    
    // Y軸の上限に10%の余裕を持たせる
    let y_max_with_margin = (max_y as f32 * 1.1) as i32;

    // 2. グラフのレイアウト（マージンや軸、タイトルの設定）
    let mut chart = ChartBuilder::on(&root)
        .caption("Task Status History", ("sans-serif", 30).into_font())
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        // X軸はインデックス（0〜データ数）、Y軸はタスク数（0〜最大値）
        .build_ranged(0..data.len(), 0..y_max_with_margin)?;

    // メッシュ（グリッド線）と軸の設定
    chart
        .configure_mesh()
        .y_desc("Number of Tasks")
        .x_desc("Date")
        // X軸の数値を日付文字列に変換する
        .x_label_formatter(&|&idx| {
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

    // 3. カラーパレットの定義（RGB）
    let color_canceled = RGBColor(180, 0, 0);
    let color_complete = RGBColor(46, 204, 113);
    let color_wip = RGBColor(230, 126, 34);
    let color_pending = RGBColor(52, 152, 219);

    // 4. 積み上げ棒グラフの描画 (古い順 `.rev()` にループ処理)
    for (i, d) in data.iter().rev().enumerate() {
        let mut current_y = 0;

        // 下から順に長方形（Rectangle）を積み上げていくヘルパーマクロ/クロージャ
        let mut draw_bar = |val: i32, color: RGBColor| -> Result<(), Box<dyn std::error::Error>> {
            if val == 0 { return Ok(()); }
            let bottom = current_y;
            let top = current_y + val;
            current_y = top;

            // 棒の幅（左右の太さ）を設定して矩形を描画
            chart.draw_series(std::iter::once(Rectangle::new(
                [(i, bottom), (i + 1, top)],
                color.filled(),
            )))?;
            Ok(())
        };

        // 積み上げ順: Canceled -> Complete -> WIP -> Pending
        draw_bar(d.3, color_canceled)?;
        draw_bar(d.2, color_complete)?;
        draw_bar(d.1, color_wip)?;
        draw_bar(d.0, color_pending)?;
    }

    // 5. 凡例（Legend）の追加
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    // バッファをファイルに書き込み
    root.present()?;
    Ok(())
}
