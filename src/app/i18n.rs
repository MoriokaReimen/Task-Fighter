use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
    LanguageRequester, // ← これを追加（requested_languages() を呼ぶために必要）
};
use lazy_static::lazy_static;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

lazy_static! {
    pub static ref STATIC_LOADER: FluentLanguageLoader = {
        let loader: FluentLanguageLoader = fluent_language_loader!();
        loader.set_use_isolating(false);
        // DesktopLanguageRequester をインスタンス化して要求された言語を取得
        let requester = DesktopLanguageRequester::new();
        let requested_languages = requester.requested_languages();
        
        // 言語の選択と読み込みを実行
        i18n_embed::select(&loader, &Localizations, &requested_languages).unwrap();
        
        loader
    };
}

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::app::i18n::STATIC_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::app::i18n::STATIC_LOADER, $message_id, $($args), *)
    }};
}
