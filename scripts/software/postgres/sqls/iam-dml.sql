set
    timezone to 'Asia/Chongqing';

-- create schema
create schema if not exists fusiondata;
set search_path to fusiondata;

--
-- initial data
------------------
-- 初始化租户
insert into tenant (name, cid, ctime)
values ('默认租户', 1, current_timestamp);

-- 初始化用户
insert into "user" (id, email, phone, name, status, gender, cid, ctime)
values (1, 'admin@fusiondata.com', null, '超管', 100, 0, 1, current_timestamp),
       (10000,
        'user@fusiondata.com',
        '13912345678',
        '普通用户',
        100,
        0,
        1,
        current_timestamp);

insert into user_credential (id, encrypted_pwd, cid, ctime)
values (1,
        '#1#$argon2id$v=19$m=19456,t=2,p=1$iPZ4/KgfN/W2Sm+G94jWgg$Y5rYVQ5VELviwJjBv/epA0PJYl3fD/UkNZrSAauEpbQ', -- 密码为：2024.Fusiondata
        1,
        current_timestamp),
       (10000,
        '#1#$argon2id$v=19$m=19456,t=2,p=1$m50K7TsFDob8KvtrAFbUPA$y25MggMqyp1tJ8011765alg+TM9v1OeAQiKhFWtdh3g',
        1,
        current_timestamp);

-- 重置 user_id_seq，使新用户注册从ID为 10001 开始
alter sequence user_id_seq restart 10001;

insert into user_tenant_rel (user_id, tenant_id, cid, ctime)
values (1, 1, 1, current_timestamp),
       (10000, 1, 1, current_timestamp);

--
-- 初始化数据
insert into role (id, name, description, status, cid, ctime)
values (1, '超级管理员', '拥有所有权限的角色', 100, 1, current_timestamp),
       (2, '普通用户', '基本权限的角色', 100, 1, current_timestamp);

insert into permission (id, code, description, resource, action, cid, ctime)
values (1, '用户查看', '查看用户信息的权限', 'user', 'read', 1, current_timestamp),
       (2, '用户创建', '创建用户的权限', 'user', 'create', 1, current_timestamp),
       (3, '用户更新', '更新用户信息的权限', 'user', 'update', 1, current_timestamp),
       (4, '用户删除', '删除用户的权限', 'user', 'delete', 1, current_timestamp);

--
-- 为超级管理员分配所有权限
insert into role_permission (role_id, permission_id, cid, ctime)
values (1, 1, 1, current_timestamp),
       (1, 2, 1, current_timestamp),
       (1, 3, 1, current_timestamp),
       (1, 4, 1, current_timestamp);

--
-- 为普通用户分配查看权限
insert into role_permission (role_id, permission_id, cid, ctime)
values (2, 1, 1, current_timestamp);

--
-- 为现有用户分配角色
insert into user_role (user_id, role_id, cid, ctime)
values (1, 1, 1, current_timestamp), -- 超管用户分配超级管理员角色
       (10000, 2, 1, current_timestamp);

-- 普通用户分配普通用户角色
--
-- 重置序列
alter sequence role_id_seq restart 3;

alter sequence permission_id_seq restart 5;

-- --------
-- -- ABAC init datas
-- --------
-- -- 插入示例属性
-- INSERT INTO attribute (name, description, entity_type, data_type, cid, ctime)
-- VALUES ('department', '用户所属部门', 'user', 'string', 1, CURRENT_TIMESTAMP),
--        ('clearance_level', '用户安全等级', 'user', 'integer', 1, CURRENT_TIMESTAMP),
--        ('document_classification', '文档分类级别', 'resource', 'string', 1, CURRENT_TIMESTAMP),
--        ('time_of_day', '当前时间', 'environment', 'string', 1, CURRENT_TIMESTAMP);
-- -- 插入示例策略
-- INSERT INTO policy (name, description, condition, effect, resource, action, cid, ctime)
-- VALUES ('高级文档访问策略', '允许高安全等级用户访问机密文档',
--         '{"user.clearance_level": {"gte": 4}, "resource.document_classification": "confidential", "environment.time_of_day": {"between": ["09:00", "17:00"]}}',
--         'allow', 'document', 'read', 1, CURRENT_TIMESTAMP);
-- -- 插入示例属性值
-- INSERT INTO attribute_value (attribute_id, entity_id, value, cid, ctime)
-- VALUES ((SELECT id FROM attribute WHERE name = 'department'), 1, 'IT', 1, CURRENT_TIMESTAMP),
--        ((SELECT id FROM attribute WHERE name = 'clearance_level'), 1, '5', 1, CURRENT_TIMESTAMP);
