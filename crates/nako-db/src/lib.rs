mod automation_proposals;
mod backend;
mod facade;
mod postgres;
mod sqlite;

pub use backend::{DatabaseBackendKind, DatabaseConnectOptions};
pub use facade::{DatabaseBackendCapabilities, NakoDatabase};
pub use sqlite::SqliteRuntimeOptions;

#[cfg(test)]
mod contract_tests;

#[cfg(test)]
mod search_tests;

#[cfg(test)]
mod tests;
