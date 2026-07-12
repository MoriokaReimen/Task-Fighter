use fluent_templates::static_loader;
use std::sync::OnceLock;
use unic_langid::{LanguageIdentifier, langid};

static_loader! {
    pub static LOCALES = {
        locales: "./i18n/",
        fallback_language: "en",
    };
}

pub static CURRENT_LOCALE: OnceLock<LanguageIdentifier> = OnceLock::new();

pub fn init_i18n() -> &'static LanguageIdentifier {
    CURRENT_LOCALE.get_or_init(|| {
        sys_locale::get_locale()
            .and_then(|s| s.parse::<LanguageIdentifier>().ok())
            .unwrap_or_else(|| langid!("en"))
    })
}

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        use fluent_templates::Loader;
        $crate::i18n::LOCALES.lookup($crate::i18n::init_i18n(), $message_id)
    }};

    ($message_id:literal, $($key:ident = $value:expr),* $(,)?) => {{
        use fluent_templates::Loader;

        let count = [$( { let _ = stringify!($key); 1 } ),*].len();
        let mut args = std::collections::HashMap::with_capacity(count);

        $(
            let k = stringify!($key);
            let v = fluent_templates::fluent_bundle::FluentValue::from($value);
            args.insert(k, v);
        )*

        $crate::i18n::LOCALES.lookup_with_args($crate::i18n::init_i18n(), $message_id, &args)
    }};
}
