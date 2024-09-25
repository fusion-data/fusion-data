// region:    --- Modules

use std::net::ToSocketAddrs;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use sqlx::postgres::any::AnyConnectionBackend;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::query::{Query, QueryAs};
use sqlx::{ConnectOptions, FromRow, IntoArguments, Pool, Postgres, Transaction};
use tokio::sync::Mutex;
use tracing::trace;
use ultimate::configuration::model::DbConf;

mod error;

pub use error::{Error, Result};

// endregion: --- Modules

pub type Db = Pool<Postgres>;

pub async fn new_db_pool_from_config(c: &DbConf) -> Result<Db> {
  if !c.enable() {
    return Err(Error::ConfigInvalid("Need set ultimate.db.enable = true"));
  }

  let mut opt = PgPoolOptions::new();
  if let Some(v) = c.max_connections() {
    opt = opt.max_connections(v);
  }
  if let Some(v) = c.min_connections() {
    opt = opt.min_connections(v);
  }
  if let Some(v) = c.acquire_timeout() {
    opt = opt.acquire_timeout(*v);
  }
  if let Some(v) = c.idle_timeout() {
    opt = opt.idle_timeout(*v);
  }
  if let Some(v) = c.max_lifetime() {
    opt = opt.max_lifetime(*v);
  }

  trace!("Db connection options are: {:?}", opt);

  let level = log::LevelFilter::Debug;
  let mut opts: PgConnectOptions = match c.url() {
    Some(url) => url.parse()?,
    None => {
      let mut o = PgConnectOptions::new();
      if let Some(host) = c.host() {
        o = o.host(host);
      }
      if let Some(port) = c.port() {
        o = o.port(port);
      }
      if let Some(socket) = c.socket() {
        o = o.socket(socket);
      }
      if let Some(database) = c.database() {
        o = o.database(database)
      }
      if let Some(username) = c.username() {
        o = o.username(username)
      }
      if let Some(password) = c.password() {
        o = o.password(password)
      }
      o
    }
  };

  // TODO 若 opts.host 是域名，需要进行DNS查找将期转换为 ip addr
  let non_ip_addr = opts.get_host().parse::<std::net::IpAddr>().is_err();
  if non_ip_addr {
    let original_host = format!("{}:{}", opts.get_host(), opts.get_port());
    let sock_addr = original_host.to_socket_addrs().unwrap().next().unwrap();
    opts = opts.host(&sock_addr.ip().to_string());
    trace!("Resolve original host, from {} to {}", original_host, opts.get_host());
  }

  opts = opts.log_statements(level);

  let db = opt.connect_with(opts).await?;
  Ok(db)
}

#[derive(Debug, Clone)]
pub struct Dbx {
  db_pool: Db,
  txn_holder: Arc<Mutex<Option<TxnHolder>>>,
  txn: bool,
}

impl Dbx {
  pub fn new(db_pool: Db, txn: bool) -> Result<Self> {
    Ok(Dbx { db_pool, txn_holder: Arc::default(), txn })
  }

  pub fn is_txn(&self) -> bool {
    self.txn
  }

  pub fn non_txn(&self) -> bool {
    !self.txn
  }
}

impl Dbx {
  pub async fn begin_txn(&self) -> Result<()> {
    if !self.txn {
      return Err(Error::CannotBeginTxnWithTxnFalse);
    }

    let mut txh_g = self.txn_holder.lock().await;
    // If we already have a tx holder, then, we increment
    if let Some(txh) = txh_g.as_mut() {
      txh.inc();
    }
    // If not, we create one with a new transaction
    else {
      let transaction = self.db_pool.begin().await?;
      let _ = txh_g.insert(TxnHolder::new(transaction));
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
      Err(Error::NoTxn)
    }
  }

  pub async fn commit_txn(&self) -> Result<()> {
    if !self.txn {
      return Err(Error::CannotCommitTxnWithTxnFalse);
    }

    let mut txh_g = self.txn_holder.lock().await;
    if let Some(txh) = txh_g.as_mut() {
      let counter = txh.dec();
      // If 0, then, it should be matching commit for the first first begin_txn
      // so we can commit.
      if counter == 0 {
        // here we take the txh out of the option
        if let Some(mut txn) = txh_g.take() {
          txn.txn.as_mut().commit().await?;
        } // TODO: Might want to add a warning on the else.
      } // TODO: Might want to add a warning on the else.

      Ok(())
    }
    // Ohterwise, we have an error
    else {
      Err(Error::TxnCantCommitNoOpenTxn)
    }
  }

  pub fn db(&self) -> &Pool<Postgres> {
    &self.db_pool
  }

  pub async fn fetch_one<'q, O, A>(&self, query: QueryAs<'q, Postgres, O, A>) -> Result<O>
  where
    O: for<'r> FromRow<'r, <Postgres as sqlx::Database>::Row> + Send + Unpin,
    A: IntoArguments<'q, Postgres> + 'q,
  {
    let data = if self.txn {
      let mut txh_g = self.txn_holder.lock().await;
      if let Some(txn) = txh_g.as_deref_mut() {
        query.fetch_one(txn.as_mut()).await?
      } else {
        query.fetch_one(self.db()).await?
      }
    } else {
      query.fetch_one(self.db()).await?
    };

    Ok(data)
  }

  pub async fn fetch_optional<'q, O, A>(&self, query: QueryAs<'q, Postgres, O, A>) -> Result<Option<O>>
  where
    O: for<'r> FromRow<'r, <Postgres as sqlx::Database>::Row> + Send + Unpin,
    A: IntoArguments<'q, Postgres> + 'q,
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

  pub async fn fetch_all<'q, O, A>(&self, query: QueryAs<'q, Postgres, O, A>) -> Result<Vec<O>>
  where
    O: for<'r> FromRow<'r, <Postgres as sqlx::Database>::Row> + Send + Unpin,
    A: IntoArguments<'q, Postgres> + 'q,
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

  pub async fn execute<'q, A>(&self, query: Query<'q, Postgres, A>) -> Result<u64>
  where
    A: IntoArguments<'q, Postgres> + 'q,
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
  txn: Transaction<'static, Postgres>,
  counter: i32,
}

impl TxnHolder {
  fn new(txn: Transaction<'static, Postgres>) -> Self {
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
  type Target = Transaction<'static, Postgres>;

  fn deref(&self) -> &Self::Target {
    &self.txn
  }
}

impl DerefMut for TxnHolder {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.txn
  }
}
