use crate::duckdb_config::DuckdbConfig;
use anyhow::Result;
use domain::Config;
use duckdb::Connection;

pub fn save_config(conn: &Connection, config: Config) -> Result<()> {
    // Convert your clean domain config into the DB wrapper data layout
    let db_config = DuckdbConfig::from(config);
    let mut stmt = conn.prepare_cached(
        "INSERT OR REPLACE INTO config (id, color_scheme, locale, email_locale) VALUES (1, $color_scheme, $locale, $email_locale);",
    )?;
    let _ = stmt.query(&db_config.to_named_params())?;

    Ok(())
}

pub fn load_config(conn: &Connection) -> Result<Config> {
    let mut stmt =
        conn.prepare_cached("SELECT color_scheme, locale, email_locale FROM config WHERE id = 1;")?;
    let mut rows = stmt.query([])?;

    if let Some(row) = rows.next()? {
        let config = DuckdbConfig::try_from(row)?;
        let config = Config::try_from(config)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DuckdbPath;
    use crate::connect;
    use domain::*;
    use duckdb::Connection;

    // テスト用のDB初期化ヘルパー関数
    fn setup_in_memory_db() -> Connection {
        let path = DuckdbPath::InMemory;
        let conn = connect(&path);

        conn.unwrap()
    }

    #[test]
    fn test_save_and_load_config() -> Result<()> {
        const COLORS: [ColorScheme; 8] = [
            ColorScheme::LightBlue,
            ColorScheme::LightBlue,
            ColorScheme::DarkOrange,
            ColorScheme::WindowsLight,
            ColorScheme::WindowsDark,
            ColorScheme::Sakura,
            ColorScheme::Violet,
            ColorScheme::Chrome,
        ];

        let conn = setup_in_memory_db();
        let ret = COLORS
            .iter()
            .map(|color| {
                let mut config = domain::Config::default();
                config.color_scheme = *color;
                let ret = save_config(&conn, config);
                assert!(ret.is_ok());
                let ret = load_config(&conn)?;
                assert!(ret.color_scheme == *color);
                Ok(())
            })
            .collect::<Result<Vec<_>>>();
        assert!(ret.is_ok());

        Ok(())
    }
}
