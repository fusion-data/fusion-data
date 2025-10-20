# IAM 多层级数据权限扩展方案

1. 扩展条件键系统

当前的条件键系统可以扩展以支持更多数据维度：

```rust
// 扩展的条件键支持（本次范围：tenant/namespace/project；保留其他维度键但不设计组织权限）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataPermissionConditionKey {
    // 现有条件键
    TenantId,
    PrincipalUserId,
    PrincipalRoles,
    IsPlatformAdmin,

    // 新增数据权限维度（组织相关维度本次不设计，不包含 Organization/Department）
    NamespaceId,
    ProjectId,
    TeamId,
    RegionId,
    ProductLineId,
    BusinessUnitId,

    // 复合条件键
    ResourceHierarchy,
    DataClassification,
    SensitivityLevel,

    // 动态条件键
    Custom(String),
}
```

2. 增强的授权上下文

重构 CtxExt trait，将 roles 方法从 trait 移动到 CtxSvc 服务中，以减少 JWT payload 大小，确保：

- 仅在需要时才加载角色信息
- 角色信息仅包含必要的权限，而不是所有角色

```rust
// jieyuan/jieyuan-core/src/model/ctx_ext.rs
pub trait CtxExt {
    // 现有方法
    fn tenant_id(&self) -> i64;
    fn user_id(&self) -> i64;
}
```

新增 CtxSvc 服务，以支持多层级数据权限

```rust
pub struct CtxSvc; // 位于 jieyuan-server 应用 crate（不可被其他库引用），用于按需查询扩展维度
impl CtxSvc {
    // 从 trait CtxExt 移动到此
    pub fn roles(&self) -> Vec<String> {
        todo!("Implement roles retrieval")
    }

    // 新增数据权限方法（本次范围：tenant/namespace/project）
    pub fn namespace_id(&self) -> Option<i64> {
        todo!("Implement namespace_id retrieval")
    }
    pub fn project_id(&self) -> Option<i64> {
        todo!("Implement project_id retrieval")
    }
    // 其他维度（team_id/region_id/product_line_id/business_unit_id）由 CtxSvc 基于 user_id, tenant_id + AuthorizeRequest.extras 按需查询；
    // 组织相关维度（organization/department）本次不设计，不提供接口。

    pub fn accessible_namespaces(&self) -> Vec<i64> {
        todo!("Implement accessible_namespaces retrieval")
    }
    pub fn accessible_projects(&self) -> Vec<i64> {
        todo!("Implement accessible_projects retrieval")
    }
    pub fn data_permissions(&self) -> &DataPermissions {
        todo!("Implement data_permissions retrieval")
    }
    pub fn resource_hierarchy(&self) -> &ResourceHierarchy {
        todo!("Implement resource_hierarchy retrieval")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DataPermissions {
    pub namespace_access: NamespaceAccess,
    pub project_access: ProjectAccess,
    // 本次不设计组织权限，移除 organization_access
    pub custom_permissions: HashMap<String, Vec<String>>,
}

// 核心新增类型（位于 jieyuan-core，可被其他库引用）
// jieyuan/jieyuan-core/src/model/iam_types.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceHierarchy {
    pub tenant_id: i64,
    pub namespace_id: Option<i64>,
    pub project_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    StringEquals,
    StringLike,
    NumericEquals,
    NumericLessThan,
    NumericGreaterThan,
    In,
    NotIn,
    BoolEquals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub key: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
    pub scope: Option<String>, // 例如 "tenant" | "namespace" | "project"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect { Allow, Deny }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub effect: Effect,
    pub action: String,
    pub resource: String,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NamespaceAccess {
    pub owned_namespaces: Vec<i64>,
    pub member_namespaces: Vec<i64>,
    pub guest_namespaces: Vec<i64>,
    pub admin_namespaces: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectAccess {
    pub owned_projects: Vec<i64>,
    pub member_projects: Vec<i64>,
    pub guest_projects: Vec<i64>,
    pub admin_projects: Vec<i64>,
}
```

3. 资源模板的层级化设计

增强资源模板以支持多层级权限：

```rust
// jieyuan/jieyuan-core/src/model/resource_template.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct HierarchicalResourceTemplate {
    pub base_template: String,
    pub hierarchy_levels: Vec<HierarchyLevel>,
    pub fallback_templates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct HierarchyLevel {
    pub level_name: String, // "tenant", "namespace", "project"
    pub condition_key: String, // "iam:namespace_id", "iam:project_id"
    pub is_required: bool,
    pub default_value: Option<String>,
}

// 示例资源模板
let template = HierarchicalResourceTemplate {
    base_template: "iam:hetumind:{tenant_id}:{namespace_id}:{project_id}:workflow/{id}".to_string(),
    hierarchy_levels: vec![
        HierarchyLevel {
            level_name: "tenant".to_string(),
            condition_key: "iam:tenant_id".to_string(),
            is_required: true,
            default_value: None,
        },
        HierarchyLevel {
            level_name: "namespace".to_string(),
            condition_key: "iam:namespace_id".to_string(),
            is_required: true,
            default_value: None,
        },
        HierarchyLevel {
            level_name: "project".to_string(),
            condition_key: "iam:project_id".to_string(),
            is_required: false,
            default_value: Some("default".to_string()),
        },
    ],
    fallback_templates: vec![
        "iam:hetumind:{tenant_id}:{namespace_id}:workflow/{id}".to_string(),
        "iam:hetumind:{tenant_id}:workflow/{id}".to_string(),
    ],
};

// 标准资源标识语法（参考 iam.md）
// - 规范格式（策略配置用）：iam:{service}:{tenant_id}[:{namespace_id}][:{project_id}]:{type}/{id}
// - 简化格式（API端使用）：iam:{service}:{type}/{id} 由运行时注入 tenant_id（以及必要层级参数）
// - 规则：
//   * 若出现 {project_id}，必须同时出现 {namespace_id}
//   * 缺失必须参数时拒绝，不得使用默认值占位（如 0）
//   * 仅允许同租户内评估与继承
```

4. 增强的策略评估引擎

扩展策略评估以支持多层级数据权限：

```rust
// jieyuan/jieyuan-server/src/access_control/enhanced_policy_engine.rs
#[derive(Clone)]
pub struct EnhancedPolicyEngine {
    base_engine: PolicyEngine,
    data_permission_resolver: DataPermissionResolver,
    hierarchy_resolver: HierarchyResolver,
}

impl EnhancedPolicyEngine {
    pub async fn evaluate_with_data_permissions(
        &self,
        ctx: &Ctx,
        action: &str,
        resource: &str,
        data_context: &DataContext,
    ) -> Result<Decision> {
        // 1. 基础策略评估（显式 Deny 优先）
        let base_decision = self.base_engine.evaluate(ctx, action, resource).await?;
        if base_decision == Decision::Deny { return Ok(Decision::Deny); }

        // 2. 数据权限评估
        let data_decision = self.evaluate_data_permissions(ctx, resource, data_context).await?;

        // 3. 层级权限评估（同租户内的垂直层级）
        let hierarchy_decision = self.evaluate_hierarchy_permissions(ctx, resource, data_context).await?;

        // 4. 综合决策
        Ok(self.combine_decisions(base_decision, data_decision, hierarchy_decision))
    }

    async fn evaluate_data_permissions(
        &self,
        ctx: &Ctx,
        resource: &str,
        data_context: &DataContext,
    ) -> Result<Decision> {
        // 解析资源中的数据维度
        let resource_dims = self.parse_resource_dimensions(resource)?;

        // 检查 namespace 权限
        if let Some(resource_namespace) = resource_dims.namespace_id {
            if !self.has_namespace_access(ctx, resource_namespace, data_context)? {
                return Ok(Decision::Deny);
            }
        }
        // 检查 project 权限
        if let Some(resource_project) = resource_dims.project_id {
            if !self.has_project_access(ctx, resource_project, data_context)? {
                return Ok(Decision::Deny);
            }
        }
        Ok(Decision::Allow)
    }
}

#[derive(Debug, Clone)]
pub struct DataContext {
    pub namespace_id: Option<i64>,
    pub project_id: Option<i64>,
    pub custom_attributes: HashMap<String, String>,
}
```

规则说明（策略求值的综合规则与显式 Deny 优先级）

- 评估时即时解析（不做预解析/缓存/入库）：校验资源标识语法，并使用 `render_resource_enhanced` 渲染占位符；若检测到所需参数缺失（如 `{namespace_id}`、`{project_id}`）立即返回错误并拒绝。所有策略均在评估时从策略配置文件加载与解析，不生成持久化表示。
- 显式 `deny` 优先：任一匹配声明命中 `deny` 即短路拒绝，后续不再评估。
- `allow` 后裁剪：命中 `allow` 后仍需通过权限边界与会话策略裁剪，越界则拒绝。
- 条件求值：绑定 `Ctx` 与必要的 `DataContext`，缺失必须参数直接拒绝，且不使用默认 ID。
- 合并顺序：基础策略 → 数据权限 → 层级权限。每一步都应用显式 `deny` 优先规则并进行裁剪。

5. 权限继承和传播机制

实现权限的层级继承：

```rust
// jieyuan/jieyuan-server/src/access_control/permission_inheritance.rs
#[derive(Debug, Clone)]
pub struct PermissionInheritanceEngine {
    inheritance_rules: Vec<InheritanceRule>,
}

#[derive(Debug, Clone)]
pub struct InheritanceRule {
    pub from_level: String, // "tenant" -> "namespace" -> "project"
    pub to_level: String,
    pub inheritance_type: InheritanceType,
    pub conditions: Vec<Condition>, // 不允许跨租户或跨平级传播
}

#[derive(Debug, Clone)]
pub enum InheritanceType {
    Full,           // 完全继承
    Filtered,       // 过滤继承
    Aggregated,     // 聚合继承
    Custom(String), // 自定义规则
}

impl PermissionInheritanceEngine {
    pub async fn resolve_inherited_permissions(
        &self,
        ctx: &Ctx,
        resource: &str,
    ) -> Result<Vec<Permission>> {
        let mut permissions = Vec::new();
        let resource_path = self.parse_resource_hierarchy(resource)?;

        // 自底向上继承权限
        for (level, resource_id) in resource_path.iter().rev() {
            let level_permissions = self.get_level_permissions(ctx, level, resource_id).await?;
            permissions.extend(self.apply_inheritance_rules(level_permissions, level));
        }

        Ok(self.consolidate_permissions(permissions))
    }
}
```

6. 实际应用示例

Hetuflow 命名空间权限控制

```rust
// hetuflow-server 中使用增强的权限控制
pub async fn create_workflow(
    ctx: Ctx,
    workflow_svc: WorkflowSvc,
    Json(req): Json<CreateWorkflowRequest>,
) -> WebResult<Workflow> {
    // 必须提供 namespace_id，缺失即拒绝
    let Some(namespace_id) = req.namespace_id else {
        return Err(WebError::forbidden("missing namespace_id"));
    };

    // 设置数据上下文（不包含组织维度）
    let data_context = DataContext {
        namespace_id: Some(namespace_id),
        project_id: None,
        custom_attributes: HashMap::new(),
    };

    // 使用增强的策略引擎评估权限
    let resource = format!("iam:hetuflow:{}:{}:workflow",
        ctx.tenant_id(),
        namespace_id
    );

    let decision = enhanced_policy_engine
        .evaluate_with_data_permissions(&ctx, "hetuflow:create", &resource, &data_context)
        .await?;

    if decision == Decision::Deny {
        return Err(WebError::forbidden("Insufficient namespace permissions"));
    }

    // 执行业务逻辑
    let workflow = workflow_svc.create_workflow(req).await?;
    Ok(Json(workflow))
}
```

Hetumind 项目级权限控制

```rust
// hetumind-studio 中使用项目级权限控制
pub async fn get_project_agents(
    ctx: Ctx,
    agent_svc: AgentSvc,
    Path(project_id): Path<ProjectId>,
) -> WebResult<Vec<Agent>> {
    // 必须提供 namespace_id（同域约束），缺失即拒绝
    let namespace_id = match ctx.namespace_id() {
        Some(ns) => ns,
        None => return Err(WebError::forbidden("missing namespace_id")),
    };

    // 构建层级化资源模板（tenant -> namespace -> project）
    let resource = format!("iam:hetumind:{}:{}:{}:agent",
        ctx.tenant_id(),
        namespace_id,
        project_id
    );

    // 检查项目访问权限（不包含组织维度）
    let data_context = DataContext {
        namespace_id: Some(namespace_id),
        project_id: Some(project_id),
        custom_attributes: HashMap::new(),
    };

    let decision = enhanced_policy_engine
        .evaluate_with_data_permissions(&ctx, "hetumind:read", &resource, &data_context)
        .await?;

    if decision == Decision::Deny {
        return Err(WebError::forbidden("Insufficient project access"));
    }

    // 执行查询逻辑
    let agents = agent_svc.get_project_agents(project_id).await?;
    Ok(Json(agents))
}
```

7. 策略配置示例

```rust
{
  "version": "2025-01-01",
  "id": "hetuflow-namespace-admin",
  "statement": [
    {
      "sid": "namespace_full_access",
      "effect": "allow",
      "action": ["hetuflow:*"],
      "resource": ["iam:hetuflow:{tenant_id}:{namespace_id}:*"],
      "condition": {
        "string_equals": {
          "iam:namespace_access_level": "admin"
        },
        "numeric_equals": {
          "iam:tenant_id": "{ctx.tenant_id}",
          "iam:namespace_id": "{ctx.namespace_id}"
        }
      }
    },
    {
      "sid": "namespace_limited_access",
      "effect": "allow",
      "action": ["hetuflow:read", "hetuflow:execute"],
      "resource": ["iam:hetuflow:{tenant_id}:{namespace_id}:*/**"],
      "condition": {
        "string_equals": {
          "iam:namespace_access_level": "member"
        },
        "numeric_equals": {
          "iam:tenant_id": "{ctx.tenant_id}",
          "iam:namespace_id": "{ctx.namespace_id}"
        }
      }
    }
  ]
}
```

⏺ 具体实现建议和迁移方案

1. 渐进式迁移策略

阶段一：扩展条件键系统

```rust
// 在现有条件键基础上扩展（本次范围不含组织维度）
// jieyuan/jieyuan-core/src/model/condition_keys.rs
pub const DATA_PERMISSION_CONDITION_KEYS: &[&str] = &[
    // 现有条件键
    "iam:tenant_id",
    "iam:user_id",
    "iam:roles",
    "iam:is_platform_admin",

    // 数据权限条件键（namespace/project）
    "iam:namespace_id",
    "iam:project_id",
    "iam:accessible_namespaces",
    "iam:accessible_projects",
    "iam:namespace_access_level",
    "iam:project_access_level",
    "iam:resource_hierarchy",
];

// 说明：本次采用独立的 access_level 键（namespace/project），避免 scope 叠加的歧义；
// 若未来统一为 "iam:access_level"，需引入规范化 scope 标识并调整条件求值器的键解析。
```

选择说明（access_level 设计）

- 使用独立键 `iam:namespace_access_level` 与 `iam:project_access_level`，使条件更直观、解析更稳定。
- 若统一为单键 `iam:access_level`，必须同时携带明确的 `scope`（如 `namespace`/`project`），并在条件解析层做键拆分与规范校验，避免跨域误匹配。
- 当前版本采用独立键，未来如确需统一，可在不破坏兼容的前提下增加解析映射层。

阶段二：增强资源模板渲染

```rust
// 扩展现有的 render_resource 函数（严格模式：缺失必须参数直接拒绝）
// jieyuan/jieyuan-core/src/model/iam_api.rs
pub fn render_resource_enhanced(
    tpl: &str,
    ctx: &Ctx,
    extras: Option<&HashMap<String, String>>,
    data_context: Option<&DataContext>
) -> Result<String, RenderError> {
    let mut s = tpl.to_string();

    // 现有占位符替换
    s = render_base_placeholders(&s, ctx, extras);

    // 新增数据权限占位符（不允许使用 unwrap_or(0) 作为缺省）
    if let Some(dc) = data_context {
        if s.contains("{namespace_id}") && dc.namespace_id.is_none() {
            return Err(RenderError::missing_param("namespace_id"));
        }
        if s.contains("{project_id}") && dc.project_id.is_none() {
            return Err(RenderError::missing_param("project_id"));
        }
        if let Some(ns) = dc.namespace_id { s = s.replace("{namespace_id}", &ns.to_string()); }
        if let Some(pid) = dc.project_id { s = s.replace("{project_id}", &pid.to_string()); }
    }

    // 智能层级注入
    s = inject_hierarchy_context(&s, ctx, data_context);

    Ok(s)
}

fn inject_hierarchy_context(
    template: &str,
    ctx: &Ctx,
    data_context: Option<&DataContext>
) -> String {
    // 根据模板的复杂程度自动注入适当的层级上下文
    if template.contains("{namespace_id}") && template.contains("{project_id}") {
        // 完整层级：iam:service:tenant:namespace:project:resource
        template.to_string()
    } else if template.contains("{namespace_id}") {
        // namespace级别：iam:service:tenant:namespace:resource
        template.to_string()
    } else {
        // tenant级别：iam:service:tenant:resource
        template.to_string()
    }
}
```

2. 数据库设计扩展

```sql
-- 扩展用户权限表以支持多层级数据权限
CREATE TABLE iam_user_data_permissions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    permission_type VARCHAR(50) NOT NULL, -- 'namespace', 'project'
    resource_id BIGINT NOT NULL,
    access_level VARCHAR(50) NOT NULL, -- 'admin', 'member', 'guest', 'viewer'
    granted_by BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    conditions JSONB, -- 额外的权限条件
    CONSTRAINT iam_user_data_permissions_pk PRIMARY KEY (id),
    CONSTRAINT iam_user_data_permissions_fk_user FOREIGN KEY (user_id) REFERENCES iam_user(id),
    CONSTRAINT iam_user_data_permissions_fk_tenant FOREIGN KEY (tenant_id) REFERENCES iam_tenant(id),
    CONSTRAINT iam_user_data_permissions_uk_user_tenant_resource UNIQUE(user_id, tenant_id, permission_type, resource_id)
);

-- 创建索引优化查询性能
CREATE INDEX idx_user_data_permissions_user ON iam_user_data_permissions(user_id);
CREATE INDEX idx_user_data_permissions_tenant ON iam_user_data_permissions(tenant_id);
CREATE INDEX idx_user_data_permissions_resource ON iam_user_data_permissions(permission_type, resource_id);
CREATE INDEX idx_user_data_permissions_level ON iam_user_data_permissions(access_level);
CREATE INDEX idx_user_data_permissions_expires ON iam_user_data_permissions(expires_at);

-- 条件规范化与索引设计建议（新增）
-- 1) 统一条件结构：在策略语句中使用规范化条件结构，便于校验与检索
--    conditions: [
--      { key: "iam:namespace_id", operator: "string_equals", value: "123" },
--      { key: "iam:namespace_access_level", operator: "in", value: ["admin", "member"] }
--    ]
-- 2) JSON Schema 校验：在导入/保存策略时执行 schema 验证，拒绝不合法条件
-- 3) 索引策略：
--    - 为常用键（principal_user_id/namespace_id/project_id/access_level）建立映射表做二级索引
--    - 同时对原始 JSONB 使用 GIN 索引以支持快速存在性检查
-- 4) 迁移建议：初版实现可仅使用 JSONB + GIN；当策略量增大或查询变复杂，再引入映射表

-- 示例：策略条件二级映射表（可选，后续按需启用）
CREATE TABLE iam_policy_condition_index (
    id BIGSERIAL PRIMARY KEY,
    policy_id BIGINT NOT NULL,
    statement_id BIGINT NOT NULL,
    key TEXT NOT NULL,
    operator TEXT NOT NULL,
    value_text TEXT,
    value_numeric BIGINT,
    value_json JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
CREATE INDEX idx_condition_index_policy ON iam_policy_condition_index (policy_id);
CREATE INDEX idx_condition_index_key_op ON iam_policy_condition_index (key, operator);
CREATE INDEX idx_condition_index_value_text ON iam_policy_condition_index (value_text);
CREATE INDEX idx_condition_index_value_numeric ON iam_policy_condition_index (value_numeric);
CREATE INDEX idx_condition_index_value_json_gin ON iam_policy_condition_index USING GIN (value_json);

-- 权限继承规则表
CREATE TABLE iam_permission_inheritance_rules (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    parent_type VARCHAR(50) NOT NULL, -- 'tenant', 'namespace', 'project'
    child_type VARCHAR(50) NOT NULL,  -- 'namespace', 'project', 'workflow'
    inheritance_type VARCHAR(50) NOT NULL, -- 'full', 'filtered', 'aggregated'
    conditions JSONB,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT iam_permission_inheritance_rules_pk PRIMARY KEY (id),
    CONSTRAINT iam_permission_inheritance_rules_uk_hierarchy UNIQUE(tenant_id, parent_type, child_type)
);
```

3. 性能优化策略

缓存层级权限（未来计划，当前阶段不实现）

```rust
// jieyuan/jieyuan-server/src/access_control/permission_cache.rs
#[derive(Clone)]
pub struct HierarchicalPermissionCache {
    // 多层缓存设计（未来计划：当前不实现缓存/入库，仅保留设计草案）
    user_permissions_cache: Arc<DashMap<String, CachedPermissions>>,
    namespace_hierarchy_cache: Arc<DashMap<i64, CachedHierarchy>>,
    project_permissions_cache: Arc<DashMap<i64, CachedProjectPermissions>>,

    // 容量控制（可选）
    max_cache_size: usize,
}

#[derive(Debug, Clone)]
pub struct CachedPermissions {
    pub permissions: Vec<Permission>,
    pub hierarchy_path: Vec<String>,
    pub access_levels: HashMap<String, Vec<String>>,
}

impl HierarchicalPermissionCache {
    pub async fn get_user_permissions(&self, ctx: &Ctx) -> Result<CachedPermissions> {
        let cache_key = format!("user:{}:tenant:{}", ctx.user_id(), ctx.tenant_id());

        // 本次不使用 TTL：直接从数据库加载，后续可在 TODO 中引入 TTL/事件广播
        let permissions = self.load_user_permissions_from_db(ctx).await?;
        let hierarchy = self.resolve_user_hierarchy(ctx).await?;

        let cached_permissions = CachedPermissions {
            permissions: permissions.clone(),
            hierarchy_path: hierarchy,
            access_levels: self.extract_access_levels(&permissions),
        };

        // 写入缓存（未来计划，不在当前阶段实现）
        self.user_permissions_cache.insert(cache_key, cached_permissions.clone());

        Ok(cached_permissions)
    }
}
```

4. 集成示例和最佳实践

Hetuflow 集成示例

```rust
// hetuflow-server/src/workflow_handler.rs
use jieyuan_server::access_control::{EnhancedPolicyEngine, DataContext};

pub async fn list_workflows(
    ctx: Ctx,
    workflow_svc: WorkflowSvc,
    Query(params): Query<ListWorkflowsQuery>,
) -> WebResult<Vec<Workflow>> {
    // 构建数据权限上下文
    let data_context = DataContext {
        namespace_id: params.namespace_id.or(ctx.namespace_id()),
        project_id: params.project_id,
        custom_attributes: HashMap::from([
            ("workflow_type".to_string(), params.workflow_type.clone().unwrap_or_default())
        ]),
    };

    // 使用增强的权限引擎
    let resource = "iam:hetuflow:workflow";
    let decision = enhanced_policy_engine
        .evaluate_with_data_permissions(&ctx, "hetuflow:list", resource, &data_context)
        .await?;

    match decision {
        Decision::Allow => {
            // 根据权限级别过滤数据
            let accessible_namespaces = ctx.accessible_namespaces();
            let accessible_projects = ctx.accessible_projects();

            let workflows = workflow_svc
                .list_workflows_with_permissions(
                    accessible_namespaces,
                    accessible_projects,
                    params
                )
                .await?;

            Ok(Json(workflows))
        }
        Decision::Deny => Err(WebError::forbidden("Insufficient permissions to list workflows")),
    }
}
```

Hetumind 集成示例

```rust
// hetumind-studio/src/agent_handler.rs
pub async fn create_agent(
    ctx: Ctx,
    agent_svc: AgentSvc,
    Json(req): Json<CreateAgentRequest>,
) -> WebResult<Agent> {
    // 多层级权限验证
    // 必须提供 project_id 和 namespace_id（同域约束），缺失即拒绝
    let Some(namespace_id) = req.namespace_id else {
        return Err(WebError::forbidden("missing namespace_id"));
    };
    let Some(project_id) = req.project_id else {
        return Err(WebError::forbidden("missing project_id"));
    };

    let resource_hierarchies = vec![
        format!("iam:hetumind:{}:{}:{}:agent",
            ctx.tenant_id(),
            namespace_id,
            project_id
        ),
        format!("iam:hetumind:{}:{}:agent",
            ctx.tenant_id(),
            namespace_id
        ),
        format!("iam:hetumind:{}:agent", ctx.tenant_id())
    ];

    let data_context = DataContext {
        namespace_id: Some(namespace_id),
        project_id: Some(project_id),
        custom_attributes: HashMap::from([
            ("agent_type".to_string(), req.agent_type.clone())
        ]),
    };

    // 逐级验证权限
    for resource in resource_hierarchies {
        let decision = enhanced_policy_engine
            .evaluate_with_data_permissions(&ctx, "hetumind:create", &resource, &data_context)
            .await?;

        if decision == Decision::Allow {
            let agent = agent_svc.create_agent(req).await?;
            return Ok(Json(agent));
        }
    }

    Err(WebError::forbidden("Insufficient permissions to create agent"))
}
```

⏺ 总结和建议

核心增强要点

1. 扩展条件键系统：在现有条件键基础上增加 namespace_id、project_id 等数据权限维度（本次不设计组织/部门维度）
2. 层级化资源模板：支持多层级资源标识符，如 iam:service:tenant:namespace:project:resource
3. 权限继承机制：实现自顶向下的权限继承和自底向上的权限聚合
4. 增强的策略引擎：支持多维度权限评估和层级权限解析
5. 性能优化（未来计划）：通过多层缓存和智能索引优化权限检查性能；当前阶段禁用缓存与持久化，仅保留设计建议。

实施路径

1. 渐进式扩展：先扩展条件键和资源模板，再增加权限继承功能
2. 向后兼容：保持现有 API 和策略格式的兼容性
3. 性能优先（未来计划）：通过缓存和索引优化确保权限检查不成为性能瓶颈；当前阶段不实现缓存与持久化。
4. 灵活配置：支持动态配置权限层级和继承规则

最佳实践

1. 权限最小化：默认拒绝，明确授权
2. 层级简化：避免过深的权限层级，建议不超过 3-4 层
3. 缓存策略（未来计划）：合理设置缓存 TTL，平衡性能和一致性；当前阶段不实现缓存与持久化。
4. 监控审计：记录权限决策过程，便于问题排查和安全审计

这个增强方案在保持现有 IAM 设计优点的基础上，通过扩展条件键、资源模板和策略引擎，实现了对 namespace_id、project_id 等多层级数据权限的支持，同时保持了系统的可扩展性和高性能。
