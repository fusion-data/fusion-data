# Hetumind 数据库设计

## 1. 数据库架构概述

Hetumind 使用 PostgreSQL 作为主数据库，采用现代化的字段命名规范，支持高并发和大规模数据处理。

### 1.1 设计原则

- **字段命名规范**: 使用 `kind` 替换 `type`，`created_at` 替换 `createdAt`，`updated_at` 替换 `updatedAt`
- **数据类型优化**: 使用 UUID 作为主键，JSONB 存储复杂数据
- **索引策略**: 针对查询模式优化索引设计
- **分区策略**: 大表采用时间分区提升性能

### 1.2 技术栈

```toml
[dependencies]
sqlx = { version = "0.8", features = ["postgres", "chrono", "uuid", "json"] }
sea-query = "0.31"
sea-query-postgres = "0.5"
uuid = { version = "1.10", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

## 2. 核心表结构设计

见：[hetumind-ddl.sql](../../scripts/software/postgres/sqls/hetumind-ddl.sql)

## 4. 查询优化策略

### 4.1 常用查询模式

```rust
// 获取工作流及其节点和连接
pub async fn get_workflow_with_details(
    &self,
    workflow_id: WorkflowId,
) -> Result<Option<WorkflowWithDetails>, sqlx::Error> {
    let workflow_query = r#"
        SELECT w.*,
               json_agg(DISTINCT n.*) as nodes,
               json_agg(DISTINCT c.*) as connections
        FROM workflows w
        LEFT JOIN nodes n ON w.id = n.workflow_id
        LEFT JOIN connections c ON w.id = c.workflow_id
        WHERE w.id = $1
        GROUP BY w.id
    "#;

    let row = sqlx::query(workflow_query)
        .bind(workflow_id)
        .fetch_optional(&self.pool)
        .await?;

    // 处理结果...
    Ok(None)
}

// 获取执行统计信息
pub async fn get_execution_stats(
    &self,
    workflow_id: WorkflowId,
    days: i32,
) -> Result<ExecutionStats, sqlx::Error> {
    let stats_query = r#"
        SELECT
            COUNT(*) as total_executions,
            COUNT(CASE WHEN status = 'success' THEN 1 END) as successful_executions,
            COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_executions,
            AVG(EXTRACT(EPOCH FROM (finished_at - started_at)) * 1000) as avg_duration_ms
        FROM executions
        WHERE workflow_id = $1
        AND started_at >= NOW() - INTERVAL '%d days'
    "#;

    let row = sqlx::query(&format!(stats_query, days))
        .bind(workflow_id)
        .fetch_one(&self.pool)
        .await?;

    Ok(ExecutionStats {
        total_executions: row.get("total_executions"),
        successful_executions: row.get("successful_executions"),
        failed_executions: row.get("failed_executions"),
        avg_duration_ms: row.get("avg_duration_ms"),
    })
}
```

### 4.2 性能监控

```rust
pub struct DatabaseMetrics {
    pool: PgPool,
}

impl DatabaseMetrics {
    pub async fn collect_metrics(&self) -> Result<DbMetrics, sqlx::Error> {
        let connection_stats = sqlx::query(
            "SELECT numbackends, xact_commit, xact_rollback FROM pg_stat_database WHERE datname = current_database()"
        )
        .fetch_one(&self.pool)
        .await?;

        let table_stats = sqlx::query(
            "SELECT schemaname, tablename, n_tup_ins, n_tup_upd, n_tup_del FROM pg_stat_user_tables"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(DbMetrics {
            active_connections: connection_stats.get("numbackends"),
            committed_transactions: connection_stats.get("xact_commit"),
            rolled_back_transactions: connection_stats.get("xact_rollback"),
            table_stats: table_stats.into_iter().map(|row| TableStats {
                schema: row.get("schemaname"),
                table: row.get("tablename"),
                inserts: row.get("n_tup_ins"),
                updates: row.get("n_tup_upd"),
                deletes: row.get("n_tup_del"),
            }).collect(),
        })
    }
}
```

## 6. 数据备份和恢复

### 6.1 备份策略

```rust
pub struct BackupManager {
    pool: PgPool,
    backup_config: BackupConfig,
}

#[derive(Debug, Clone)]
pub struct BackupConfig {
    pub backup_interval_hours: u64,
    pub retention_days: u32,
    pub backup_location: String,
    pub compress: bool,
}

impl BackupManager {
    pub async fn create_backup(&self) -> Result<BackupInfo, BackupError> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_file = format!("hetumind_backup_{}.sql", timestamp);

        // 使用 pg_dump 创建备份
        let output = tokio::process::Command::new("pg_dump")
            .args(&[
                "--no-password",
                "--format=custom",
                "--compress=9",
                "--file", &backup_file,
                &self.get_database_url(),
            ])
            .output()
            .await?;

        if !output.status.success() {
            return Err(BackupError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(BackupInfo {
            file_path: backup_file,
            created_at: Utc::now(),
            size_bytes: std::fs::metadata(&backup_file)?.len(),
        })
    }

    pub async fn restore_backup(&self, backup_file: &str) -> Result<(), BackupError> {
        let output = tokio::process::Command::new("pg_restore")
            .args(&[
                "--no-password",
                "--clean",
                "--if-exists",
                "--dbname", &self.get_database_url(),
                backup_file,
            ])
            .output()
            .await?;

        if !output.status.success() {
            return Err(BackupError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(())
    }
}
```

这个数据库设计为 Hetumind 系统提供了高性能、可扩展的数据存储基础，支持大规模工作流执行和实时查询需求。
