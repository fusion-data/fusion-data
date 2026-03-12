---
name: sql-database
description: Fusion-Data ORM patterns, 4-file structure, fusionsql entities, BMC/Service layers, multi-tenancy
---

# FusionSQL ORM

## 4-File Structure

| File | Purpose | Location |
|------|---------|----------|
| `{entity}_entity.rs` | DB entity (Fields, FromRow) | core/ |
| `{entity}_model.rs` | Filters, request/response | core/ |
| `{entity}_bmc.rs` | CRUD operations | bin/infra/bmc/ |
| `{entity}_svc.rs` | Business logic | bin/domain/ |

## Entity Definition
```rust
use fusionsql::{Entity, EnumDef, Fields};
use fusion_common::time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "user_entity")]
pub struct UserEntity {
    pub id: UserId,
    pub tenant_id: TenantId,
    pub name: String,
    pub email: String,
    pub created_at: OffsetDateTime,
    pub logical_deletion: Option<OffsetDateTime>,
}
```

## Filters & Pagination
```rust
use fusionsql::{filter::{OpValString, OpValUuid}, FilterNodes, page::Page};

#[derive(Debug, Clone, Default, Deserialize, FilterNodes)]
pub struct UserFilter {
    pub tenant_id: Option<OpValUuid>,
    pub name: Option<OpValString>,
    pub email: Option<OpValString>,
}

// Usage
let page = Page { page: 1, page_size: 20 };
let users = UserBmc::list(&mm, Some(filter), Some(page)).await?;
```

## BMC Pattern
```rust
impl UserBmc {
    pub async fn get_by_id(mm: &ModelManager, id: UserId) -> Result<Option<UserEntity>, SqlError> {
        let db = mm.db();
        let query = Query::select()
            .columns(UserEntity::columns())
            .from(UserEntity::table())
            .and_where(Expr::col(UserEntity::Id).eq(id))
            .and_where(Expr::col(UserEntity::LogicalDeletion).is_null())
            .to_owned();
        db.fetch_one(query).await
    }
}
```

## Transactions
```rust
let mut tx = mm.dbx().begin_txn().await?;
UserBmc::update(&mm, &user).await?;
tx.commit().await?;
```

## Multi-Tenancy

**Tables with `tenant_id`**: user_entity, project, workflow_entity, credential_entity

**Tables without**: sched_task_instance, sched_agent, system_config

## Related Skills
- `rust-backend`: Component integration, service patterns
