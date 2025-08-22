use std::time::Duration;

use modelsql::{ModelManager, SqlError, base::DbBmc, generate_pg_bmc_common, generate_pg_bmc_filter};

use crate::model::{DistributedLockEntity, DistributedLockFilter, DistributedLockForInsert, DistributedLockForUpdate};

pub struct DistributedLockBmc;

impl DbBmc for DistributedLockBmc {
  const TABLE: &str = "distributed_lock";
  const ID_GENERATED_BY_DB: bool = false;
}

generate_pg_bmc_common!(
  Bmc: DistributedLockBmc,
  Entity: DistributedLockEntity,
  ForUpdate: DistributedLockForUpdate,
  ForInsert: DistributedLockForInsert,
);

generate_pg_bmc_filter!(
  Bmc: DistributedLockBmc,
  Entity: DistributedLockEntity,
  Filter: DistributedLockFilter,
  ForUpdate: DistributedLockForUpdate,
);

impl DistributedLockBmc {
  /// 尝试获取或更新分布式锁
  ///
  /// 实现高频心跳 + 低频 token 递增机制：
  /// - INSERT: token 由 bigserial 自动生成
  /// - UPDATE: 根据条件递增 token (锁过期、节点变更、超过递增周期)
  /// - 围栏令牌机制防止旧主干扰
  ///
  /// # 参数
  /// - `mm`: 模型管理器
  /// - `id`: 锁的唯一标识符
  /// - `value`: 锁持有者标识（如节点ID）
  /// - `ttl`: 锁的超时时间（TTL）
  /// - `token_increment_interval`: token 递增周期
  ///
  /// # 返回值
  /// - `Some(entity)`: 成功获取或更新锁
  /// - `None`: 锁被其他节点持有且未过期
  pub async fn try_acquire_or_update(
    mm: &ModelManager,
    id: &str,
    value: &str,
    ttl: &Duration,
    token_increment_interval: &Duration,
  ) -> Result<Option<DistributedLockEntity>, SqlError> {
    let sql = r#"insert into distributed_lock(id, value, locked_at, expires_at)
values ($1, $2, now(), now() + $3)
on conflict (id) do update
set value      = excluded.value,
    expires_at = excluded.expires_at,
    -- 如果接管锁或间隔时间超过阈值 → token + 1
    token      = case
                   when distributed_lock.expires_at < now()
                        or distributed_lock.value <> excluded.value
                        or now() - distributed_lock.locked_at > $4
                   then distributed_lock.token + 1
                   else distributed_lock.token
                 end,
    -- 如果 token 有变化，则更新 locked_at
    locked_at  = case
                   when distributed_lock.expires_at < now()
                        or distributed_lock.value <> excluded.value
                        or now() - distributed_lock.locked_at > $4
                   then now()
                   else distributed_lock.locked_at
                 end
where distributed_lock.expires_at < now()
   or distributed_lock.value = excluded.value
returning *"#;
    let db = mm.dbx().db_postgres()?;
    let item = sqlx::query_as::<_, DistributedLockEntity>(sql)
      .bind(id)
      .bind(value)
      .bind(ttl)
      .bind(token_increment_interval)
      .fetch_optional(&db)
      .await?;
    Ok(item)
  }

  /// 释放分布式锁
  ///
  /// 只有锁的持有者（value 匹配）才能释放锁
  ///
  /// # 参数
  /// - `mm`: 模型管理器
  /// - `id`: 锁的唯一标识符
  /// - `value`: 锁持有者标识
  ///
  /// # 返回值
  /// - `true`: 成功释放锁
  /// - `false`: 锁不存在或不是当前持有者
  pub async fn release_leadership(mm: &ModelManager, id: &str, value: &str) -> Result<bool, SqlError> {
    let sql = r#"delete from distributed_lock where id = $1 and value = $2"#;
    let db = mm.dbx().db_postgres()?;
    let rows = sqlx::query(sql).bind(id).bind(value).execute(&db).await?;
    Ok(rows.rows_affected() == 1)
  }

  /// 验证分布式锁参数配置的安全性
  ///
  /// 根据文档设计，确保 TTL > token_increment_interval，推荐 TTL >= 2 * token_increment_interval
  ///
  /// # 参数
  /// - `ttl`: 锁的超时时间
  /// - `token_increment_interval`: token 递增周期
  ///
  /// # 返回值
  /// - `Ok(())`: 参数配置安全
  /// - `Err(String)`: 参数配置有风险，返回错误信息
  pub fn validate_lock_config(ttl: Duration, token_increment_interval: Duration) -> Result<(), String> {
    if ttl <= token_increment_interval {
      return Err(format!(
        "不安全的配置：TTL({:?}) 必须大于 token_increment_interval({:?})",
        ttl, token_increment_interval
      ));
    }

    if ttl < token_increment_interval * 2 {
      return Err(format!(
        "建议配置：TTL({:?}) 应该 >= 2 * token_increment_interval({:?}) 以确保围栏令牌安全",
        ttl, token_increment_interval
      ));
    }

    Ok(())
  }

  /// 获取推荐的分布式锁配置
  ///
  /// 基于文档建议返回安全的参数配置
  ///
  /// # 返回值
  /// - `(ttl, token_increment_interval, heartbeat_interval)`: 推荐配置
  pub fn get_recommended_config() -> (Duration, Duration, Duration) {
    let ttl = Duration::from_secs(60); // TTL: 60秒
    let token_increment_interval = Duration::from_secs(20); // token递增周期: 20秒
    let heartbeat_interval = Duration::from_secs(10); // 心跳频率: 10秒

    (ttl, token_increment_interval, heartbeat_interval)
  }
}
