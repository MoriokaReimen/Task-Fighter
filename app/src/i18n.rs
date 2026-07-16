use anyhow::{Context, Result};
use fluent_templates::static_loader;
use std::sync::{OnceLock, RwLock};
use unic_langid::{LanguageIdentifier, langid};

static_loader! {
    // マクロから直接呼べるように pub にしておきます
    pub static LOCALES = {
        locales: "./i18n/",
        fallback_language: "en",
    };
}

/// ロケール管理をカプセル化する構造体
pub struct I18n {
    current_locale: RwLock<Option<LanguageIdentifier>>,
}

impl I18n {
    /// 構造体の新しいインスタンスを作成（内部用）
    const fn new() -> Self {
        Self {
            current_locale: RwLock::new(None),
        }
    }

    /// グローバルで共有される唯一の I18n インスタンスへの参照を取得します
    pub fn global() -> &'static Self {
        static INSTANCE: OnceLock<I18n> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    /// 必要に応じてシステムロケールで初期化し、現在のロケールを返します
    pub fn init(&self) -> LanguageIdentifier {
        if let Ok(guard) = self.current_locale.read() {
            if let Some(ref lang) = *guard {
                return lang.clone();
            }
        }

        let mut guard = self.current_locale.write().unwrap();
        if guard.is_none() {
            let detected = sys_locale::get_locale()
                .and_then(|s| s.parse::<LanguageIdentifier>().ok())
                .unwrap_or_else(|| langid!("en"));
            *guard = Some(detected);
        }

        guard.as_ref().unwrap().clone()
    }

    /// 現在のロケールを取得します。未初期化やロックエラー時は Err を返します
    pub fn get_locale(&self) -> Result<LanguageIdentifier> {
        let guard = self
            .current_locale
            .read()
            .map_err(|_| anyhow::anyhow!("I18n lock is poisoned"))?;

        let lang = guard.as_ref().context("Locale is not initialized yet")?;

        Ok(lang.clone())
    }

    /// 任意のロケールに動的に切り替えます
    pub fn set_locale(&self, lang: LanguageIdentifier) {
        if let Ok(mut guard) = self.current_locale.write() {
            *guard = Some(lang);
        }
    }
}

/// アプリケーションのどこからでも呼び出せる簡易マクロ
#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        use fluent_templates::Loader;
        $crate::i18n::LOCALES.lookup(&$crate::i18n::I18n::global().init(), $message_id)
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

        $crate::i18n::LOCALES.lookup_with_args(&$crate::i18n::I18n::global().init(), $message_id, &args)
    }};
}
