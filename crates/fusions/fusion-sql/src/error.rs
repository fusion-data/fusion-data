use std::borrow::Cow;

use fusion_sql_core::filter::IntoSeaError;
use sqlx::error::DatabaseError;
use thiserror::Error;

use crate::id::Id;

pub type Result<T> = core::result::Result<T, SqlError>;

#[derive(Debug, Error)]
pub enum SqlError {
  #[error("Unauthorized: {0}")]
  Unauthorized(String),

  #[error("Execute error. table: {table}, message: {message}")]
  ExecuteError { table: String, message: String },

  #[error("Execute fail. table: '{schema:?}.{table}'")]
  ExecuteFail { schema: Option<&'static str>, table: &'static str },

  #[error("Count fail. table: '{schema:?}.{table}'")]
  CountFail { schema: Option<&'static str>, table: &'static str },

  #[error("Invalid database. database: {0}")]
  InvalidDatabase(&'static str),

  #[error("Invalid argument, error message: {message}")]
  InvalidArgument { message: String },

  #[error("Entity not found. entity: '{schema:?}.{entity}', id: {id:?}")]
  EntityNotFound { schema: Option<&'static str>, entity: &'static str, id: Id },

  #[error("Data not found. table is '{schema:?}.{table}'")]
  NotFound { schema: Option<&'static str>, table: &'static str, sql: String },

  #[error("List limit over max. max: {max}, actual: {actual}")]
  ListLimitOverMax { max: u64, actual: u64 },

  #[error("List limit under min. min: {min}, actual: {actual}")]
  ListLimitUnderMin { min: u64, actual: u64 },

  #[error("List page under min. min: {min}, actual: {actual}")]
  ListPageUnderMin { min: u64, actual: u64 },

  // -- DB
  #[error("User already exists. {key}: '{value}'")]
  UserAlreadyExists { key: &'static str, value: String },

  #[error("Unique violation. table: '{table}', constraint: {constraint}")]
  UniqueViolation { table: String, constraint: String },

  // -- ModelManager
  #[error("Can't create ModelManagerProvider. provider: {0}")]
  CantCreateModelManagerProvider(String),

  #[error(transparent)]
  IntoSeaError(#[from] IntoSeaError),

  // -- Externals
  #[error(transparent)]
  SeaQueryError(#[from] sea_query::error::Error),

  #[error(transparent)]
  JsonError(#[from] serde_json::Error),

  #[error(transparent)]
  DbxError(#[from] crate::store::dbx::DbxError),

  #[error(transparent)]
  Sqlx(#[from] sqlx::Error),
}

impl SqlError {
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
        .unwrap_or_else(|| SqlError::UniqueViolation { table: table.to_string(), constraint: constraint.to_string() }),
      _ => self,
    }
  }

  /// A convenient function to return the eventual database error (Postgres)
  /// if this Error is an SQLX Error that contains a database error.
  pub fn as_database_error(&self) -> Option<&(dyn DatabaseError + 'static)> {
    match self {
      SqlError::DbxError(crate::store::dbx::DbxError::Sqlx(sqlx_error)) => sqlx_error.as_database_error(),
      _ => None,
    }
  }
}
