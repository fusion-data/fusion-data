use fusion_common::ctx::Ctx;
use fusionsql_core::page::Page;
use sea_query::{DeleteStatement, Expr, InsertStatement, IntoIden, SelectStatement, UpdateStatement, WithQuery};
#[cfg(any(feature = "with-postgres", feature = "with-sqlite"))]
use sea_query_binder::{SqlxBinder, SqlxValues};

use crate::{
  Result, SqlError,
  base::{BmcConfig, CommonIden, TimestampIden},
  field::{SeaField, SeaFields},
  store::dbx::DbxProvider,
};

pub fn build_sqlx_for_update(dbx_type: &DbxProvider, query: UpdateStatement) -> (String, SqlxValues) {
  match dbx_type {
    #[cfg(feature = "with-postgres")]
    DbxProvider::Postgres => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      (sql, values)
    }
    #[cfg(feature = "with-sqlite")]
    DbxProvider::Sqlite => {
      let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
      (sql, values)
    }
  }
}

pub fn build_sqlx_for_select(dbx_type: &DbxProvider, query: SelectStatement) -> (String, SqlxValues) {
  match dbx_type {
    #[cfg(feature = "with-postgres")]
    DbxProvider::Postgres => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      (sql, values)
    }
    #[cfg(feature = "with-sqlite")]
    DbxProvider::Sqlite => {
      let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
      (sql, values)
    }
  }
}

pub fn build_sqlx_for_query(dbx_type: &DbxProvider, query: WithQuery) -> (String, SqlxValues) {
  match dbx_type {
    #[cfg(feature = "with-postgres")]
    DbxProvider::Postgres => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      (sql, values)
    }
    #[cfg(feature = "with-sqlite")]
    DbxProvider::Sqlite => {
      let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
      (sql, values)
    }
  }
}

pub fn build_sqlx_for_delete(dbx_type: &DbxProvider, query: DeleteStatement) -> (String, SqlxValues) {
  match dbx_type {
    #[cfg(feature = "with-postgres")]
    DbxProvider::Postgres => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      (sql, values)
    }
    #[cfg(feature = "with-sqlite")]
    DbxProvider::Sqlite => {
      let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
      (sql, values)
    }
  }
}

pub fn build_sqlx_for_insert(dbx_type: &DbxProvider, query: InsertStatement) -> (String, SqlxValues) {
  match dbx_type {
    #[cfg(feature = "with-postgres")]
    DbxProvider::Postgres => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      (sql, values)
    }
    #[cfg(feature = "with-sqlite")]
    DbxProvider::Sqlite => {
      let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
      (sql, values)
    }
  }
}

/// This method must be called when a model controller intends to create its entity.
pub fn prep_fields_for_create(bmc_config: &BmcConfig, mut fields: SeaFields, ctx: &Ctx) -> SeaFields {
  fill_creations(bmc_config, &mut fields, ctx);
  if bmc_config.id_generated_by_db {
    fields = SeaFields::new(fields.into_iter().filter(|f| f.iden.to_string() != bmc_config.column_id).collect());
  }

  fields
}

/// This method must be called when a Model Controller plans to update its entity.
pub fn prep_fields_for_update(bmc_config: &BmcConfig, mut fields: SeaFields, ctx: &Ctx) -> SeaFields {
  fill_modifications(bmc_config, &mut fields, ctx);
  fields
}

pub fn clear_id_from_fields(bmc_config: &BmcConfig, fields: SeaFields) -> SeaFields {
  let mut fields = fields.into_vec();
  fields.retain(|f| f.iden != bmc_config.column_id.into_iden());
  SeaFields::new(fields)
}

/// Update the creations info for create
/// (e.g., created_by, created_at, and updated_by, updated_at will be updated with the same values)
fn fill_creations(bmc_config: &BmcConfig, fields: &mut SeaFields, ctx: &Ctx) {
  if bmc_config.has_owner_id {
    fields.push(SeaField::new(CommonIden::OwnerId.into_iden(), ctx.user_id()));
  }
  if bmc_config.has_created_by && !fields.exists(TimestampIden::CreatedBy.into_iden()) {
    fields.push(SeaField::new(TimestampIden::CreatedBy, ctx.user_id()));
  }
  if bmc_config.has_created_at && !fields.exists(TimestampIden::CreatedAt.into_iden()) {
    fields.push(SeaField::new(TimestampIden::CreatedAt, *ctx.req_time()));
  }
}

/// Update the modifications info only for update.
/// (.e.g., only updated_by, updated_at will be updated)
fn fill_modifications(bmc_config: &BmcConfig, fields: &mut SeaFields, ctx: &Ctx) {
  if bmc_config.has_updated_by && !fields.exists(TimestampIden::UpdatedBy.into_iden()) {
    fields.push(SeaField::new(TimestampIden::UpdatedBy, ctx.user_id()));
  }
  if bmc_config.has_updated_at && !fields.exists(TimestampIden::UpdatedAt.into_iden()) {
    fields.push(SeaField::new(TimestampIden::UpdatedAt, *ctx.req_time()));
  }
}

pub fn compute_page(bmc_config: &BmcConfig, page: Option<Page>) -> Result<Page> {
  if let Some(mut page) = page {
    // Validate the limit.
    if let Some(limit) = page.limit {
      if limit > bmc_config.list_limit_max {
        return Err(SqlError::ListLimitOverMax { max: bmc_config.list_limit_max, actual: limit });
      } else if limit < 1 {
        return Err(SqlError::ListLimitUnderMin { min: 1, actual: limit });
      }
    } else {
      // Set the default limit if no limit
      page.limit = Some(bmc_config.list_limit_default);
    }
    if let Some(page) = page.page
      && page < 1
    {
      return Err(SqlError::ListPageUnderMin { min: 1, actual: page });
    }
    if page.order_bys.is_none() || page.order_bys.iter().any(|o| o.is_empty()) {
      page.order_bys = bmc_config.order_bys.as_ref().map(Into::into);
    }
    Ok(page)
  } else {
    // When None, return default
    Ok(Page {
      limit: Some(bmc_config.list_limit_default),
      order_bys: bmc_config.order_bys.as_ref().map(Into::into),
      ..Default::default()
    })
  }
}

/// 检查 sql execute 语句后受影响的数量
pub fn check_number_of_affected(bmc_config: &BmcConfig, expect_n: usize, return_n: u64) -> Result<u64> {
  // -- Check result
  if return_n as usize != expect_n {
    Err(SqlError::EntityNotFound {
      schema: bmc_config.schema,
      entity: bmc_config.table,
      id: 0.into(), // Using 0 because multiple IDs could be not found, you may want to improve error handling here
    })
  } else {
    Ok(return_n)
  }
}

pub fn fill_update_statement(bmc_config: &BmcConfig, stmt: &mut UpdateStatement) {
  if bmc_config.use_logical_deletion {
    stmt.and_where(Expr::col(CommonIden::LogicalDeletion).is_null());
  }
}

pub fn fill_select_statement(bmc_config: &BmcConfig, stmt: &mut SelectStatement) {
  if bmc_config.use_logical_deletion {
    stmt.and_where(Expr::col(CommonIden::LogicalDeletion).is_null());
  }
}

pub fn fill_delete_statement(bmc_config: &BmcConfig, stmt: &mut DeleteStatement) {
  if bmc_config.use_logical_deletion {
    stmt.and_where(Expr::col(CommonIden::LogicalDeletion).is_null());
  }
}
