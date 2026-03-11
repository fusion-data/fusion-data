use thiserror::Error;

pub type Result<T> = core::result::Result<T, DbxError>;

#[derive(Debug, Error)]
pub enum DbxError {
  #[error("Failed to count rows")]
  CountFail,

  #[error("Unsupported database: {0}. This operation requires a specific database backend (PostgreSQL or SQLite).")]
  UnsupportedDatabase(&'static str),

  #[error(
    "Cannot begin transaction: the database connection was not created with transaction support. Use `txn_cloned()` or `get_txn_clone()` to create a transaction-enabled connection."
  )]
  CannotBeginTxnWithTxnFalse,

  #[error("Cannot commit transaction: the database connection was not created with transaction support")]
  CannotCommitTxnWithTxnFalse,

  #[error("Cannot commit: no transaction is currently open. Did you call `begin_txn()` first?")]
  TxnCantCommitNoOpenTxn,

  #[error("Cannot rollback: no transaction is currently open")]
  NoTxn,

  #[error("Savepoint error: {0}")]
  SavePointError(String),

  #[error("Invalid database configuration: {0}")]
  ConfigInvalid(&'static str),

  #[error(
    "Transaction depth mismatch: begin was called {begin_count} time(s) but commit/rollback was called {end_count} time(s). This may indicate unbalanced transaction calls."
  )]
  TransactionDepthMismatch { begin_count: usize, end_count: usize },

  #[error(transparent)]
  Sqlx(#[from] sqlx::Error),
}
