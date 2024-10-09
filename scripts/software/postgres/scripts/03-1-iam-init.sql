SET TIMEZONE TO 'Asia/Chongqing';
\c fusiondata;
\c - fusiondata;

--
-- initial data
------------------
-- 初始化租户
INSERT INTO iam.tenant(name, cid, ctime)
VALUES ('默认租户', 1, CURRENT_TIMESTAMP);
-- 初始化用户
INSERT INTO iam."user" (id, email, phone, name, status, gender, cid, ctime)
VALUES (1, 'admin@fusiondata.com', NULL, '超管', 100, 0, 1, CURRENT_TIMESTAMP),
       (10000, 'user@fusiondata.com', '13912345678', '普通用户', 100, 0, 1, CURRENT_TIMESTAMP);
INSERT INTO iam.user_credential (id, encrypted_pwd, cid, ctime)
VALUES (1,
        '#1#$argon2id$v=19$m=19456,t=2,p=1$iPZ4/KgfN/W2Sm+G94jWgg$Y5rYVQ5VELviwJjBv/epA0PJYl3fD/UkNZrSAauEpbQ', -- 密码为：2024.Fusiondata
        1, CURRENT_TIMESTAMP),
       (10000, '#1#$argon2id$v=19$m=19456,t=2,p=1$m50K7TsFDob8KvtrAFbUPA$y25MggMqyp1tJ8011765alg+TM9v1OeAQiKhFWtdh3g',
        1, CURRENT_TIMESTAMP);
-- 重置 user_id_seq，使新用户注册从ID为 10001 开始
ALTER SEQUENCE iam.user_id_seq RESTART 10001;
INSERT INTO iam.user_tenant_rel (user_id, tenant_id, cid, ctime)
VALUES (1, 1, 1, CURRENT_TIMESTAMP),
       (10000, 1, 1, CURRENT_TIMESTAMP);
--
-- 初始化数据
INSERT INTO iam.role (id, name, description, status, cid, ctime)
VALUES (1, '超级管理员', '拥有所有权限的角色', 100, 1, CURRENT_TIMESTAMP),
       (2, '普通用户', '基本权限的角色', 100, 1, CURRENT_TIMESTAMP);

INSERT INTO iam.permission (id, code, description, resource, action, cid, ctime)
VALUES (1, '用户查看', '查看用户信息的权限', 'user', 'read', 1, CURRENT_TIMESTAMP),
       (2, '用户创建', '创建用户的权限', 'user', 'create', 1, CURRENT_TIMESTAMP),
       (3, '用户更新', '更新用户信息的权限', 'user', 'update', 1, CURRENT_TIMESTAMP),
       (4, '用户删除', '删除用户的权限', 'user', 'delete', 1, CURRENT_TIMESTAMP);
--
-- 为超级管理员分配所有权限
INSERT INTO iam.role_permission (role_id, permission_id, cid, ctime)
VALUES (1, 1, 1, CURRENT_TIMESTAMP),
       (1, 2, 1, CURRENT_TIMESTAMP),
       (1, 3, 1, CURRENT_TIMESTAMP),
       (1, 4, 1, CURRENT_TIMESTAMP);
--
-- 为普通用户分配查看权限
INSERT INTO iam.role_permission (role_id, permission_id, cid, ctime)
VALUES (2, 1, 1, CURRENT_TIMESTAMP);
--
-- 为现有用户分配角色
INSERT INTO iam.user_role (user_id, role_id, cid, ctime)
VALUES (1, 1, 1, CURRENT_TIMESTAMP), -- 超管用户分配超级管理员角色
       (10000, 2, 1, CURRENT_TIMESTAMP);
-- 普通用户分配普通用户角色
--
-- 重置序列
ALTER SEQUENCE iam.role_id_seq RESTART 3;
ALTER SEQUENCE iam.permission_id_seq RESTART 5;


--------
-- ABAC init datas
--------
-- 插入示例属性
INSERT INTO iam.attribute (name, description, entity_type, data_type, cid, ctime)
VALUES ('department', '用户所属部门', 'user', 'string', 1, CURRENT_TIMESTAMP),
       ('clearance_level', '用户安全等级', 'user', 'integer', 1, CURRENT_TIMESTAMP),
       ('document_classification', '文档分类级别', 'resource', 'string', 1, CURRENT_TIMESTAMP),
       ('time_of_day', '当前时间', 'environment', 'string', 1, CURRENT_TIMESTAMP);
-- 插入示例策略
INSERT INTO iam.policy (name, description, condition, effect, resource, action, cid, ctime)
VALUES ('高级文档访问策略', '允许高安全等级用户访问机密文档',
        '{"user.clearance_level": {"gte": 4}, "resource.document_classification": "confidential", "environment.time_of_day": {"between": ["09:00", "17:00"]}}',
        'allow', 'document', 'read', 1, CURRENT_TIMESTAMP);
-- 插入示例属性值
INSERT INTO iam.attribute_value (attribute_id, entity_id, value, cid, ctime)
VALUES ((SELECT id FROM iam.attribute WHERE name = 'department'), 1, 'IT', 1, CURRENT_TIMESTAMP),
       ((SELECT id FROM iam.attribute WHERE name = 'clearance_level'), 1, '5', 1, CURRENT_TIMESTAMP);
