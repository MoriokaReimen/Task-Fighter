use anyhow::{Context, Result};
use domain::Locale;
use fluent_templates::static_loader;
use std::sync::{OnceLock, RwLock};
use unic_langid::{LanguageIdentifier, langid};

static_loader! {
    pub static LOCALES = {
        locales: "./i18n/",
        fallback_language: "en",
    };
}

pub struct I18n {
    current_locale: RwLock<Option<LanguageIdentifier>>,
}

impl I18n {
    const fn new() -> Self {
        Self {
            current_locale: RwLock::new(None),
        }
    }

    pub fn global() -> &'static Self {
        static INSTANCE: OnceLock<I18n> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    pub fn init(&self) -> LanguageIdentifier {
        if let Some(lang) = self.current_locale.read().ok().and_then(|g| g.clone()) {
            return lang;
        }

        let mut guard = self
            .current_locale
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.get_or_insert_with(Self::detect_system_locale).clone()
    }

    pub fn get_locale(&self) -> Result<LanguageIdentifier> {
        let guard = self
            .current_locale
            .read()
            .map_err(|_| anyhow::anyhow!("I18n lock is poisoned"))?;
        guard.clone().context("Locale is not initialized yet")
    }

    pub fn set_locale(&self, lang: LanguageIdentifier) {
        if let Ok(mut guard) = self.current_locale.write() {
            *guard = Some(lang);
        }
    }

    pub fn set_locale_from_config(&self, locale: Locale) {
        let lang = match locale {
            Locale::System => Self::detect_system_locale(),
            Locale::English => langid!("en"),
            Locale::Japanese => langid!("ja"),
            Locale::German => langid!("de"),
            Locale::Chinese => langid!("zh"),
            Locale::Vietnamese => langid!("vi"),
            Locale::Spanish => langid!("es"),
        };
        self.set_locale(lang);
    }

    fn detect_system_locale() -> LanguageIdentifier {
        sys_locale::get_locale()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| langid!("en"))
    }
}

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        use fluent_templates::Loader;
        $crate::i18n::LOCALES.lookup(&$crate::i18n::I18n::global().init(), $message_id)
    }};
    ($message_id:literal, $($key:ident = $value:expr),+ $(,)?) => {{
        use fluent_templates::Loader;
        let args: std::collections::HashMap<&str, fluent_templates::fluent_bundle::FluentValue> =
            [$((stringify!($key), fluent_templates::fluent_bundle::FluentValue::from($value))),+]
                .into_iter()
                .collect();
        $crate::i18n::LOCALES.lookup_with_args(&$crate::i18n::I18n::global().init(), $message_id, &args)
    }};
}
