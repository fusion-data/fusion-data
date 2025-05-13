use std::{
  ops::{Deref, DerefMut},
  sync::Arc,
};

use log::LevelFilter;
use sqlx::{
  query::{Query, QueryAs},
  sqlite::{Sqlite, SqliteConnectOptions, SqlitePoolOptions},
  ConnectOptions, FromRow, IntoArguments, Pool, Transaction,
};

use tokio::sync::Mutex;
use tracing::{debug, trace};

use crate::DbConfig;

use super::{DbxError, DbxType, DbxTypeTrait, Result};

type Db = Pool<Sqlite>;

pub fn new_sqlite_pool_from_config(c: &DbConfig, _application_name: Option<&str>) -> Result<Db> {
  if !c.enable() {
    return Err(DbxError::ConfigInvalid("Need set ultimate.db.enable = true"));
  }

  let mut pool_opts = SqlitePoolOptions::new();
  if let Some(v) = c.max_connections() {
    pool_opts = pool_opts.max_connections(v);
  }
  if let Some(v) = c.min_connections() {
    pool_opts = pool_opts.min_connections(v);
  }
  if let Some(v) = c.acquire_timeout() {
    pool_opts = pool_opts.acquire_timeout(*v);
  }
  if let Some(v) = c.idle_timeout() {
    pool_opts = pool_opts.idle_timeout(*v);
  }
  if let Some(v) = c.max_lifetime() {
    pool_opts = pool_opts.max_lifetime(*v);
  }
  trace!("Sqlite connection options are: {:?}", pool_opts);

  let mut conn_opts: SqliteConnectOptions =
    if let Some(url) = c.url() { url.parse().unwrap() } else { Default::default() };
  conn_opts = conn_opts.log_statements(LevelFilter::Debug);

  let pool = pool_opts.connect_lazy_with(conn_opts);
  debug!("Connect to db pool: {:?}", pool);
  Ok(pool)
}

#[derive(Debug, Clone)]
pub struct DbxSqlite {
  db_pool: Db,
  txn_holder: Arc<Mutex<Option<TxnHolder>>>,
  txn: bool,
}

impl DbxTypeTrait for DbxSqlite {
  fn dbx_type(&self) -> &DbxType {
    &DbxType::Sqlite
  }
}

impl DbxSqlite {
  pub fn new(db_pool: Db, txn: bool) -> Self {
    Self { db_pool, txn_holder: Arc::default(), txn }
  }

  pub fn is_txn(&self) -> bool {
    self.txn
  }

  pub fn non_txn(&self) -> bool {
    !self.txn
  }
}

impl DbxSqlite {
  pub async fn begin_txn(&self) -> Result<()> {
    if !self.txn {
      return Err(DbxError::CannotBeginTxnWithTxnFalse);
    }

    let mut txh_g = self.txn_holder.lock().await;
    // If we already have a tx holder, then, we increment
    if let Some(txh) = txh_g.as_mut() {
      txh.inc();
    } else {
      // If not, we create one with a new transaction
      let txn = self.db_pool.begin().await?;
      let _ = txh_g.insert(TxnHolder::new(txn));
    }

    Ok(())
  }

  pub async fn rollback_txn(&self) -> Result<()> {
    let mut txh_g = self.txn_holder.lock().await;
    if let Some(mut txh) = txh_g.take() {
      // Take the TxnHolder out of the Option
      if txh.counter > 1 {
        txh.counter -= 1;
        let _ = txh_g.replace(txh); // Put it back if not the last reference
      } else {
        // Perform the actual rollback
        txh.txn.rollback().await?;
        // No need to replace, as we want to leave it as None
      }
      Ok(())
    } else {
      Err(DbxError::NoTxn)
    }
  }

  pub async fn commit_txn(&self) -> Result<()> {
    if !self.txn {
      return Err(DbxError::CannotCommitTxnWithTxnFalse);
    }

    let mut txh_g = self.txn_holder.lock().await;
    if let Some(txh) = txh_g.as_mut() {
      let counter = txh.dec();
      // If 0, then, it should be matching commit for the first first begin_txn
      // so we can commit.
      if counter == 0 {
        // here we take the txh out of the option
        if let Some(txh) = txh_g.take() {
          txh.txn.commit().await?;
          // txn.txn.as_mut().commit().await?;
        } // TODO: Might want to add a warning on the else.
      } // TODO: Might want to add a warning on the else.

      Ok(())
    }
    // Ohterwise, we have an error
    else {
      Err(DbxError::TxnCantCommitNoOpenTxn)
    }
  }

  pub fn db(&self) -> &Db {
    &self.db_pool
  }

  pub async fn fetch_one<'q, O, A>(&self, query: QueryAs<'q, Sqlite, O, A>) -> Result<O>
  where
    O: for<'r> FromRow<'r, <Sqlite as sqlx::Database>::Row> + Send + Unpin,
    A: IntoArguments<'q, Sqlite> + 'q,
  {
    if self.txn {
      let mut txh_g = self.txn_holder.lock().await;
      if let Some(txn) = txh_g.as_deref_mut() {
        let res = query.fetch_one(txn.as_mut()).await?;
        return Ok(res);
      }
    }

    let res = query.fetch_one(self.db()).await?;
    Ok(res)
  }

  pub async fn fetch_optional<'q, O, A>(&self, query: QueryAs<'q, Sqlite, O, A>) -> Result<Option<O>>
  where
    O: for<'r> FromRow<'r, <Sqlite as sqlx::Database>::Row> + Send + Unpin,
    A: IntoArguments<'q, Sqlite> + 'q,
  {
    let data = if self.txn {
      let mut txh_g = self.txn_holder.lock().await;
      if let Some(txn) = txh_g.as_deref_mut() {
        query.fetch_optional(txn.as_mut()).await?
      } else {
        query.fetch_optional(self.db()).await?
      }
    } else {
      query.fetch_optional(self.db()).await?
    };

    Ok(data)
  }

  pub async fn fetch_all<'q, O, A>(&self, query: QueryAs<'q, Sqlite, O, A>) -> Result<Vec<O>>
  where
    O: for<'r> FromRow<'r, <Sqlite as sqlx::Database>::Row> + Send + Unpin,
    A: IntoArguments<'q, Sqlite> + 'q,
  {
    let data = if self.txn {
      let mut txh_g = self.txn_holder.lock().await;
      if let Some(txn) = txh_g.as_deref_mut() {
        query.fetch_all(txn.as_mut()).await?
      } else {
        query.fetch_all(self.db()).await?
      }
    } else {
      query.fetch_all(self.db()).await?
    };

    Ok(data)
  }

  pub async fn execute<'q, A>(&self, query: Query<'q, Sqlite, A>) -> Result<u64>
  where
    A: IntoArguments<'q, Sqlite> + 'q,
  {
    let row_affected = if self.txn {
      let mut txh_g = self.txn_holder.lock().await;
      if let Some(txn) = txh_g.as_deref_mut() {
        query.execute(txn.as_mut()).await?.rows_affected()
      } else {
        query.execute(self.db()).await?.rows_affected()
      }
    } else {
      query.execute(self.db()).await?.rows_affected()
    };

    Ok(row_affected)
  }
}

#[derive(Debug)]
struct TxnHolder {
  txn: Transaction<'static, Sqlite>,
  counter: i32,
}

impl TxnHolder {
  fn new(txn: Transaction<'static, Sqlite>) -> Self {
    TxnHolder { txn, counter: 1 }
  }

  fn inc(&mut self) {
    self.counter += 1;
  }

  fn dec(&mut self) -> i32 {
    self.counter -= 1;
    self.counter
  }
}

impl Deref for TxnHolder {
  type Target = Transaction<'static, Sqlite>;

  fn deref(&self) -> &Self::Target {
    &self.txn
  }
}

impl DerefMut for TxnHolder {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.txn
  }
}
