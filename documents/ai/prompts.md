# 常用提示词

## 编程类

开头的敬语：

你是一名资深的软件架构师，拥有丰富的软件架构设计和开发经验，擅长架构设计、领域驱动建模。
熟悉 Rust Typescript Python Java 等编程语言。

### 提示词增强

- 输出开发落地的优化版提示词，并提供数据库迁移片段与服务方法骨架代码

- 检查上面的方案，进行必要的补充和优化。然后输出是否还有优化建议和需要澄清的问题？

- 只更新设计文档，不要进行任何代码实现。若有疑问或更好的方案，请输出疑问或方案让我审核！

### Coding

- 复用 fusion-xxx 库功能，如： fusion-common, fusion-core, fusion-web, fusion-db, fusionsql 等
- 复用现有的错误处理模式
- 遵循当前项目 Rust 编程最佳实践，对参数使用 snake_case 风格命名，对于 serde 序列化使用 `#[serde(rename_all = "snake_case")]` 注解
- 注意 Arc 的使用以支持并发访问
- 编写单元测试并运行，确保代码实现正确
- 定义结构化的数据类型优先（如：`pub struct Config`）；若需要直接使用 JSON 对象，请使用 `serde_json::json!` 宏进行构造
- 使用 `serde` 的 struct, enum，当手动构建 JSON 字符串时注意序列化方式。比如：`#[serde(rename_all = "snake_case")]` 需要使用 snake_case 风格
- 完成后输出任务总结

### 实现测试

- 按照数据结构（struct, enum）生成格式正确的测试数据，使用 `serde` 时注意序列化风格：`snake_case`、`camelCase`、`UPPER_CASE`、`CamelCase`、`lowercase` 等，或是否有添加 `#[repr(i32)]` 序列化为 i32 数值类型）
- 添加必要的日志输出（使用 log crate）

## 示例

### 示例 1

```markdown
你是一名经验丰富的资深软件开发工程师，擅长架构设计、领域驱动建模，熟悉 Rust、Typescript 等编程语言。仔细阅读 @documents/oauth.md 文档设计，规划任务实现方案目标，完成相关编码工作。

建议任务顺序：

1. 更新 SQL DDL 定义
2. 更新 jieyuan 相关项目
3. 更新 hetumind 相关项目

注意事项：

- 复用现有的数据结构
- 复用现有的错误处理模式
- 遵循当前项目 Rust 编程最佳实践，对参数使用 snake_case 风格命名
- 注意 Arc 的使用以支持并发访问
- 使用 `serde` 的 struct, enum，当手动构建 JSON 字符串时注意序列化方式。比如：`#[serde(rename_all = "snake_case")]` 需要使用 snake_case 风格
- 完成 SQL DDL 文件更新后暂停，由我手动执行 SQL 语句后再通知你继续执行后续任务
- 若有任何疑问或需要澄清的地方，请输出疑问或方案让我审核！
```

### 示例 2

```markdown
任务： 为 @hetumind/hetumind-core/src/workflow/\*.rs 中的所有 struct 添加构造函数和修改方法

## 构造函数规则

生成 pub fn new(...) -> Self 函数，规则如下：

- 参数： 非 Option、非 bool、非容器类型（Vec/HashMap/HashSet）的字段
- 限制： 最多 5 个参数，超过则不生成 new 但生成修改方法
- 初始化： 其他字段使用 Default::default()
- 报告： 执行后打印参数超过 5 个的 struct 名称

## 修改方法规则

### 基本类型方法

// T 或 Option<T> 类型
pub fn with_field_name(mut self, field_name: impl Into<String>) -> Self
// Option<T> 参数类型不包裹 Option

// 数值、bool、enum 类型直接使用原始类型，不用 impl Into<T>

### 容器类型方法

#### Vec 类型：

pub fn with_options<I, V>(mut self, options: I) -> Self
where
I: IntoIterator<Item = V>,
V: Into<Box<NodeProperty>>,
{
self.options = Some(options.into_iter().map(|v| v.into()).collect());
self
}

pub fn add_option(mut self, option: impl Into<Box<NodeProperty>>) -> Self {
self.options.get_or_insert_with(Vec::new).push(option.into());
self
}

#### HashMap 类型：

pub fn with_routing<I, K, V>(mut self, routing: I) -> Self
where
I: IntoIterator<Item = (K, V)>,
K: Into<String>,
V: Into<JsonValue>,
{
self.routing = Some(routing.into_iter().map(|(k, v)| (k.into(), v.into())).collect());
self
}

pub fn add_routing(mut self, key: impl Into<String>, value: impl Into<JsonValue>) -> Self {
self.routing.get_or_insert_with(HashMap::default).insert(key.into(), value.into());
self
}

## 特殊处理

- 包裹类型： Box<T>、Arc<T> 等在方法参数中保持包裹类型
- 已有方法： 检查避免重复生成已存在的 new、with_xxx、add_xxx 方法
- newtype 类型的 struct 不需要添加修改方法
```

### 提示词 3

```markdown
任务： 重构 @hetumind/hetumind-studio/src/ 中所有使用 TypedBuilder 的代码

重构规则

1. 识别目标代码：查找所有使用 `::builder()` 方法的代码
2. 替换构造方式：

- 将 Xxxx::builder().field(value).build() 替换为 Xxxx::new(...).with_field(value)
- 使用 new() 函数的必需参数
- 保留所有 with_xxx() 和 add_xxx() 调用

3. 保持功能不变：确保重构后的代码行为完全一致

执行步骤

1. 扫描 hetumind-nodes/src/ 目录，识别所有使用的 struct 及 new 构造函数和 with_xxx, add_xxx 修改函数
2. 分析每个使用场景，确定对应的 new() 函数参数
3. 逐个文件进行重构替换
4. 编译验证确保无错误

输出要求

- 列出所有修改的文件和位置
- 报告编译结果
```

### 提示词 4 - 实现用户注册、刷新 token、token 解析等接口

```markdown
任务：在 jieyuan 项目中开发以下 API 接口

API 接口规范：

1. POST /api/auth/signup 用户注册接口

   - 请求参数：使用 @jieyuan/jieyuan-core/src/model/auth.rs 中定义的 SignupReq 结构体
     - 必须字段：password（密码）
     - 互斥字段：email 或 phone（必须提供且仅能提供一个）
   - 响应：
     - 成功：HTTP 200 状态码，返回空 JSON 对象
     - 失败：HTTP 400 状态码，返回包含错误信息的 WebError 结构体

2. POST /api/auth/signout 用户登出接口

   - 请求参数：无（从 Authorization 头中提取 token）
   - 响应：
     - 成功：HTTP 200 状态码，返回空 JSON 对象
     - 失败：HTTP 400 状态码，返回包含错误信息的 WebError 结构体

3. POST /api/auth/refresh_token 刷新令牌接口

   - 请求参数：使用 @jieyuan/jieyuan-core/src/model/auth.rs 中定义的 RefreshTokenReq 结构体（包含 refresh_token 字段）
   - 响应：
     - 成功：HTTP 200 状态码，返回 SigninResponse 结构体（复用登录接口的响应格式）
     - 失败：HTTP 401 状态码，返回包含错误信息的 WebError 结构体

4. POST /api/auth/extract_token 令牌解析接口
   - 请求参数：无（从 Authorization 头中提取 token）
   - 响应：
     - 成功：返回 fusion_common::ctx::Ctx 的 JSON 格式数据（使用 extract_ctx 函数解析 token）
     - 失败：HTTP 401 状态码，返回包含错误信息的 WebError 结构体

开发要求：

1. 实现位置：@jieyuan/jieyuan/src/endpoint/api/auth.rs 文件
2. 代码复用：
   - 使用 @jieyuan/jieyuan-core/ 项目中已定义的数据结构（struct/enum）
   - 保持现有错误处理模式
3. 编码规范：
   - 遵循 Rust 最佳实践
   - 参数使用 snake_case 命名风格
   - 合理使用 Arc 以支持并发访问
4. 质量要求：
   - 确保接口线程安全
   - 保持代码风格与项目现有代码一致
   - 实现完整的参数校验逻辑
5. 确保代码编译成功
```

### 提示词 5 - 实现 WebAuth 中间件，通过调用 jieyuan 的 /api/auth/extract_token 接口校验 token 并获取 Ctx 上下文

```markdown
任务：在 @jieyuan/jieyuan-core/src/web/middleware 目录下实现 WebAuth 中间件，通过调用 jieyuan 的 /api/auth/extract_token 接口进行 token 校验并获取 Ctx 上下文。需严格参照 @crates/fusions/fusion-web/src/middleware/web_auth.rs 文件的 Axum 中间件实现方式，具体要求如下：

开发规范：

1. 请求处理流程：
   - 保持与原中间件完全一致的请求处理流程
   - 从请求头的 Authorization 字段提取 token
   - 使用 reqwest 库向 /api/auth/extract_token 接口发送 HTTP 请求进行校验
2. 接口配置：
   - 新增必填参数 api_base_url 作为 WebAuth::new 构造函数的参数
   - 保留原 includes 和 excludes 配置项作为可选参数
3. 代码实现要求：
   - 复用 @jieyuan/jieyuan-core/ 项目中现有的数据结构（struct/enum）
   - 保持现有错误处理模式和响应格式
   - 实现完整的参数校验逻辑
   - 使用 Arc 确保线程安全
   - 遵循 Rust 最佳实践和项目代码风格
4. 质量保证：
   - 确保中间件线程安全
   - 保持与项目现有代码风格一致
   - 通过所有编译检查
   - 参数命名采用 snake_case 风格
5. 错误处理：
   - 与原中间件保持一致的错误处理机制
   - 对 HTTP 请求失败情况实现适当处理
   - 确保错误响应格式与参考实现完全匹配
```

### 提示词 5 - 为 jieyuan 的 User 添加租户功能

```markdown
你是一名经验丰富的资深软件开发工程师。

任务：为 @jieyuan/ 的 User 模型添加租户功能

- 邮箱与手机号全局唯一，作为跨租户的统一身份标识。
- 登录必须明确指定目标租户，客户端需传 `tenant_id`；每个租户登录生成独立令牌，令牌中包含 `tenant_id`。
- `iam_tenant_user.status` 采用两值枚举：`99`（禁用）、`100`（启用）。
- 用户状态规则：
  - 新注册用户初始为 `user.status = 1`（未关联租户）。
  - 仅当用户至少关联一个 `iam_tenant_user.status = 100` 的租户时，方可将 `user.status` 更新为 `100`。
  - 未关联或无启用关联的用户禁止登录。
- 状态联动通过服务层事务维护：增删改用户与租户的关联时，同步计算并更新 `user.status`。
- 超级管理员：`tenant_id = 1` 为平台租户，可执行跨租户的平台级操作；暂不引入平台角色/权限。
- 历史数据迁移不需要（当前系统尚未完成）。
- API 层改动：修改登录接口，强制传入 `tenant_id`；不提供“切换租户”端点。

**数据库修改要求**

- 修改 @scripts/software/postgres/sqls/jieyuan-ddl.sql：
  - 创建多对多关联表 `iam_tenant_user`，字段包含：
    - `tenant_id BIGINT NOT NULL`
    - `user_id BIGINT NOT NULL`
    - `status SMALLINT NOT NULL`（取值仅 `99`、`100`）
    - `created_at TIMESTAMPTZ DEFAULT now()`
    - `updated_at TIMESTAMPTZ DEFAULT now()`
  - 建立复合索引：`(tenant_id, user_id)`。
  - 从 `iam_user` 表移除 `tenant_id` 字段。
  - 外键与级联建议：`tenant_id → iam_tenant(id)`、`user_id → iam_user(id)`，`ON DELETE CASCADE`。

**业务逻辑调整要求**

- 登录流程校验：
  - 校验 `user.status == 100`。
  - 校验 `iam_tenant_user` 存在 `(tenant_id, user_id)` 且 `status == 100`。
  - 令牌携带 `tenant_id` 并作为服务端隔离上下文。
- 用户创建与更新：
  - 创建用户：初始 `user.status = 1`；若同时提交租户关联且 `status = 100`，在事务中建立关联、最终可置 `user.status = 100`。
  - 更新用户：任何关联的新增、删除、禁用、启用均在事务中维护，并重算 `user.status`。
- 租户隔离：
  - 单个用户可关联多个租户。
  - 所有用户操作必须在传入 `tenant_id` 的上下文中执行，并在仓储层/服务层统一强制隔离。

**代码修改**

- 查询模型与仓储：
  - 为 `User` 模型查询参数添加 `tenant_id: Option<OpValInt64>` 支持。
  - 在仓储层实现带 `tenant_id` 的查询与校验（通过连接 `iam_tenant_user`）。
- 关联管理接口：
  - `link_user_to_tenant(user_id, tenant_id, status)`、`unlink_user_from_tenant(...)`。
  - 在事务内维护 `iam_tenant_user` 与 `user.status` 的一致性。
- API：
  - 修改登录接口签名以强制传入 `tenant_id`；生成包含 `tenant_id` 的租户级令牌。
- 测试用例：
  - 注册用户不可登录（无租户关联）。
  - 关联租户启用后可登录。
  - 多租户关联，基于不同 `tenant_id` 的登录校验与令牌隔离。
  - 解除/禁用关联时，回退或维持 `user.status`（根据剩余启用关联是否存在）。

**关联关系要求**

- 支持单个用户关联多个租户。
- 所有用户操作包含租户隔离逻辑，基于 `tenant_id` 的上下文进行权限与数据访问控制。

**注意事项**

- 复用现有的错误处理模式
- 遵循当前项目 Rust 编程最佳实践，对参数使用 snake_case 风格命名
- 注意 Arc 的使用以支持并发访问
- 编写单元测试并运行，确保代码实现正确
```

---

### 优化版开发提示词
