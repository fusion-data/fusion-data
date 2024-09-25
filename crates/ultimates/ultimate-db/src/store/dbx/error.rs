use thiserror::Error;
use ultimate::DataError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
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

impl From<Error> for DataError {
  fn from(e: Error) -> Self {
    // TODO 更多的日志打印
    DataError::server_error(e.to_string())
  }
}
