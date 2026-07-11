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

impl ToSql for DuckdbConfig {
    fn to_sql(&self) -> DuckdbResult<ToSqlOutput<'_>> {
        // ColorScheme -> i32
        let val_i32 = i32::from(self.color_scheme);

        // DuckDBの整数型へマッピングして出力
        Ok(ToSqlOutput::from(val_i32))
    }
}

impl FromSql for DuckdbConfig {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        // 修正：ValueRef を i32 自体の FromSql 実装に委ねて安全にパースします。
        // これにより、DuckDB 内部の整数表現（TinyInt, BigIntなど）の差異を自動で吸収してくれます。
        let val_i32 = i32::column_result(value)?;

        // i32 から ColorScheme へ変換
        let color_scheme = ColorScheme::try_from(val_i32).map_err(|err| {
            FromSqlError::Other(
                format!(
                    "Retrieved undefined ColorScheme id from DuckDB: {val_i32} (Detail: {err})"
                )
                .into(),
            )
        })?;

        Ok(Self { color_scheme })
    }
}
