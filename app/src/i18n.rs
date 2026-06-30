use fluent_templates::static_loader; // ← これがマクロ内部で明示的に必要になります
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

pub fn init_i18n() -> &'static LanguageIdentifier {
    CURRENT_LOCALE.get_or_init(|| {
        let locale_str = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
        locale_str
            .parse::<LanguageIdentifier>()
            .unwrap_or_else(|_| "en".parse().unwrap())
    })
}

// 3. 完全自己完結型に修正した fl! マクロ
#[macro_export]
macro_rules! fl {
    // 引数なしの場合
    ($message_id:literal) => {{
        use fluent_templates::Loader; // マクロ展開先でトレイトをスコープに入れる
        let lang = $crate::i18n::init_i18n();
        $crate::i18n::LOCALES.lookup(lang, $message_id)
    }};

    // 引数（変数埋め込み）がある場合
    ($message_id:literal, $($key:expr => $value:expr),* $(,)?) => {{
        use fluent_templates::Loader; // マクロ展開先でトレイトをスコープに入れる
        let lang = $crate::i18n::init_i18n();
        let mut args = std::collections::HashMap::new();
        $(
            args.insert($key, fluent_templates::fluent_bundle::FluentValue::from($value));
        )*
        $crate::i18n::LOCALES.lookup_with_args(lang, $message_id, &args)
    }};
}
