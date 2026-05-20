use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use taru_core::{Result, TaruError};

use crate::sqlite::SqliteRuntimeOptions;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseBackendKind {
    #[default]
    Sqlite,
    Postgres,
}

impl DatabaseBackendKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
        }
    }
}

impl fmt::Display for DatabaseBackendKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for DatabaseBackendKind {
    type Err = TaruError;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "sqlite" => Ok(Self::Sqlite),
            "postgres" | "postgresql" => Ok(Self::Postgres),
            other => Err(TaruError::InvalidInput {
                message: format!(
                    "unsupported database backend '{other}'; expected 'sqlite' or 'postgres'"
                ),
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatabaseConnectOptions {
    pub backend: DatabaseBackendKind,
    pub url: String,
    pub sqlite_runtime: Option<SqliteRuntimeOptions>,
}

impl DatabaseConnectOptions {
    #[must_use]
    pub fn sqlite(database_url: impl Into<String>) -> Self {
        Self {
            backend: DatabaseBackendKind::Sqlite,
            url: database_url.into(),
            sqlite_runtime: None,
        }
    }

    #[must_use]
    pub fn sqlite_with_runtime(
        database_url: impl Into<String>,
        runtime: SqliteRuntimeOptions,
    ) -> Self {
        Self {
            backend: DatabaseBackendKind::Sqlite,
            url: database_url.into(),
            sqlite_runtime: Some(runtime),
        }
    }

    #[must_use]
    pub fn postgres(database_url: impl Into<String>) -> Self {
        Self {
            backend: DatabaseBackendKind::Postgres,
            url: database_url.into(),
            sqlite_runtime: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_kind_parses_public_config_values() {
        assert_eq!("sqlite".parse(), Ok(DatabaseBackendKind::Sqlite));
        assert_eq!("postgres".parse(), Ok(DatabaseBackendKind::Postgres));
        assert_eq!("postgresql".parse(), Ok(DatabaseBackendKind::Postgres));
    }

    #[test]
    fn backend_kind_rejects_implicit_or_unknown_values() {
        let err = "sqlite://taru.db"
            .parse::<DatabaseBackendKind>()
            .unwrap_err();

        assert!(matches!(err, TaruError::InvalidInput { .. }));
        assert!(err.to_string().contains("unsupported database backend"));
    }

    #[test]
    fn sqlite_connect_options_can_carry_runtime_policy() {
        let options = DatabaseConnectOptions::sqlite_with_runtime(
            "sqlite::memory:",
            SqliteRuntimeOptions::in_memory(),
        );

        assert_eq!(options.backend, DatabaseBackendKind::Sqlite);
        assert_eq!(options.url, "sqlite::memory:");
        assert_eq!(
            options.sqlite_runtime,
            Some(SqliteRuntimeOptions::in_memory())
        );
    }
}
