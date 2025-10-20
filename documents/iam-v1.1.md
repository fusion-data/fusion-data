# IAM 多层级数据权限扩展方案

1. 扩展条件键系统

当前的条件键系统可以扩展以支持更多数据维度：

```rust
// 扩展的条件键支持
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataPermissionConditionKey {
    // 现有条件键
    TenantId,
    PrincipalUserId,
    PrincipalRoles,
    IsPlatformAdmin,

    // 新增数据权限维度
    NamespaceId,
    ProjectId,
    OrganizationId,
    DepartmentId,
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

扩展 CtxExt trait 以支持多层级数据权限：

```rust
// jieyuan/jieyuan-core/src/model/ctx_ext.rs
pub trait CtxExt {
    // 现有方法
    fn tenant_id(&self) -> i64;
    fn user_id(&self) -> i64;
    fn roles(&self) -> &[String];

    // 新增数据权限方法
    fn namespace_id(&self) -> Option<i64>;
    fn project_id(&self) -> Option<i64>;
    fn organization_id(&self) -> Option<i64>;
    fn department_id(&self) -> Option<i64>;
    fn accessible_namespaces(&self) -> &[i64];
    fn accessible_projects(&self) -> &[i64];
    fn data_permissions(&self) -> &DataPermissions;
    fn resource_hierarchy(&self) -> &ResourceHierarchy;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DataPermissions {
    pub namespace_access: NamespaceAccess,
    pub project_access: ProjectAccess,
    pub organization_access: OrganizationAccess,
    pub custom_permissions: HashMap<String, Vec<String>>,
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
    pub level_name: String, // "tenant", "namespace", "project", "organization"
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
```

4. 增强的策略评估引擎

扩展策略评估以支持多层级数据权限：

```rust
// jieyuan/jieyuan/src/access_control/enhanced_policy_engine.rs
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
        // 1. 基础策略评估
        let base_decision = self.base_engine.evaluate(ctx, action, resource).await?;

        if base_decision == Decision::Deny {
            return Ok(Decision::Deny);
        }

        // 2. 数据权限评估
        let data_decision = self.evaluate_data_permissions(ctx, resource, data_context).await?;

        // 3. 层级权限评估
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

        // 检查namespace权限
        if let Some(resource_namespace) = resource_dims.namespace_id {
            if !self.has_namespace_access(ctx, resource_namespace, data_context)? {
                return Ok(Decision::Deny);
            }
        }

        // 检查project权限
        if let Some(resource_project) = resource_dims.project_id {
            if !self.has_project_access(ctx, resource_project, data_context)? {
                return Ok(Decision::Deny);
            }
        }

        // 检查其他数据维度...

        Ok(Decision::Allow)
    }
}

#[derive(Debug, Clone)]
pub struct DataContext {
    pub namespace_id: Option<i64>,
    pub project_id: Option<i64>,
    pub organization_id: Option<i64>,
    pub custom_attributes: HashMap<String, String>,
}
```

5. 权限继承和传播机制

实现权限的层级继承：

```rust
// jieyuan/jieyuan/src/access_control/permission_inheritance.rs
#[derive(Debug, Clone)]
pub struct PermissionInheritanceEngine {
    inheritance_rules: Vec<InheritanceRule>,
}

#[derive(Debug, Clone)]
pub struct InheritanceRule {
    pub from_level: String, // "organization" -> "department" -> "team"
    pub to_level: String,
    pub inheritance_type: InheritanceType,
    pub conditions: Vec<Condition>,
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
    // 设置数据上下文
    let data_context = DataContext {
        namespace_id: req.namespace_id,
        project_id: None,
        organization_id: ctx.organization_id(),
        custom_attributes: HashMap::new(),
    };

    // 使用增强的策略引擎评估权限
    let resource = format!("iam:hetuflow:{}:{}:workflow",
        ctx.tenant_id(),
        req.namespace_id.unwrap_or(0)
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
    // 构建层级化资源模板
    let resource = format!("iam:hetumind:{}:{}:{}/agent",
        ctx.tenant_id(),
        ctx.namespace_id().unwrap_or(0),
        project_id
    );

    // 检查项目访问权限
    let data_context = DataContext {
        namespace_id: ctx.namespace_id(),
        project_id: Some(project_id),
        organization_id: ctx.organization_id(),
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
          "iam:namespace_id": "{ctx_namespace_id}"
        }
      }
    },
    {
      "sid": "project_limited_access",
      "effect": "allow",
      "action": ["hetuflow:read", "hetuflow:execute"],
      "resource": ["iam:hetuflow:{tenant_id}:{namespace_id}:{project_id}:*"],
      "condition": {
        "string_equals": {
          "iam:project_access_level": ["member", "admin"]
        },
        "numeric_equals": {
          "iam:project_id": "{ctx_project_id}"
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
// 在现有条件键基础上扩展
// jieyuan/jieyuan-core/src/model/condition_keys.rs
pub const DATA_PERMISSION_CONDITION_KEYS: &[&str] = &[
    // 现有条件键
    "iam:tenant_id",
    "iam:principal_user_id",
    "iam:principal_roles",
    "iam:is_platform_admin",

    // 新增数据权限条件键
    "iam:namespace_id",
    "iam:project_id",
    "iam:organization_id",
    "iam:department_id",
    "iam:accessible_namespaces",
    "iam:accessible_projects",
    "iam:data_permission_level",
    "iam:resource_hierarchy",
];
```

阶段二：增强资源模板渲染

```rust
// 扩展现有的 render_resource 函数
// jieyuan/jieyuan-core/src/model/iam_api.rs
pub fn render_resource_enhanced(
    tpl: &str,
    ctx: &Ctx,
    extras: Option<&HashMap<String, String>>,
    data_context: Option<&DataContext>
) -> String {
    let mut s = tpl.to_string();

    // 现有占位符替换
    s = render_base_placeholders(&s, ctx, extras);

    // 新增数据权限占位符
    if let Some(dc) = data_context {
        s = s.replace("{namespace_id}", &dc.namespace_id.unwrap_or(0).to_string());
        s = s.replace("{project_id}", &dc.project_id.unwrap_or(0).to_string());
        s = s.replace("{organization_id}", &dc.organization_id.unwrap_or(0).to_string());
    }

    // 智能层级注入
    s = inject_hierarchy_context(&s, ctx, data_context);

    s
}

fn inject_hierarchy_context(
    template: &str,
    ctx: &Ctx,
    data_context: Option<&DataContext>
) -> String {
    // 根据模板的复杂程度自动注入适当的层级上下文
    if template.contains("{namespace_id}") &&
        template.contains("{project_id}") {
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
    permission_type VARCHAR(50) NOT NULL, -- 'namespace', 'project', 'organization'
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

-- 权限继承规则表
CREATE TABLE iam_permission_inheritance_rules (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    parent_type VARCHAR(50) NOT NULL, -- 'organization', 'namespace', 'project'
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

缓存层级权限

```rust
// jieyuan/jieyuan/src/access_control/permission_cache.rs
#[derive(Clone)]
pub struct HierarchicalPermissionCache {
    // 多层缓存设计
    user_permissions_cache: Arc<DashMap<String, CachedPermissions>>,
    namespace_hierarchy_cache: Arc<DashMap<i64, CachedHierarchy>>,
    project_permissions_cache: Arc<DashMap<i64, CachedProjectPermissions>>,

    // 缓存配置
    cache_ttl: Duration,
    max_cache_size: usize,
}

#[derive(Debug, Clone)]
pub struct CachedPermissions {
    pub permissions: Vec<Permission>,
    pub hierarchy_path: Vec<String>,
    pub expires_at: Instant,
    pub access_levels: HashMap<String, Vec<String>>,
}

impl HierarchicalPermissionCache {
    pub async fn get_user_permissions(&self, ctx: &Ctx) -> Result<CachedPermissions> {
        let cache_key = format!("user:{}:tenant:{}", ctx.user_id(), ctx.tenant_id());

        // 检查缓存
        if let Some(cached) = self.user_permissions_cache.get(&cache_key) {
            if cached.expires_at > Instant::now() {
                return Ok(cached.clone());
            }
        }

        // 缓存未命中，从数据库加载
        let permissions = self.load_user_permissions_from_db(ctx).await?;
        let hierarchy = self.resolve_user_hierarchy(ctx).await?;

        let cached_permissions = CachedPermissions {
            permissions: permissions.clone(),
            hierarchy_path: hierarchy,
            expires_at: Instant::now() + self.cache_ttl,
            access_levels: self.extract_access_levels(&permissions),
        };

        // 更新缓存
        self.user_permissions_cache.insert(cache_key, cached_permissions.clone());

        Ok(cached_permissions)
    }
}
```

4. 集成示例和最佳实践

Hetuflow 集成示例

```rust
// hetuflow-server/src/workflow_handler.rs
use jieyuan::access_control::{EnhancedPolicyEngine, DataContext};

pub async fn list_workflows(
    ctx: Ctx,
    workflow_svc: WorkflowSvc,
    Query(params): Query<ListWorkflowsQuery>,
) -> WebResult<Vec<Workflow>> {
    // 构建数据权限上下文
    let data_context = DataContext {
        namespace_id: params.namespace_id.or(ctx.namespace_id()),
        project_id: params.project_id,
        organization_id: ctx.organization_id(),
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
    let resource_hierarchies = vec![
        format!("iam:hetumind:{}:{}:{}:agent",
            ctx.tenant_id(),
            req.namespace_id.unwrap_or(0),
            req.project_id.unwrap_or(0)
        ),
        format!("iam:hetumind:{}:{}:agent",
            ctx.tenant_id(),
            req.namespace_id.unwrap_or(0)
        ),
        format!("iam:hetumind:{}:agent", ctx.tenant_id())
    ];

    let data_context = DataContext {
        namespace_id: req.namespace_id,
        project_id: req.project_id,
        organization_id: ctx.organization_id(),
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

1. 扩展条件键系统：在现有条件键基础上增加 namespace_id、project_id、organization_id 等数据权限维度
2. 层级化资源模板：支持多层级资源标识符，如 iam:service:tenant:namespace:project:resource
3. 权限继承机制：实现自顶向下的权限继承和自底向上的权限聚合
4. 增强的策略引擎：支持多维度权限评估和层级权限解析
5. 性能优化：通过多层缓存和智能索引优化权限检查性能

实施路径

1. 渐进式扩展：先扩展条件键和资源模板，再增加权限继承功能
2. 向后兼容：保持现有 API 和策略格式的兼容性
3. 性能优先：通过缓存和索引优化确保权限检查不成为性能瓶颈
4. 灵活配置：支持动态配置权限层级和继承规则

最佳实践

1. 权限最小化：默认拒绝，明确授权
2. 层级简化：避免过深的权限层级，建议不超过 3-4 层
3. 缓存策略：合理设置缓存 TTL，平衡性能和一致性
4. 监控审计：记录权限决策过程，便于问题排查和安全审计

这个增强方案在保持现有 IAM 设计优点的基础上，通过扩展条件键、资源模板和策略引擎，实现了对 namespace_id、project_id 等多层级数据权限的支持，同时保持了系统的可扩展性和高性能。
