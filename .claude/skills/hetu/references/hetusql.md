# hetusql

SQL/ORM 层：Entity 定义、Fields 宏、FilterNodes、BMC 模式、分页、事务。

## Imports

```rust
// 核心
use hetusql::{ModelManager, DbConfig, DbxProvider};
use hetusql::base::{DbBmc, BmcConfig, crud_fns};
use hetusql::store::{Dbx, create_dbx};

// 字段和过滤
use hetusql::field::{Fields, HasFields, SeaFields, SeaField};
use hetusql_core::filter::{FilterNode, FilterGroups, OpValString, OpValInt64, OpValDateTime};
use hetusql_macros::{Fields, FilterNodes};

// 分页
use hetusql_core::page::{Page, Paged, PageResult, OrderBys, OrderBy};

// ID 类型
use hetusql_core::id::Id;
```

## Entity 定义

### 使用 Fields 宏

```rust
use hetusql_macros::Fields;
use serde::{Deserialize, Serialize};
use hetus::common::time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, Fields)]
#[hetusql(table = "users", schema = "public")]
pub struct UserEntity {
    pub id: i64,
    pub tenant_id: i64,
    pub username: String,
    pub email: String,

    #[field(ignore)]  // 忽略字段（不参与查询）
    pub password_hash: String,

    #[field(name = "created_at")]  // 重命名字段
    pub created: OffsetDateTime,

    pub updated_at: Option<OffsetDateTime>,
    pub logical_deletion: Option<OffsetDateTime>,
}
```

### Fields Trait

```rust
// 自动实现
pub trait HasFields {
    fn field_names() -> &'static [&'static str];
    fn field_metas() -> &'static FieldMetas;
}

// 使用
let names = UserEntity::field_names();
// ["id", "tenant_id", "username", "email", "created_at", ...]

// 转换为 SeaFields（用于查询）
let user = UserEntity { ... };
let fields: SeaFields = user.not_none_sea_fields();  // 只包含非 None 字段
```

## Filters

### OpVal 操作符

```rust
use hetusql_core::filter::*;

// 字符串操作
OpValString::Eq("value")           // =
OpValString::Ne("value")           // !=
OpValString::Contains("sub")       // LIKE '%sub%'
OpValString::StartsWith("pre")     // LIKE 'pre%'
OpValString::EndsWith("suf")       // LIKE '%suf'
OpValString::In(vec!["a", "b"])    // IN ('a', 'b')

// 数值操作
OpValInt64::Eq(100)
OpValInt64::Ne(100)
OpValInt64::Gt(100)                // >
OpValInt64::Gte(100)               // >=
OpValInt64::Lt(100)                // <
OpValInt64::Lte(100)               // <=
OpValInt64::In(vec![1, 2, 3])

// 时间操作
OpValDateTime::Eq(dt)
OpValDateTime::Gte(dt)
OpValDateTime::Lte(dt)
OpValDateTime::Between(start, end)
```

### FilterNode

```rust
use hetusql_core::filter::FilterNode;

// 创建单个节点
let node = FilterNode::new("id", OpValInt64::Gt(100));

// 简化写法
let node: FilterNode = ("status", "active").into();
let node: FilterNode = ("created_at", OpValDateTime::Gte(now_offset())).into();
```

### FilterGroups

```rust
use hetusql_core::filter::FilterGroups;

// AND/OR 组合
let filters: FilterGroups = vec![
    vec![
        ("id", OpValInt64::Gt(0)),      // id > 0
        ("status", "active"),            // AND status = 'active'
    ],
    // OR
    vec![
        ("name", OpValString::Contains("test")),
    ],
].into();
// SQL: (id > 0 AND status = 'active') OR (name LIKE '%test%')
```

### FilterNodes 宏

```rust
use hetusql_macros::FilterNodes;
use hetusql_core::filter::{OpValString, OpValInt64};

#[derive(Debug, Default, Deserialize, FilterNodes)]
pub struct UserFilter {
    pub id: Option<OpValInt64>,
    pub tenant_id: Option<OpValInt64>,
    pub username: Option<OpValString>,
    pub email: Option<OpValString>,

    #[hetusql(cast_as = "uuid")]
    pub external_id: Option<OpValString>,
}

// 转换为 FilterGroups
let filter = UserFilter {
    id: Some(OpValInt64::Gt(100)),
    username: Some(OpValString::Contains("john")),
    ..Default::default()
};
let nodes: FilterGroups = filter.into();
```

## BMC 模式

### 定义 BMC

```rust
use hetusql::base::{DbBmc, BmcConfig};

pub struct UserBmc;

impl DbBmc for UserBmc {
    fn _bmc_config() -> &'static BmcConfig {
        &BmcConfig::new_table("users")
            .with_column_id("id")
            .with_id_generated_by_db(true)      // ID 由数据库生成
            .with_has_created_by(true)          // 有 created_by 字段
            .with_has_created_at(true)          // 有 created_at 字段
            .with_has_updated_by(true)          // 有 updated_by 字段
            .with_has_updated_at(true)          // 有 updated_at 字段
            .with_use_logical_deletion(true)    // 使用逻辑删除
            .with_has_owner_id(true)            // 有 owner_id（用于多租户）
    }
}
```

### CRUD 函数

```rust
use hetusql::base::crud_fns;
use hetusql_core::id::Id;

// 创建
pub async fn create(mm: &ModelManager, user: UserForCreate) -> Result<i64> {
    crud_fns::create::<UserBmc, _>(mm, user).await
}

// 查询单个
pub async fn get(mm: &ModelManager, id: i64) -> Result<Option<UserEntity>> {
    crud_fns::get_by_id::<UserBmc, UserEntity, _>(mm, Id::I64(id)).await
}

// 列表查询
pub async fn list(
    mm: &ModelManager,
    filter: Option<UserFilter>,
    page: Option<Page>,
) -> Result<PageResult<UserEntity>> {
    crud_fns::list::<UserBmc, UserEntity, _>(
        mm,
        filter.map(|f| f.into()),
        page,
    ).await
}

// 更新
pub async fn update(mm: &ModelManager, id: i64, user: UserForUpdate) -> Result<()> {
    crud_fns::update_by_id::<UserBmc, _>(mm, Id::I64(id), user).await
}

// 删除（逻辑删除或物理删除）
pub async fn delete(mm: &ModelManager, id: i64) -> Result<()> {
    crud_fns::delete_by_id::<UserBmc>(mm, Id::I64(id)).await
}

// 自定义查询
pub async fn find_by_email(mm: &ModelManager, email: &str) -> Result<Option<UserEntity>> {
    let filter = UserFilter {
        email: Some(OpValString::Eq(email.to_string())),
        ..Default::default()
    };
    let result = list(mm, Some(filter), Some(Page::single())).await?;
    Ok(result.result.into_iter().next())
}
```

## 分页

### Page 请求

```rust
use hetusql_core::page::{Page, OrderBys, OrderBy};

let page = Page {
    page: Some(1),           // 页码（从 1 开始）
    limit: Some(20),         // 每页数量
    offset: None,            // 或使用 offset
    order_bys: Some(OrderBys::from(vec![
        OrderBy::Desc("created_at"),
        OrderBy::Asc("id"),
    ])),
};

// 单条查询
let single = Page::single();

// 无分页
let all = Page::all();
```

### PageResult 响应

```rust
use hetusql_core::page::{Paged, PageResult};

pub struct PageResult<T> {
    pub page: Paged,
    pub result: Vec<T>,
}

pub struct Paged {
    pub total: u64,      // 总记录数
    pub has_more: bool,  // 是否有更多
}

// 使用
let result = UserBmc::list(&mm, Some(filter), Some(page)).await?;
println!("Total: {}, Has more: {}", result.page.total, result.page.has_more);
for user in result.result {
    println!("{:?}", user);
}
```

## 事务

### 闭包式事务（推荐）

```rust
mm.transaction(|mm| async move {
    // 在事务中执行多个操作
    UserBmc::create(&mm, user).await?;
    ProfileBmc::create(&mm, profile).await?;

    // 返回值会自动提交
    Ok(())
}).await?;
```

### 手动事务

```rust
// 开始事务
let mm_txn = mm.txn_cloned();
mm_txn.dbx().begin_txn().await?;

// 执行操作
UserBmc::create(&mm_txn, user).await?;

// 提交或回滚
mm_txn.dbx().commit_txn().await?;
// 或
mm_txn.dbx().rollback_txn().await?;
```

### 事务嵌套

```rust
// 使用 savepoint
mm.transaction(|mm| async move {
    // 外层操作
    UserBmc::create(&mm, user).await?;

    // 内层事务（savepoint）
    mm.transaction(|mm| async move {
        ProfileBmc::create(&mm, profile).await?;
        Ok(())
    }).await?;

    Ok(())
}).await?;
```

## 多租户

### 使用过滤器拦截器

```rust
let mm = mm.with_filter_interceptor(|bmc_config, ctx, filters| {
    if bmc_config.has_owner_id {
        // 自动添加 tenant_id 过滤
        filters.add("tenant_id", OpValInt64::Eq(ctx.tenant_id()));
    }
    Ok(filters)
});
```

### 实体设计

```rust
// 有租户隔离的表
#[derive(Fields)]
#[hetusql(table = "users")]
pub struct UserEntity {
    pub id: i64,
    pub tenant_id: i64,  // 租户 ID
    // ...
}

// 无租户隔离的表（系统级）
#[derive(Fields)]
#[hetusql(table = "system_config")]
pub struct SystemConfigEntity {
    pub id: i64,
    // 无 tenant_id
}
```

## 4-文件结构

| 文件 | 用途 | 位置 |
|------|------|------|
| `{entity}_entity.rs` | 数据库实体定义 | `core/` |
| `{entity}_model.rs` | 过滤器、请求/响应类型 | `core/` |
| `{entity}_bmc.rs` | CRUD 操作 | `bin/infra/bmc/` |
| `{entity}_svc.rs` | 业务逻辑 | `bin/domain/` |

## Best Practices

1. **使用 BMC**: 所有数据库操作都通过 BMC 层，保持一致性
2. **事务**: 使用闭包式事务自动管理提交/回滚
3. **多租户**: 使用 `with_filter_interceptor` 自动添加租户过滤
4. **分页**: 始终使用 `Page` 和 `PageResult` 标准化分页
5. **逻辑删除**: 使用 `logical_deletion` 字段而非物理删除

## Examples from Codebase

- `crates/hetusql/src/model_manager.rs` - ModelManager 实现
- `crates/hetusql-macros/src/derives_field/` - Fields 宏实现
- `jieyuan/jieyuan-server/src/user/user_bmc.rs` - BMC 示例
