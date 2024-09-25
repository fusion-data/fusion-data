use std::borrow::Cow;

use sqlx::error::DatabaseError;
use thiserror::Error;
use ultimate::DataError;

use crate::Id;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Unauthorized")]
  Unauthorized,

  #[error("Invalid argment, error message: {message}")]
  InvalidArgument { message: String },

  #[error("Entity not found. entity: '{schema}.{entity}', id: {id}")]
  EntityNotFound { schema: &'static str, entity: &'static str, id: Id },

  #[error("Data not found. table is '{schema}.{table}'")]
  NotFound { schema: &'static str, table: &'static str, sql: String },

  #[error("List limit over max. max: {max}, actual: {actual}")]
  ListLimitOverMax { max: i64, actual: i64 },

  #[error("Count fail")]
  CountFail,

  // -- DB
  #[error("User already exists. {key}: '{value}'")]
  UserAlreadyExists { key: &'static str, value: String },

  #[error("Unique violation. table: '{table}', constraint: {constraint}")]
  UniqueViolation { table: String, constraint: String },

  // -- ModelManager
  #[error("Can't create ModelManagerProvider. provider: {0}")]
  CantCreateModelManagerProvider(String),

  // -- Modules
  #[error(transparent)]
  SecurityError(#[from] ultimate::security::Error),

  #[error(transparent)]
  DbxError(#[from] crate::store::dbx::Error),

  // -- Externals
  #[error(transparent)]
  SeaQueryError(#[from] sea_query::error::Error),

  #[error(transparent)]
  ModqlIntoSea(#[from] modql::filter::IntoSeaError),

  #[error(transparent)]
  JsonError(#[from] serde_json::Error),
}

impl Error {
  /// This function will transform the error into a more precise variant if it is an SQLX or PGError Unique Violation.
  /// The resolver can contain a function (table_name: &str, constraint: &str) that may return a specific Error if desired.
  /// If the resolver is None, or if the resolver function returns None, it will default to Error::UniqueViolation {table, constraint}.
  pub fn resolve_unique_violation<F>(self, resolver: Option<F>) -> Self
  where
    F: FnOnce(&str, &str) -> Option<Self>,
  {
    match self.as_database_error().map(|db_error| (db_error.code(), db_error.table(), db_error.constraint())) {
      // "23505" => postgresql "unique violation"
      Some((Some(Cow::Borrowed("23505")), Some(table), Some(constraint))) => resolver
        .and_then(|fun| fun(table, constraint))
        .unwrap_or_else(|| Error::UniqueViolation { table: table.to_string(), constraint: constraint.to_string() }),
      _ => self,
    }
  }

  /// A convenient function to return the eventual database error (Postgres)
  /// if this Error is an SQLX Error that contains a database error.
  pub fn as_database_error(&self) -> Option<&(dyn DatabaseError + 'static)> {
    match self {
      Error::DbxError(crate::store::dbx::Error::Sqlx(sqlx_error)) => sqlx_error.as_database_error(),
      _ => None,
    }
  }
}

impl From<Error> for DataError {
  fn from(e: Error) -> Self {
    match e {
      Error::EntityNotFound { .. } => Self::not_found(e.to_string()),
      Error::NotFound { .. } => Self::not_found(e.to_string()),
      Error::UserAlreadyExists { .. } => Self::confilicted(e.to_string()),
      Error::UniqueViolation { .. } => Self::confilicted(e.to_string()),
      Error::SeaQueryError(_) => Self::bad_request(e.to_string()),
      _ => DataError::server_error(e.to_string()),
    }
  }
}
