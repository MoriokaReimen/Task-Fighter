use anyhow::Result as AnyhowResult;
use domain::{ColorScheme, Config, Locale};
use duckdb::Row;
use std::collections::HashMap;
use duckdb::{
    Result as DuckdbResult,
    types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
};
use std::str::FromStr;

pub struct DuckdbColorScheme(pub ColorScheme);
pub struct DuckdbLocale(pub Locale);

pub struct DuckdbConfig {
    pub color_scheme: DuckdbColorScheme,
    pub locale: DuckdbLocale,
}

impl From<Config> for DuckdbConfig {
    fn from(config: Config) -> Self {
        Self {
            color_scheme: DuckdbColorScheme(config.color_scheme),
            locale: DuckdbLocale(config.locale),
        }
    }
}

impl TryFrom<DuckdbConfig> for Config {
    type Error = anyhow::Error;

    fn try_from(duckdb_config: DuckdbConfig) -> AnyhowResult<Self> {
        Ok(Self {
            color_scheme: duckdb_config.color_scheme.0,
            locale: duckdb_config.locale.0,
        })
    }
}

impl ToSql for DuckdbColorScheme {
    fn to_sql(&self) -> DuckdbResult<ToSqlOutput<'_>> {
        let val_i32 = i32::from(self.0);
        Ok(ToSqlOutput::from(val_i32))
    }
}

impl FromSql for DuckdbColorScheme {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let val_i32 = i32::column_result(value)?;
        
        let color_scheme = ColorScheme::try_from(val_i32).map_err(|err| {
            FromSqlError::Other(
                format!("Retrieved undefined ColorScheme id from DuckDB: {val_i32} (Detail: {err})").into(),
            )
        })?;
        
        Ok(DuckdbColorScheme(color_scheme))
    }
}

// --- Locale ---
impl ToSql for DuckdbLocale {
    fn to_sql(&self) -> DuckdbResult<ToSqlOutput<'_>> {
        let val_i32 = i32::from(self.0);
        Ok(ToSqlOutput::from(val_i32))
    }
}

impl FromSql for DuckdbLocale {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let val_i32 = i32::column_result(value)?;
        
        let locale = Locale::try_from(val_i32).map_err(|err| {
            FromSqlError::Other(
                format!("Retrieved undefined Locale id from DuckDB: {val_i32} (Detail: {err})").into(),
            )
        })?;
        
        Ok(DuckdbLocale(locale))
    }
}

impl TryFrom<&Row<'_>> for DuckdbConfig {
    type Error = duckdb::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            color_scheme: row.get("color_scheme")?,
            locale: row.get("locale")?,
        })
    }
}

impl DuckdbConfig {
    pub fn to_named_params(&self) -> HashMap<&str, &dyn ToSql> {
        HashMap::from_iter([
            ("color_scheme", &self.color_scheme as &dyn ToSql),
            ("locale", &self.locale as &dyn ToSql),
        ])
    }
}
