# ABAC

为了支持ABAC（基于属性的访问控制）模型，我们需要扩展现有的数据模型。以下是ABAC支持的设计文档和相应的SQL语句：

## ABAC 支持设计文档

1. 属性表：存储各种实体（用户、资源、环境等）的属性
2. 策略表：定义ABAC策略规则
3. 属性值表：存储具体实体的属性值

### SQL语句

```sql

```

### 示例数据

```sql
-- 插入示例属性
INSERT INTO iam.attribute (name, description, entity_type, data_type, cid, ctime)
VALUES
('department', '用户所属部门', 'user', 'string', 1, CURRENT_TIMESTAMP),
('clearance_level', '用户安全等级', 'user', 'integer', 1, CURRENT_TIMESTAMP),
('document_classification', '文档分类级别', 'resource', 'string', 1, CURRENT_TIMESTAMP),
('time_of_day', '当前时间', 'environment', 'string', 1, CURRENT_TIMESTAMP);

-- 插入示例策略
INSERT INTO iam.policy (name, description, condition, effect, resource, action, cid, ctime)
VALUES
('高级文档访问策略', '允许高安全等级用户访问机密文档',
 '{"user.clearance_level": {"gte": 4}, "resource.document_classification": "confidential", "environment.time_of_day": {"between": ["09:00", "17:00"]}}',
 'allow', 'document', 'read', 1, CURRENT_TIMESTAMP);

-- 插入示例属性值
INSERT INTO iam.attribute_value (attribute_id, entity_id, value, cid, ctime)
VALUES
((SELECT id FROM iam.attribute WHERE name = 'department'), 1, 'IT', 1, CURRENT_TIMESTAMP),
((SELECT id FROM iam.attribute WHERE name = 'clearance_level'), 1, '5', 1, CURRENT_TIMESTAMP);
```

这个设计允许您：

1. 定义各种类型的属性（用户、资源、环境）
2. 创建复杂的ABAC策略
3. 为具体实体（如用户）分配属性值

在实际应用中，您需要实现一个策略引擎来解析和评估这些ABAC规则。该引擎将结合RBAC和ABAC的规则，以决定是否授予访问权限。

请注意，这只是ABAC支持的基本框架。您可能需要根据具体需求进行调整和扩展。例如，您可能需要添加版本控制、策略优先级等功能。
