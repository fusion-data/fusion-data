use sea_query::{Condition, Expr, PostgresQueryBuilder, Query, SelectStatement};
use sea_query_binder::SqlxBinder;
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use sqlx::Row;

use crate::base::{prep_fields_for_create, prep_fields_for_update, CommonIden, DbBmc};
use crate::field::{HasSeaFields, SeaField, SeaFields};
use crate::filter::{FilterGroups, ListOptions};
use crate::id::Id;
use crate::page::PageResult;
use crate::{ModelManager, Result, SqlError};

/// Create a new entity。需要自增主键ID
#[tracing::instrument(skip(mm, data))]
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
  let mut query = Query::insert();
  query
    .into_table(MC::table_ref())
    .columns(columns)
    .values(sea_values)?
    .returning(Query::returning().columns([CommonIden::Id]));

  // -- Exec query
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);
  // NOTE: For now, we will use the _txn for all create.
  //       We could have a with_txn as function argument if perf is an issue (it should not be)
  let (id,) = mm.dbx().fetch_one(sqlx_query).await?;
  Ok(id)
}

#[tracing::instrument(skip(mm, data))]
pub async fn create_many<MC, E>(mm: &ModelManager, data: Vec<E>) -> Result<Vec<i64>>
where
  MC: DbBmc,
  E: HasSeaFields,
{
  let ctx = mm.ctx_ref()?;
  let mut ids = Vec::with_capacity(data.len());

  // Prepare insert query
  let mut query = Query::insert();

  for item in data {
    let mut fields = item.not_none_sea_fields();
    fields = prep_fields_for_create::<MC>(fields, ctx);
    let (columns, sea_values) = fields.for_sea_insert();

    // Append values for each item
    query.into_table(MC::table_ref()).columns(columns.clone()).values(sea_values)?;
  }

  query.returning(Query::returning().columns([CommonIden::Id]));

  // Execute query
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);

  let rows = mm.dbx().fetch_all(sqlx_query).await?;
  for row in rows {
    let (id,): (i64,) = row;
    ids.push(id);
  }

  Ok(ids)
}

#[tracing::instrument(skip(mm, data))]
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
  let mut query = Query::insert();
  query.into_table(MC::table_ref()).columns(columns).values(sea_values)?;
  // .returning(Query::returning().columns([CommonIden::Id]));

  // -- Exec query
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let sqlx_query = sqlx::query_with(&sql, values);

  let count = mm.dbx().execute(sqlx_query).await?;
  if count == 1 {
    Ok(())
  } else {
    // TODO 需要更有效的插入失败错误
    Err(SqlError::CountFail)
  }
}

#[tracing::instrument(skip(mm, data))]
pub async fn insert_many<MC, E>(mm: &ModelManager, data: impl IntoIterator<Item = E>) -> Result<u64>
where
  MC: DbBmc,
  E: HasSeaFields,
{
  let ctx = mm.ctx_ref()?;

  // Prepare insert query
  let mut query = Query::insert();

  for item in data {
    let mut fields = item.not_none_sea_fields();
    fields = prep_fields_for_create::<MC>(fields, ctx);
    let (columns, sea_values) = fields.for_sea_insert();

    // Append values for each item
    query.into_table(MC::table_ref()).columns(columns).values(sea_values)?;
  }

  // Execute query
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let sqlx_query = sqlx::query_with(&sql, values);
  let rows = mm.dbx().execute(sqlx_query).await?;
  Ok(rows)
}

#[tracing::instrument(skip(mm, id))]
pub async fn find_by_id<MC, E>(mm: &ModelManager, id: Id) -> Result<E>
where
  MC: DbBmc,
  E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
  E: HasSeaFields,
{
  // -- Build the query
  let mut query = Query::select();
  query.from(MC::table_ref()).columns(E::sea_column_refs());

  // condition from filter
  let filters: FilterGroups = id.to_filter_node("id").into();
  let cond: Condition = filters.try_into()?;
  query.cond_where(cond);

  // -- Execute the query
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
  match mm.dbx().fetch_optional(sqlx_query).await? {
    Some(entity) => Ok(entity),
    None => Err(SqlError::EntityNotFound { schema: MC::SCHEMA, entity: MC::TABLE, id }),
  }
}

#[tracing::instrument(skip(mm, filter))]
pub async fn find_unique<MC, E, F>(mm: &ModelManager, filter: F) -> Result<Option<E>>
where
  MC: DbBmc,
  E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
  E: HasSeaFields,
  F: Into<FilterGroups>,
{
  // -- Build the query
  let mut query = Query::select();
  query.from(MC::table_ref()).columns(E::sea_column_refs());

  // condition from filter
  let filters: FilterGroups = filter.into();
  let cond: Condition = filters.try_into()?;
  query.cond_where(cond);

  // -- Execute the query
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
  let entity = mm.dbx().fetch_optional(sqlx_query).await?;

  Ok(entity)
}

#[tracing::instrument(skip(mm, filter))]
pub async fn find_first<MC, E, F>(mm: &ModelManager, filter: F) -> Result<Option<E>>
where
  MC: DbBmc,
  E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
  E: HasSeaFields,
  F: Into<FilterGroups>,
{
  let list = find_many::<MC, E, F>(mm, filter, None).await?;
  Ok(list.into_iter().next())
}

#[tracing::instrument(skip(mm, filter, list_options))]
pub async fn find_many<MC, E, F>(mm: &ModelManager, filter: F, list_options: Option<ListOptions>) -> Result<Vec<E>>
where
  MC: DbBmc,
  E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
  E: HasSeaFields,
  F: Into<FilterGroups>,
{
  // -- Build the query
  let mut query = Query::select();
  query.from(MC::table_ref()).columns(E::sea_column_refs());

  // condition from filter
  let filters: FilterGroups = filter.into();
  let cond: Condition = filters.try_into()?;
  query.cond_where(cond);

  // list options
  let list_options = compute_list_options::<MC>(list_options)?;
  list_options.apply_to_sea_query(&mut query);

  // -- Execute the query
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

  let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
  let entities = mm.dbx().fetch_all(sqlx_query).await?;

  Ok(entities)
}

#[tracing::instrument(skip(mm, f))]
pub async fn find_many_on<MC, E, F>(mm: &ModelManager, f: F) -> Result<Vec<E>>
where
  MC: DbBmc,
  E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
  E: HasSeaFields,
  F: FnOnce(&mut SelectStatement) -> Result<()>,
{
  // -- Build the query
  let mut query = Query::select();
  query.from(MC::table_ref()).columns(E::sea_column_refs());

  // condition from filter and list options
  f(&mut query)?;

  // -- Execute the query
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

  let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
  let entities = mm.dbx().fetch_all(sqlx_query).await?;

  Ok(entities)
}

#[tracing::instrument(skip(mm, filter))]
pub async fn count<MC, F>(mm: &ModelManager, filter: F) -> Result<i64>
where
  MC: DbBmc,
  F: Into<FilterGroups>,
{
  let db = mm.dbx().db();
  // -- Build the query
  let mut query = Query::select().from(MC::table_ref()).expr(Expr::col(sea_query::Asterisk).count()).to_owned();

  // condition from filter
  let filters: FilterGroups = filter.into();
  let cond: Condition = filters.try_into()?;
  query.cond_where(cond);

  let query_str = query.to_string(PostgresQueryBuilder);

  let result = sqlx::query(&query_str).fetch_one(db).await.map_err(|_| SqlError::CountFail)?;

  let count: i64 = result.try_get("count").map_err(|_| SqlError::CountFail)?;
  Ok(count)
}

#[tracing::instrument(skip(mm, f))]
pub async fn count_on<MC, F>(mm: &ModelManager, f: F) -> Result<i64>
where
  MC: DbBmc,
  F: FnOnce(&mut SelectStatement) -> Result<()>,
{
  // -- Build the query
  let mut query = Query::select();
  query.from(MC::table_ref());
  query.expr(Expr::col(sea_query::Asterisk).count());

  // -- condition from filter
  f(&mut query)?;

  // -- Generate sql and values
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  println!("sql: {}, values: {:?}", sql, values);
  let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);

  // -- Execute the query
  let (count,) = mm.dbx().fetch_one(sqlx_query).await.map_err(|_| SqlError::CountFail)?;
  Ok(count)
}

#[tracing::instrument(skip(mm, filter, list_options))]
pub async fn page<MC, E, F>(mm: &ModelManager, filter: F, list_options: ListOptions) -> Result<PageResult<E>>
where
  MC: DbBmc,
  F: Into<FilterGroups>,
  E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
  E: HasSeaFields,
{
  let filter: FilterGroups = filter.into();
  let total_size = count::<MC, _>(mm, filter.clone()).await?;
  let items = find_many::<MC, E, _>(mm, filter, Some(list_options)).await?;

  Ok(PageResult::new(total_size, items))
}

#[tracing::instrument(skip(mm, id, data))]
pub async fn update_by_id<MC, E>(mm: &ModelManager, id: Id, data: E) -> Result<()>
where
  MC: DbBmc,
  E: HasSeaFields,
{
  let ctx = mm.ctx_ref()?;

  // -- Prep Fields
  let mut fields = data.not_none_sea_fields();
  if MC::has_modification_timestamps() {
    fields = prep_fields_for_update::<MC>(fields, ctx);
  }

  // -- Build query
  let fields = fields.for_sea_update();
  let mut query = Query::update();
  query.table(MC::table_ref()).values(fields).and_where(Expr::col(CommonIden::Id).eq(id.clone()));

  // -- Execute query
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let sqlx_query = sqlx::query_with(&sql, values);
  let count = mm.dbx().execute(sqlx_query).await?;

  // -- Check result
  _check_result::<MC>(count, id)
}

/// 根据过滤条件更新，返回更新的记录数
#[tracing::instrument(skip(mm, filter, data))]
pub async fn update<MC, E, F>(mm: &ModelManager, filter: F, data: E) -> Result<u64>
where
  MC: DbBmc,
  F: Into<FilterGroups>,
  E: HasSeaFields,
{
  let ctx = mm.ctx_ref()?;

  // -- Prep Fields
  let mut fields = data.not_none_sea_fields();
  if MC::has_modification_timestamps() {
    fields = prep_fields_for_update::<MC>(fields, ctx);
  }

  // -- Build query
  let fields = fields.for_sea_update();
  let mut query = Query::update();
  query.table(MC::table_ref()).values(fields);
  let filters: FilterGroups = filter.into();
  let cond: Condition = filters.try_into()?;
  query.cond_where(cond);

  // -- Execute query
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let sqlx_query = sqlx::query_with(&sql, values);
  let count = mm.dbx().execute(sqlx_query).await?;

  Ok(count)
}

#[tracing::instrument(skip(mm, id))]
pub async fn delete_by_id<MC>(mm: &ModelManager, id: Id) -> Result<()>
where
  MC: DbBmc,
{
  let ctx = mm.ctx_ref()?;

  // -- Build query
  let (sql, values) = if MC::use_logical_deletion() {
    // -- Prep Fields
    let mut fields = SeaFields::new(vec![SeaField::new(CommonIden::LogiscalDeletion, true)]);
    if MC::has_modification_timestamps() {
      fields = prep_fields_for_update::<MC>(fields, ctx);
    }

    let fields = fields.for_sea_update();
    Query::update()
      .table(MC::table_ref())
      .values(fields)
      .and_where(Expr::col(CommonIden::Id).eq(id.clone()))
      .build_sqlx(PostgresQueryBuilder)
  } else {
    Query::delete()
      .from_table(MC::table_ref())
      .and_where(Expr::col(CommonIden::Id).eq(id.clone()))
      .build_sqlx(PostgresQueryBuilder)
  };

  // -- Execute query
  // let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let sqlx_query = sqlx::query_with(&sql, values);
  let count = mm.dbx().execute(sqlx_query).await?;

  _check_result::<MC>(count, id)
}

#[tracing::instrument(skip(mm, ids))]
pub async fn delete_by_ids<MC>(mm: &ModelManager, ids: Vec<Id>) -> Result<u64>
where
  MC: DbBmc,
{
  if ids.is_empty() {
    return Ok(0);
  }

  // -- Build query
  let mut query = Query::delete();
  query.from_table(MC::table_ref()).and_where(Expr::col(CommonIden::Id).is_in(ids));

  // -- Execute query
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let sqlx_query = sqlx::query_with(&sql, values);
  let n = mm.dbx().execute(sqlx_query).await?;

  Ok(n)
}

#[tracing::instrument(skip(mm, filter))]
pub async fn delete<MC, F>(mm: &ModelManager, filter: F) -> Result<u64>
where
  MC: DbBmc,
  F: Into<FilterGroups>,
{
  let mut query = Query::delete();
  query.from_table(MC::table_ref());
  let filters: FilterGroups = filter.into();
  let cond: Condition = filters.try_into()?;
  query.cond_where(cond);

  // -- Execute query
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let sqlx_query = sqlx::query_with(&sql, values);
  let n = mm.dbx().execute(sqlx_query).await?;

  Ok(n)
}

// #[tracing::instrument(skip(list_options))]
pub fn compute_list_options<MC>(list_options: Option<ListOptions>) -> Result<ListOptions>
where
  MC: DbBmc,
{
  if let Some(mut list_options) = list_options {
    // Validate the limit.
    if let Some(limit) = list_options.limit {
      if limit > MC::LIST_LIMIT_MAX {
        return Err(SqlError::ListLimitOverMax { max: MC::LIST_LIMIT_MAX, actual: limit });
      } else if limit < 1 {
        return Err(SqlError::ListLimitUnderMin { min: 1, actual: limit });
      }
    } else {
      // Set the default limit if no limit
      list_options.limit = Some(MC::LIST_LIMIT_DEFAULT);
    }
    if let Some(page) = list_options.page {
      if page < 1 {
        return Err(SqlError::ListPageUnderMin { min: 1, actual: page });
      }
    }
    Ok(list_options)
  } else {
    // When None, return default
    Ok(ListOptions { limit: Some(MC::LIST_LIMIT_DEFAULT), ..Default::default() })
  }
}

/// Check result
fn _check_result<MC>(count: u64, id: Id) -> Result<()>
where
  MC: DbBmc,
{
  if count == 0 {
    Err(SqlError::EntityNotFound { schema: MC::SCHEMA, entity: MC::TABLE, id })
  } else {
    Ok(())
  }
}
