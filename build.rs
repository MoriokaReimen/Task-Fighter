use std::env;
use std::path::Path;

fn main() {
    /* Set icon for windows build */
    if cfg!(target_os = "windows") {
        winres::WindowsResource::new()
            .set_icon("./asset/icon.ico")
            .compile()
            .unwrap();
    }

    // コピー元のディレクトリ（例: プロジェクトルートの "assets"）
    let source_dir = "runtime";

    // Cargoが提供するビルド出力ディレクトリを取得
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_dir = Path::new(&out_dir).join("../../../"); // バイナリと同じ階層へ

    // ディレクトリのコピー設定
    let mut options = fs_extra::dir::CopyOptions::new();
    options.overwrite = true; // 既存のファイルを上書き
    options.copy_inside = true; // ディレクトリそのものをコピー（assets/の中身ではなくassetsフォルダごと）

    // コピー実行
    if let Err(e) = fs_extra::dir::copy(source_dir, &dest_dir, &options) {
        println!("cargo:warning=Failed to copy directory: {}", e);
    }

    // ソースディレクトリが変更されたら再ビルドするようにCargoに伝える
    println!("cargo:rerun-if-changed={}", source_dir);
}
