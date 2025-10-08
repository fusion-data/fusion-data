use sea_query::{Condition, Expr, Query, SelectStatement};
use sea_query_binder::SqlxBinder;
use sqlx::Row;

use fusionsql_core::filter::FilterGroups;

use crate::base::utils::{build_sqlx_for_delete, build_sqlx_for_update};
use crate::base::{CommonIden, DbBmc, prep_fields_for_create, prep_fields_for_update};
use crate::field::{HasSeaFields, SeaField, SeaFields};
use crate::id::Id;
use crate::store::Dbx;
use crate::store::dbx::DbxProviderTrait;
use crate::{ModelManager, Result, SqlError};

/// Create a new entity。需要自增主键ID
pub async fn create<MC, E>(mm: &ModelManager, data: E) -> Result<i64>
where
  MC: DbBmc,
  E: HasSeaFields,
{
  let ctx = mm.ctx_ref()?;
  // -- Extract fields (name / sea-query value expression)
  let mut fields = data.not_none_sea_fields();
  fields = prep_fields_for_create::<MC>(fields, ctx);

  // -- Build query
  let (columns, sea_values) = fields.for_sea_insert();
  let mut stmt = Query::insert();
  stmt
    .into_table(MC::table_ref())
    .columns(columns)
    .values(sea_values)?
    .returning(Query::returning().columns([MC::COLUMN_ID]));

  // -- Exec query
  let id = mm.dbx().create(stmt).await?;
  Ok(id)
}

pub async fn create_many<MC, E>(mm: &ModelManager, data: Vec<E>) -> Result<Vec<i64>>
where
  MC: DbBmc,
  E: HasSeaFields,
{
  let ctx = mm.ctx_ref()?;

  // Prepare insert query
  let mut stmt = Query::insert();

  for item in data {
    let mut fields = item.not_none_sea_fields();
    fields = prep_fields_for_create::<MC>(fields, ctx);
    let (columns, sea_values) = fields.for_sea_insert();

    // Append values for each item
    stmt.into_table(MC::table_ref()).columns(columns).values(sea_values)?;
  }

  stmt.returning(Query::returning().columns([MC::COLUMN_ID]));

  // Execute query
  let ids = mm.dbx().create_many(stmt).await?;
  Ok(ids)
}

pub async fn insert<MC, E>(mm: &ModelManager, data: E) -> Result<()>
where
  MC: DbBmc,
  E: HasSeaFields,
{
  let ctx = mm.ctx_ref()?;

  // -- Extract fields (name / sea-query value expression)
  let mut fields = data.not_none_sea_fields();
  fields = prep_fields_for_create::<MC>(fields, ctx);

  // -- Build query
  let (columns, sea_values) = fields.for_sea_insert();
  let mut stmt = Query::insert();
  stmt.into_table(MC::table_ref()).columns(columns).values(sea_values)?;
  // .returning(Query::returning().columns([CommonIden::Id]));

  // -- Exec query
  if mm.dbx().execute(stmt).await? == 1 {
    Ok(())
  } else {
    Err(SqlError::ExecuteFail { schema: MC::SCHEMA, table: MC::TABLE })
  }
}

pub async fn insert_many<MC, E>(mm: &ModelManager, data: impl IntoIterator<Item = E>) -> Result<u64>
where
  MC: DbBmc,
  E: HasSeaFields,
{
  let ctx = mm.ctx_ref()?;

  // Prepare insert query
  let mut stmt = Query::insert();

  for item in data {
    let mut fields = item.not_none_sea_fields();
    fields = prep_fields_for_create::<MC>(fields, ctx);
    let (columns, sea_values) = fields.for_sea_insert();

    // Append values for each item
    stmt.into_table(MC::table_ref()).columns(columns).values(sea_values)?;
  }

  // Execute query
  let rows = mm.dbx().execute(stmt).await?;
  Ok(rows)
}

pub async fn count<MC, F>(mm: &ModelManager, filter: F) -> Result<u64>
where
  MC: DbBmc,
  F: Into<FilterGroups>,
{
  // -- Build the query
  let mut stmt = Query::select()
    .from(MC::table_ref())
    .expr_as(Expr::col(sea_query::Asterisk).count(), "count")
    .to_owned();

  // condition from filter
  let filters: FilterGroups = filter.into();
  let cond: Condition = filters.try_into()?;
  stmt.cond_where(cond);

  match mm.dbx() {
    #[cfg(feature = "with-postgres")]
    Dbx::Postgres(dbx_postgres) => {
      let query_str = stmt.to_string(sea_query::PostgresQueryBuilder);

      let result = sqlx::query(&query_str).fetch_one(dbx_postgres.db()).await.map_err(|e| {
        log::error!("count fail: {:?}", e);
        SqlError::CountFail { schema: MC::SCHEMA, table: MC::TABLE }
      })?;
      let count: i64 = result.try_get("count")?;
      Ok(count as u64)
    }
    #[cfg(feature = "with-sqlite")]
    Dbx::Sqlite(dbx_sqlite) => {
      let query_str = stmt.to_string(sea_query::SqliteQueryBuilder);
      let result = sqlx::query(&query_str)
        .fetch_one(dbx_sqlite.db())
        .await
        .map_err(|_| SqlError::CountFail { schema: MC::SCHEMA, table: MC::TABLE })?;
      let count: i64 =
        result.try_get("count").map_err(|_| SqlError::CountFail { schema: MC::SCHEMA, table: MC::TABLE })?;
      Ok(count as u64)
    }
  }
}

pub async fn count_on<MC, F>(mm: &ModelManager, f: F) -> Result<u64>
where
  MC: DbBmc,
  F: FnOnce(&mut SelectStatement) -> Result<()>,
{
  // -- Build the query
  let mut stmt = Query::select();
  stmt.from(MC::table_ref());
  stmt.expr(Expr::col(sea_query::Asterisk).count());

  // -- condition from filter
  f(&mut stmt)?;

  match mm.dbx() {
    #[cfg(feature = "with-postgres")]
    Dbx::Postgres(dbx_postgres) => {
      let query_str = stmt.to_string(sea_query::PostgresQueryBuilder);

      let result = sqlx::query(&query_str).fetch_one(dbx_postgres.db()).await.map_err(|e| {
        log::error!("count_on fail: {:?}", e);
        SqlError::CountFail { schema: MC::SCHEMA, table: MC::TABLE }
      })?;
      let count: i64 =
        result.try_get("count").map_err(|_| SqlError::CountFail { schema: MC::SCHEMA, table: MC::TABLE })?;
      Ok(count as u64)
    }
    #[cfg(feature = "with-sqlite")]
    Dbx::Sqlite(dbx_sqlite) => {
      let query_str = stmt.to_string(sea_query::SqliteQueryBuilder);
      let result = sqlx::query(&query_str)
        .fetch_one(dbx_sqlite.db())
        .await
        .map_err(|_| SqlError::CountFail { schema: MC::SCHEMA, table: MC::TABLE })?;
      let count: i64 =
        result.try_get("count").map_err(|_| SqlError::CountFail { schema: MC::SCHEMA, table: MC::TABLE })?;
      Ok(count as u64)
    }
  }
}

pub async fn update_by_id<MC, E>(mm: &ModelManager, id: Id, data: E) -> Result<()>
where
  MC: DbBmc,
  E: HasSeaFields,
{
  let ctx = mm.ctx_ref()?;

  // -- Prep Fields
  let mut fields = data.not_none_sea_fields();
  if MC::_has_updated_at() {
    fields = prep_fields_for_update::<MC>(fields, ctx);
  }

  // -- Build query
  let fields = fields.for_sea_update();
  let mut stmt = Query::update();
  stmt.table(MC::table_ref()).values(fields).and_where(Expr::col(MC::COLUMN_ID).eq(id.clone()));

  // -- Execute query
  let count = match mm.dbx() {
    #[cfg(feature = "with-postgres")]
    Dbx::Postgres(dbx_postgres) => {
      let (sql, values) = stmt.build_sqlx(sea_query::PostgresQueryBuilder);
      let sqlx_query = sqlx::query_with(&sql, values);
      dbx_postgres.execute(sqlx_query).await?
    }
    #[cfg(feature = "with-sqlite")]
    Dbx::Sqlite(dbx_sqlite) => {
      let (sql, values) = stmt.build_sqlx(sea_query::SqliteQueryBuilder);
      let sqlx_query = sqlx::query_with(&sql, values);
      dbx_sqlite.execute(sqlx_query).await?
    }
  };

  // -- Check result
  _check_result::<MC>(count, id)
}

/// 根据过滤条件更新，返回更新的记录数
pub async fn update<MC, E, F>(mm: &ModelManager, filter: F, data: E) -> Result<u64>
where
  MC: DbBmc,
  F: Into<FilterGroups>,
  E: HasSeaFields,
{
  let ctx = mm.ctx_ref()?;

  // -- Prep Fields
  let mut fields = data.not_none_sea_fields();
  if MC::_has_updated_at() {
    fields = prep_fields_for_update::<MC>(fields, ctx);
  }

  // -- Build query
  let fields = fields.for_sea_update();
  let mut stmt = Query::update();
  stmt.table(MC::table_ref()).values(fields);
  let filters: FilterGroups = filter.into();
  let cond: Condition = filters.try_into()?;
  stmt.cond_where(cond);

  // -- Execute query
  let count = match mm.dbx() {
    #[cfg(feature = "with-postgres")]
    Dbx::Postgres(dbx_postgres) => {
      let (sql, values) = stmt.build_sqlx(sea_query::PostgresQueryBuilder);
      let sqlx_query = sqlx::query_with(&sql, values);
      dbx_postgres.execute(sqlx_query).await?
    }
    #[cfg(feature = "with-sqlite")]
    Dbx::Sqlite(dbx_sqlite) => {
      let (sql, values) = stmt.build_sqlx(sea_query::SqliteQueryBuilder);
      let sqlx_query = sqlx::query_with(&sql, values);
      dbx_sqlite.execute(sqlx_query).await?
    }
  };

  Ok(count)
}

pub async fn delete_by_id<MC>(mm: &ModelManager, id: Id) -> Result<()>
where
  MC: DbBmc,
{
  let ctx = mm.ctx_ref()?;

  // -- Build query
  let (sql, values) = if MC::_use_logical_deletion() {
    // -- Prep Fields
    let mut fields = SeaFields::new(vec![SeaField::new(CommonIden::LogiscalDeletion, true)]);
    if MC::_has_updated_at() {
      fields = prep_fields_for_update::<MC>(fields, ctx);
    }

    let fields = fields.for_sea_update();
    let mut stmt = Query::update();
    stmt.table(MC::table_ref()).values(fields).and_where(Expr::col(MC::COLUMN_ID).eq(id.clone()));
    stmt.build_sqlx(sea_query::PostgresQueryBuilder)
  } else {
    let mut query = Query::delete();
    query.from_table(MC::table_ref()).and_where(Expr::col(MC::COLUMN_ID).eq(id.clone()));
    query.build_sqlx(sea_query::PostgresQueryBuilder)
  };

  // -- Execute query
  let count = match mm.dbx() {
    #[cfg(feature = "with-postgres")]
    Dbx::Postgres(dbx_postgres) => {
      let sqlx_query = sqlx::query_with(&sql, values);
      dbx_postgres.execute(sqlx_query).await?
    }
    #[cfg(feature = "with-sqlite")]
    Dbx::Sqlite(dbx_sqlite) => {
      let sqlx_query = sqlx::query_with(&sql, values);
      dbx_sqlite.execute(sqlx_query).await?
    }
  };

  // -- Check result
  _check_result::<MC>(count, id)
}

pub async fn delete_by_ids<MC>(mm: &ModelManager, ids: Vec<Id>) -> Result<u64>
where
  MC: DbBmc,
{
  let ctx: &fusion_common::ctx::Ctx = mm.ctx_ref()?;

  if ids.is_empty() {
    return Ok(0);
  }

  // -- Build query
  let (sql, values) = if MC::_use_logical_deletion() {
    // -- Prep Fields
    let mut fields = SeaFields::new(vec![SeaField::new(CommonIden::LogiscalDeletion, true)]);
    if MC::_has_updated_at() {
      fields = prep_fields_for_update::<MC>(fields, ctx);
    }
    let fields = fields.for_sea_update();
    let mut stmt = Query::update();
    stmt.table(MC::table_ref()).values(fields).and_where(Expr::col(MC::COLUMN_ID).is_in(ids));
    build_sqlx_for_update(mm.dbx().provider(), stmt)
  } else {
    let mut stmt = Query::delete();
    stmt.from_table(MC::table_ref()).and_where(Expr::col(MC::COLUMN_ID).is_in(ids));
    build_sqlx_for_delete(mm.dbx().provider(), stmt)
  };

  // -- Execute query
  let n = match mm.dbx() {
    #[cfg(feature = "with-postgres")]
    Dbx::Postgres(dbx_postgres) => {
      let sqlx_query = sqlx::query_with(&sql, values);
      dbx_postgres.execute(sqlx_query).await?
    }
    #[cfg(feature = "with-sqlite")]
    Dbx::Sqlite(dbx_sqlite) => {
      let sqlx_query = sqlx::query_with(&sql, values);
      dbx_sqlite.execute(sqlx_query).await?
    }
  };

  Ok(n)
}

pub async fn delete<MC, F>(mm: &ModelManager, filter: F) -> Result<u64>
where
  MC: DbBmc,
  F: Into<FilterGroups>,
{
  let ctx: &fusion_common::ctx::Ctx = mm.ctx_ref()?;

  let filters: FilterGroups = filter.into();
  let cond: Condition = filters.try_into()?;

  // -- Build query
  let (sql, values) = if MC::_use_logical_deletion() {
    // -- Prep Fields
    let mut fields = SeaFields::new(vec![SeaField::new(CommonIden::LogiscalDeletion, true)]);
    if MC::_has_updated_at() {
      fields = prep_fields_for_update::<MC>(fields, ctx);
    }
    let fields = fields.for_sea_update();
    let mut stmt = Query::update();
    stmt.table(MC::table_ref()).values(fields).cond_where(cond);
    build_sqlx_for_update(mm.dbx().provider(), stmt)
  } else {
    let mut stmt = Query::delete();
    stmt.from_table(MC::table_ref());
    stmt.cond_where(cond);
    build_sqlx_for_delete(mm.dbx().provider(), stmt)
  };

  // -- Execute query
  let n = match mm.dbx() {
    #[cfg(feature = "with-postgres")]
    Dbx::Postgres(dbx_postgres) => {
      let sqlx_query = sqlx::query_with(&sql, values);
      dbx_postgres.execute(sqlx_query).await?
    }
    #[cfg(feature = "with-sqlite")]
    Dbx::Sqlite(dbx_sqlite) => {
      let sqlx_query = sqlx::query_with(&sql, values);
      dbx_sqlite.execute(sqlx_query).await?
    }
  };

  Ok(n)
}

/// Check result
fn _check_result<MC>(count: u64, id: Id) -> Result<()>
where
  MC: DbBmc,
{
  if count == 0 { Err(SqlError::EntityNotFound { schema: MC::SCHEMA, entity: MC::TABLE, id }) } else { Ok(()) }
}
