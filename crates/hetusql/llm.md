# hetusql LLM 使用指南

> hetusql ORM 框架参考文档，基于 sea-query + sqlx

## 架构概览

```
model     → Entity, Filter, ForCreate, ForUpdate
bmc       → BMC 层（数据访问，宏生成 CRUD）
service   → Service 层（业务逻辑，调用 BMC）
```

---

## 类型速查

### BmcConfig

```rust
BmcConfig::new_table("table_name")
  .with_id_generated_by_db(bool)
  .with_has_created_by(bool)
  .with_has_created_at(bool)
  .with_has_updated_by(bool)
  .with_has_updated_at(bool)
  .with_use_logical_deletion(bool)
  .with_list_limit_default(u64)
  .with_list_limit_max(u64)
```

### DbBmc Trait

```rust
pub trait DbBmc {
  fn _bmc_config() -> &'static BmcConfig;
}
```

### ModelManager

```rust
ModelManager::new()
mm.dbx()                     // 获取 Dbx
mm.get_txn_clone()           // 事务克隆
mm.ctx()                     // 获取上下文
```

---

## 标准模板

### Model 模板

```rust
// model/user.rs
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, hetusql::Fields)]
pub struct User {
  pub id: i64,
  pub name: String,
  pub email: String,
  pub status: i32,
  pub created_by: i64,
  pub created_at: DateTime<FixedOffset>,
  pub updated_by: Option<i64>,
  pub updated_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Clone, Deserialize, Fields)]
pub struct UserForCreate {
  pub name: String,
  pub email: String,
  pub status: i32,
}

#[derive(Debug, Clone, Deserialize, Fields)]
pub struct UserForUpdate {
  pub name: Option<String>,
  pub email: Option<String>,
  pub status: Option<i32>,
}

#[derive(Debug, Default, Deserialize, hetusql::FilterNodes)]
pub struct UserFilter {
  pub id: Option<OpValInt64>,
  pub name: Option<OpValString>,
  pub email: Option<OpValString>,
  pub status: Option<OpValInt32>,
  pub created_at: Option<OpValDateTime>,
}
```

### BMC 模板

```rust
// bmc/user_bmc.rs
use hetusql::{DbBmc, BmcConfig, ModelManager, SqlError};
use hetusql::macros::{generate_pg_bmc_common, generate_pg_bmc_filter};
use std::sync::OnceLock;

pub struct UserBmc;

impl DbBmc for UserBmc {
  fn _bmc_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
      BmcConfig::new_table("user")
        .with_list_limit_default(20)
        .with_list_limit_max(100)
    })
  }
}

generate_pg_bmc_common!(
  Bmc: UserBmc,
  Entity: User,
  ForCreate: UserForCreate,
  ForUpdate: UserForUpdate,
);

generate_pg_bmc_filter!(
  Bmc: UserBmc,
  Entity: User,
  Filter: UserFilter,
);

impl UserBmc {
  // 自定义方法
  pub async fn find_by_email(mm: &ModelManager, email: &str) -> Result<Option<User>, SqlError> {
    let filter = UserFilter {
      email: Some(OpValString::eq(email)),
      ..Default::default()
    };
    Self::find_unique(mm, vec![filter]).await
  }
}
```

### Service 模板

```rust
// service/user_svc.rs
use hetusql::ModelManager;

#[derive(Clone)]
pub struct UserService {
  mm: ModelManager,
}

impl UserService {
  pub fn new(mm: ModelManager) -> Self {
    Self { mm }
  }

  pub async fn create(&self, input: UserForCreate) -> Result<i64, DataError> {
    UserBmc::create(&self.mm, input)
      .await
      .map_err(DataError::from)
  }

  pub async fn get_by_id(&self, id: i64) -> Result<User, DataError> {
    UserBmc::get_by_id(&self.mm, id)
      .await
      .map_err(DataError::from)?
      .ok_or_else(|| DataError::not_found("用户不存在"))
  }

  pub async fn update(&self, id: i64, input: UserForUpdate) -> Result<(), DataError> {
    UserBmc::update_by_id(&self.mm, id, input)
      .await
      .map_err(DataError::from)
  }

  pub async fn delete(&self, id: i64) -> Result<(), DataError> {
    UserBmc::delete_by_id(&self.mm, id)
      .await
      .map_err(DataError::from)
  }

  pub async fn page(&self, filter: UserFilter, page: Page) -> Result<PageResult<User>, DataError> {
    UserBmc::page(&self.mm, vec![filter], page)
      .await
      .map_err(DataError::from)
  }
}
```

---

## BMC API 参考

### generate_pg_bmc_common! 生成的方法

| 方法         | 签名                                                         | 说明           |
| ------------ | ------------------------------------------------------------ | -------------- |
| create       | `(mm, UserForCreate) -> Result<i64, SqlError>`               | 创建，返回 ID  |
| insert       | `(mm, UserForCreate) -> Result<User, SqlError>`              | 创建，返回实体 |
| get_by_id    | `(mm, id) -> Result<Option<User>, SqlError>`                 | 按 ID 查询     |
| find_by_id   | `(mm, id) -> Result<Option<User>, SqlError>`                 | 按 ID 查询     |
| find_many    | `(mm, filters, page) -> Result<Vec<User>, SqlError>`         | 列表查询       |
| find_unique  | `(mm, filters) -> Result<Option<User>, SqlError>`            | 唯一查询       |
| list         | `(mm, filters, page) -> Result<Vec<User>, SqlError>`         | 列表查询       |
| exists       | `(mm, filters) -> Result<bool, SqlError>`                    | 是否存在       |
| update_by_id | `(mm, id, UserForUpdate) -> Result<(), SqlError>`            | 按 ID 更新     |
| update       | `(mm, filters, id, UserForUpdate) -> Result<User, SqlError>` | 条件更新       |
| delete_by_id | `(mm, id) -> Result<(), SqlError>`                           | 按 ID 删除     |
| delete       | `(mm, filters) -> Result<u64, SqlError>`                     | 条件删除       |
| count        | `(mm, filters) -> Result<u64, SqlError>`                     | 计数           |
| page         | `(mm, filters, Page) -> Result<PageResult<User>, SqlError>`  | 分页查询       |

---

## Filter 系统

### FilterNodes 宏

```rust
#[derive(FilterNodes)]
pub struct UserFilter {
  pub name: Option<OpValString>,
  pub status: Option<OpValInt32>,
}
```

### FilterGroup / FilterGroups

```rust
// AND: (name = 'John' AND status = 1)
let group = FilterGroup(vec![
  ("name", OpValString::eq("John")).into(),
  ("status", OpValInt32::eq(1)).into(),
]);

// OR: ((name = 'John') OR (name = 'Jane'))
let groups = FilterGroups(vec![
  FilterGroup(vec![("name", OpValString::eq("John")).into()]),
  FilterGroup(vec![("name", OpValString::eq("Jane")).into()]),
]);

// 使用
UserBmc::find_many(mm, vec![groups], None).await?;
```

### OpVal 类型

| 类型                 | 字段                                                                                             | 便捷方法                                                           |
| -------------------- | ------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------ |
| OpValString          | eq, not, in\_, not_in, lt, lte, gt, gte, contains, starts_with, ends_with, null, empty, +ci 变体 | `eq(s)`, `in_(vec)`, `contains(s)`, `starts_with(s)`, `null(bool)` |
| OpValInt32/Int64     | eq, not, in\_, not_in, lt, lte, gt, gte, null                                                    | `eq(n)`, `in_(vec)`, `gt(n)`                                       |
| OpValFloat32/Float64 | eq, not, in\_, not_in, lt, lte, gt, gte, null                                                    | `eq(n)`, `lt(n)`                                                   |
| OpValDateTime        | eq, not, in\_, not_in, lt, lte, gt, gte, null                                                    | `eq(dt)`, `gt(dt)`                                                 |
| OpValBool            | eq                                                                                               | `eq(bool)`                                                         |
| OpValUuid            | eq, not, in\_, not_in, null                                                                      | `eq(uuid)`                                                         |

---

## 分页查询

### Page

```rust
pub struct Page {
  pub page: Option<u64>,        // 页码（从 1 开始）
  pub limit: Option<u64>,       // 条数
  pub offset: Option<u64>,      // 偏移
  pub order_bys: Option<OrderBys>,
}
```

### OrderBy

```rust
OrderBy("name")          // ASC
OrderBy("!name")         // DESC
OrderBys::from(vec!["name", "!created_at"])
```

### PageResult

```rust
pub struct PageResult<T> {
  pub page: Paged,      // { total: u64, has_more: bool }
  pub result: Vec<T>,
}
```

### 使用

```rust
let page = Page {
  page: Some(1),
  limit: Some(20),
  ..Default::default()
};
let result = UserBmc::page(&mm, vec![filter], page).await?;
```

---

## 错误处理

> **注意**: `SqlError -> DataError` 和 `DbxError -> DataError` 转换已在 hetusql 中实现。

### SqlError -> DataError 转换

```rust
// hetusql 已内置 From<SqlError> for hetu_common::DataError
// 自动映射规则：
// - SqlError::Unauthorized -> DataError::unauthorized
// - SqlError::EntityNotFound -> DataError::not_found
// - SqlError::NotFound -> DataError::not_found
// - SqlError::UniqueViolation -> DataError::conflicted
// - SqlError::UserAlreadyExists -> DataError::conflicted
// - SqlError::InvalidArgument -> DataError::bad_request
// - SqlError::ListLimitOverMax/UnderMin/PageUnderMin -> DataError::bad_request
// - 其他 -> DataError::server_error 或 DataError::internal
```

### DbxError -> DataError 转换

```rust
// hetusql 已内置 From<DbxError> for hetu_common::DataError
// 自动映射为 DataError::server_error
```

### BMC 层

```rust
// BMC 层返回 SqlError
pub async fn get_by_id(mm: &ModelManager, id: i64) -> Result<Option<User>, SqlError>;
```

### Service 层

```rust
use hetu_common::DataError;

// 直接使用 ? 运算符，自动转换 SqlError -> DataError
pub async fn get_by_id(&self, id: i64) -> Result<User, DataError> {
  UserBmc::get_by_id(&self.mm, id)
    .await?  // SqlError 自动转换为 DataError
    .ok_or_else(|| DataError::not_found("用户不存在"))
}

// 事务处理
pub async fn create_with_txn(&self, input: UserForCreate) -> Result<i64, DataError> {
  let mm = self.mm.get_txn_clone();
  mm.dbx().begin_txn().await?;  // DbxError 自动转换为 DataError

  let id = UserBmc::create(&mm, input).await?;

  mm.dbx().commit_txn().await?;
  Ok(id)
}
```

---

## 事务处理

```rust
// 获取事务克隆
let mm = self.mm.get_txn_clone();
mm.dbx().begin_txn().await?;

// 执行操作
TenantUserBmc::delete(&mm, vec![filter]).await?;
TenantUserBmc::create(&mm, input).await?;

// 提交或回滚
match result {
  Ok(_) => mm.dbx().commit_txn().await?,
  Err(e) => { mm.dbx().rollback_txn().await?; return Err(e); }
}
```

---

## 命名约定

| 实体 | User |
| BMC | UserBmc |
| Service | UserService |
| Filter | UserFilter |
| ForCreate | UserForCreate |
| ForUpdate | UserForUpdate |
| 文件 | model/user.rs, bmc/user_bmc.rs, service/user_svc.rs |

---

## 宏参考

```rust
// 生成 CRUD 方法
generate_pg_bmc_common!(
  Bmc: UserBmc,
  Entity: User,
  ForCreate: UserForCreate,
  ForUpdate: UserForUpdate,
);

// 生成过滤查询方法
generate_pg_bmc_filter!(
  Bmc: UserBmc,
  Entity: User,
  Filter: UserFilter,
);

// 派生宏
#[derive(Fields)]        // UserForCreate, UserForUpdate
#[derive(FilterNodes)]   // UserFilter
#[derive(FromRow)]       // User (sqlx)
```

---

**版本**: hetusql v0.1.0+
