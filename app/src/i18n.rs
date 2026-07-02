use fluent_templates::static_loader;
use std::sync::OnceLock;
use unic_langid::LanguageIdentifier;

// 1. 翻訳ファイルの埋め込み
static_loader! {
    pub static LOCALES = {
        locales: "./i18n/",
        fallback_language: "en",
    };
}

// 2. 現在のOSの言語を保持するシングルトン
pub static CURRENT_LOCALE: OnceLock<LanguageIdentifier> = OnceLock::new();

/// OSのロケールを取得し、初期化する
pub fn init_i18n() -> &'static LanguageIdentifier {
    CURRENT_LOCALE.get_or_init(|| {
        sys_locale::get_locale()
            .and_then(|s| s.parse::<LanguageIdentifier>().ok())
            .unwrap_or_else(|| "en".parse().expect("Valid fallback language identifier"))
    })
}

// 3. 高速かつ簡潔に修正した fl! マクロ
#[macro_export]
macro_rules! fl {
    // 引数なしの場合
    ($message_id:literal) => {{
        use fluent_templates::Loader;
        $crate::i18n::LOCALES.lookup($crate::i18n::init_i18n(), $message_id)
    }};

    // 引数（変数埋め込み）がある場合
    ($message_id:literal, $($key:expr => $value:expr),* $(,)?) => {{
        use fluent_templates::Loader;

        // 繰り返し数（引数の個数）をコンパイル時に数えて HashMap の容量をあらかじめ確保
        let mut args = std::collections::HashMap::with_capacity([$({ let _ = &$key; 1 }),*].len());
        $(
            args.insert($key, fluent_templates::fluent_bundle::FluentValue::from($value));
        )*

        $crate::i18n::LOCALES.lookup_with_args($crate::i18n::init_i18n(), $message_id, &args)
    }};
}
