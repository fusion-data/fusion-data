#[cfg(feature = "with-postgres")]
mod dbx_postgres;
#[cfg(feature = "with-sqlite")]
mod dbx_sqlite;
mod error;

use std::future::Future;

pub use error::{DbxError, Result};

#[cfg(feature = "with-postgres")]
pub use dbx_postgres::*;
#[cfg(feature = "with-sqlite")]
pub use dbx_sqlite::*;
use sea_query::InsertStatement;
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};

use crate::DbConfig;

pub trait DbxProviderTrait {
  fn provider(&self) -> &DbxProvider;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DbxProvider {
  #[cfg(feature = "with-postgres")]
  Postgres,
  #[cfg(feature = "with-sqlite")]
  Sqlite,
}

#[derive(Debug, Clone)]
// #[non_exhaustive]
pub enum Dbx {
  #[cfg(feature = "with-postgres")]
  Postgres(DbxPostgres),
  #[cfg(feature = "with-sqlite")]
  Sqlite(DbxSqlite),
}

impl DbxProviderTrait for Dbx {
  fn provider(&self) -> &DbxProvider {
    match self {
      #[cfg(feature = "with-postgres")]
      Dbx::Postgres(_) => &DbxProvider::Postgres,
      #[cfg(feature = "with-sqlite")]
      Dbx::Sqlite(_) => &DbxProvider::Sqlite,
    }
  }
}

pub async fn create_dbx(db_config: &DbConfig, application_name: Option<&str>) -> Result<Dbx> {
  let dbx = match db_config.dbx_type() {
    #[cfg(feature = "with-postgres")]
    DbxProvider::Postgres => {
      Dbx::Postgres(DbxPostgres::new(new_pg_pool_from_config(db_config, application_name).await?, false))
    }
    #[cfg(feature = "with-sqlite")]
    DbxProvider::Sqlite => {
      Dbx::Sqlite(DbxSqlite::new(new_sqlite_pool_from_config(db_config, application_name)?, false))
    }
  };
  Ok(dbx)
}

impl Dbx {
  #[cfg(feature = "with-postgres")]
  pub async fn use_postgres<F, Fut, T>(&self, f: F) -> Result<T>
  where
    F: FnOnce(DbxPostgres) -> Fut,
    Fut: Future<Output = Result<T>>,
  {
    match self {
      Dbx::Postgres(dbx) => f(dbx.clone()).await,
      #[allow(unreachable_patterns)]
      _ => Err(DbxError::UnsupportedDatabase("Need postgres database")),
    }
  }

  #[cfg(feature = "with-sqlite")]
  pub async fn use_sqlite<F, Fut, T>(&self, f: F) -> Result<T>
  where
    F: FnOnce(DbxSqlite) -> Fut,
    Fut: Future<Output = Result<T>>,
  {
    match self {
      Dbx::Sqlite(dbx) => f(dbx.clone()).await,
      #[allow(unreachable_patterns)]
      _ => Err(DbxError::UnsupportedDatabase("Need sqlite database")),
    }
  }

  pub fn txn_cloned(&self) -> Dbx {
    match self {
      #[cfg(feature = "with-postgres")]
      Dbx::Postgres(dbx_postgres) => Dbx::Postgres(DbxPostgres::new(dbx_postgres.db().clone(), true)),
      #[cfg(feature = "with-sqlite")]
      Dbx::Sqlite(dbx_sqlite) => Dbx::Sqlite(DbxSqlite::new(dbx_sqlite.db().clone(), true)),
    }
  }

  pub fn is_txn(&self) -> bool {
    match self {
      #[cfg(feature = "with-postgres")]
      Dbx::Postgres(dbx_postgres) => dbx_postgres.is_txn(),
      #[cfg(feature = "with-sqlite")]
      Dbx::Sqlite(dbx_sqlite) => dbx_sqlite.is_txn(),
    }
  }

  pub async fn create(&self, query: InsertStatement) -> Result<i64> {
    match self {
      #[cfg(feature = "with-postgres")]
      Dbx::Postgres(dbx_postgres) => {
        let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);
        // NOTE: For now, we will use the _txn for all create.
        //       We could have a with_txn as function argument if perf is an issue (it should not be)
        let (id,) = dbx_postgres.fetch_one(sqlx_query).await?;
        Ok(id)
      }
      #[cfg(feature = "with-sqlite")]
      Dbx::Sqlite(dbx_sqlite) => {
        let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);
        let (id,) = dbx_sqlite.fetch_one(sqlx_query).await?;
        Ok(id)
      }
    }
  }

  pub async fn create_many(&self, query: InsertStatement) -> Result<Vec<i64>> {
    match self {
      #[cfg(feature = "with-postgres")]
      Dbx::Postgres(dbx_postgres) => {
        let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);
        let rows = dbx_postgres.fetch_all(sqlx_query).await?;
        let ids = rows.iter().map(|row| row.0).collect();
        Ok(ids)
      }
      #[cfg(feature = "with-sqlite")]
      Dbx::Sqlite(dbx_sqlite) => {
        let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);
        let rows = dbx_sqlite.fetch_all(sqlx_query).await?;
        let ids = rows.iter().map(|row| row.0).collect();
        Ok(ids)
      }
    }
  }

  // pub async fn insert(&self, query: InsertStatement) -> Result<u64> {
  //   // let count = match self {
  //   //   #[cfg(feature = "with-postgres")]
  //   //   Dbx::Postgres(dbx_postgres) => {
  //   //     let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
  //   //     let sqlx_query = sqlx::query_with(&sql, values);
  //   //     dbx_postgres.execute(sqlx_query).await?
  //   //   }
  //   //   #[cfg(feature = "with-sqlite")]
  //   //   Dbx::Sqlite(dbx_sqlite) => {
  //   //     let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
  //   //     let sqlx_query = sqlx::query_with(&sql, values);
  //   //     dbx_sqlite.execute(sqlx_query).await?
  //   //   }
  //   // };
  //   // Ok(count)
  //   self.execute(query).await
  // }

  pub async fn execute<Q>(&self, query: Q) -> Result<u64>
  where
    Q: SqlxBinder,
  {
    let count = match self {
      #[cfg(feature = "with-postgres")]
      Dbx::Postgres(dbx_postgres) => {
        let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
        let sqlx_query = sqlx::query_with(&sql, values);
        dbx_postgres.execute(sqlx_query).await?
      }
      #[cfg(feature = "with-sqlite")]
      Dbx::Sqlite(dbx_sqlite) => {
        let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
        let sqlx_query = sqlx::query_with(&sql, values);
        dbx_sqlite.execute(sqlx_query).await?
      }
    };
    Ok(count)
  }

  pub async fn begin_txn(&self) -> Result<()> {
    match self {
      #[cfg(feature = "with-postgres")]
      Dbx::Postgres(dbx_postgres) => dbx_postgres.begin_txn().await,
      #[cfg(feature = "with-sqlite")]
      Dbx::Sqlite(dbx_sqlite) => dbx_sqlite.begin_txn().await,
    }
  }

  pub async fn commit_txn(&self) -> Result<()> {
    match self {
      #[cfg(feature = "with-postgres")]
      Dbx::Postgres(dbx_postgres) => dbx_postgres.commit_txn().await,
      #[cfg(feature = "with-sqlite")]
      Dbx::Sqlite(dbx_sqlite) => dbx_sqlite.commit_txn().await,
    }
  }

  #[cfg(feature = "with-postgres")]
  pub fn db_postgres(&self) -> Result<sqlx::PgPool> {
    match self {
      Dbx::Postgres(dbx_postgres) => Ok(dbx_postgres.db().clone()),
      #[allow(unreachable_patterns)]
      _ => Err(DbxError::UnsupportedDatabase("Need postgres database")),
    }
  }

  #[cfg(feature = "with-sqlite")]
  pub fn db_sqlite(&self) -> Result<sqlx::SqlitePool> {
    match self {
      Dbx::Sqlite(dbx_sqlite) => Ok(dbx_sqlite.db().clone()),
      #[allow(unreachable_patterns)]
      _ => Err(DbxError::UnsupportedDatabase("Need sqlite database")),
    }
  }
}
