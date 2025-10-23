use fusion_common::time::{OffsetDateTime, now_utc};
use fusionsql::{ModelManager, base::DbBmc};
use sea_query::{Expr, Query};
use sea_query_binder::SqlxBinder;

use super::InvalidAuthTokenIden;

pub struct InvalidAuthTokenBmc;

impl DbBmc for InvalidAuthTokenBmc {
  const TABLE: &'static str = "invalid_auth_token";
  const ID_GENERATED_BY_DB: bool = false;
}

impl InvalidAuthTokenBmc {
  /// 添加无效令牌到黑名单
  pub async fn add_token(
    mm: &ModelManager,
    token: &str,
    expires_at: OffsetDateTime,
  ) -> Result<(), fusion_core::DataError> {
    let (sql, values) = Query::insert()
      .into_table(InvalidAuthTokenIden::Table)
      .columns([InvalidAuthTokenIden::Token, InvalidAuthTokenIden::ExpiresAt])
      .values_panic([token.into(), expires_at.into()])
      .build_sqlx(sea_query::PostgresQueryBuilder);

    mm.dbx()
      .use_postgres(|dbx| async move {
        let sqlx_query = sqlx::query_with(&sql, values);
        dbx.execute(sqlx_query).await
      })
      .await?;

    Ok(())
  }

  /// 检查令牌是否在黑名单中
  pub async fn is_token_invalid(mm: &ModelManager, token: &str) -> Result<bool, fusion_core::DataError> {
    let (sql, values) = Query::select()
      .column(InvalidAuthTokenIden::Token)
      .from(InvalidAuthTokenIden::Table)
      .and_where(Expr::col(InvalidAuthTokenIden::Token).eq(token))
      .and_where(Expr::col(InvalidAuthTokenIden::ExpiresAt).gt(now_utc()))
      .build_sqlx(sea_query::PostgresQueryBuilder);

    let exists = mm
      .dbx()
      .use_postgres(|dbx| async move {
        // Simple existence check - use query_as for proper type handling
        let sqlx_query = sqlx::query_as::<_, (String,)>(&sql);
        let result: Option<(String,)> = dbx.fetch_optional(sqlx_query).await?;
        Ok(result.is_some())
      })
      .await?;

    Ok(exists)
  }

  /// 清理过期的无效令牌
  pub async fn cleanup_expired_tokens(mm: &ModelManager) -> Result<u64, fusion_core::DataError> {
    let (sql, values) = Query::delete()
      .from_table(InvalidAuthTokenIden::Table)
      .and_where(Expr::col(InvalidAuthTokenIden::ExpiresAt).lt(now_utc()))
      .build_sqlx(sea_query::PostgresQueryBuilder);

    let result = mm
      .dbx()
      .use_postgres(|dbx| async move {
        let sqlx_query = sqlx::query_with(&sql, values);
        dbx.execute(sqlx_query).await
      })
      .await?;

    Ok(result)
  }
}
