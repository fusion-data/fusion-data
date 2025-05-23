use thiserror::Error;

pub type Result<T> = core::result::Result<T, DbxError>;

#[derive(Debug, Error)]
pub enum DbxError {
  #[error("Count fail")]
  CountFail,

  #[error("UnsupportedDatabase({0})")]
  UnsupportedDatabase(&'static str),

  #[error("TxnCantCommitNoOpenTxn")]
  TxnCantCommitNoOpenTxn,

  #[error("CannotBeginTxnWithTxnFalse")]
  CannotBeginTxnWithTxnFalse,

  #[error("CannotCommitTxnWithTxnFalse")]
  CannotCommitTxnWithTxnFalse,

  #[error("NoTxn")]
  NoTxn,

  #[error("ConfigInvalid({0})")]
  ConfigInvalid(&'static str),

  #[error(transparent)]
  Sqlx(#[from] sqlx::Error),
}
