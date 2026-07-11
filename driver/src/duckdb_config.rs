use anyhow::Result as AnyhowResult;
use domain::{ColorScheme, Config};
use duckdb::{
    Result as DuckdbResult,
    types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
};

// =========================================================================
// 1. DuckdbConfig structure and type conversion implementations
// =========================================================================
pub struct DuckdbConfig {
    color_scheme: ColorScheme,
}

impl From<Config> for DuckdbConfig {
    fn from(config: Config) -> Self {
        Self {
            color_scheme: config.color_scheme,
        }
    }
}

impl TryFrom<DuckdbConfig> for Config {
    type Error = anyhow::Error;

    fn try_from(duckdb_config: DuckdbConfig) -> AnyhowResult<Self> {
        Ok(Self {
            color_scheme: duckdb_config.color_scheme,
        })
    }
}

// =========================================================================
// 2. Private helper functions for ColorScheme mapping (avoids orphan rules)
// =========================================================================
const fn color_scheme_to_str(scheme: ColorScheme) -> &'static str {
    match scheme {
        ColorScheme::LightBlue => "LightBlue",
        // Add other variants here as they are introduced
    }
}

fn color_scheme_from_str(s: &str) -> Option<ColorScheme> {
    match s {
        "LightBlue" => Some(ColorScheme::LightBlue),
        _ => None,
    }
}

// =========================================================================
// 3. duckdb crate trait implementations for Native ENUM mapping
// =========================================================================

// Rust -> DuckDB (Converts internal enum variant into an ENUM string for database insertion)
impl ToSql for DuckdbConfig {
    fn to_sql(&self) -> DuckdbResult<ToSqlOutput<'_>> {
        let val_str = color_scheme_to_str(self.color_scheme);
        Ok(ToSqlOutput::from(val_str.to_string()))
    }
}

// DuckDB -> Rust (Extracts the ENUM string from the database and rebuilds the struct)
impl FromSql for DuckdbConfig {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        // Native DuckDB ENUM types can be safely extracted as a string via ValueRef
        let s = value.as_str()?;

        let color_scheme = color_scheme_from_str(s).ok_or_else(|| {
            FromSqlError::Other(format!("Retrieved undefined ColorScheme from DuckDB: {s}").into())
        })?;

        Ok(Self { color_scheme })
    }
}
