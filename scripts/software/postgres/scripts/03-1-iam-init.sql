set timezone to 'Asia/Chongqing';
\c fusiondata;
\c - fusiondata;

--
-- initial data
------------------
insert into iam."user" (id, email, phone, name, status, gender, cid, ctime)
values (1, 'admin@fusiondata.com', null, '超管', 100, 0, 1, current_timestamp),
       (10000, 'user@fusiondata.com', '13912345678', '普通用户', 100, 0, 1, current_timestamp);
insert into iam.user_credential (id, encrypted_pwd, cid, ctime)
values (1,
        '#1#$argon2id$v=19$m=19456,t=2,p=1$iPZ4/KgfN/W2Sm+G94jWgg$Y5rYVQ5VELviwJjBv/epA0PJYl3fD/UkNZrSAauEpbQ', -- 密码为：2024.Fusiondata
        1, current_timestamp),
       (10000, '#1#$argon2id$v=19$m=19456,t=2,p=1$m50K7TsFDob8KvtrAFbUPA$y25MggMqyp1tJ8011765alg+TM9v1OeAQiKhFWtdh3g',
        1, current_timestamp);
-- 重置 user_id_seq，使新用户注册从ID为 10001 开始
alter sequence iam.user_id_seq restart 10001;
--
-- 初始化数据
insert into iam.role (id, name, description, status, cid, ctime)
values (1, '超级管理员', '拥有所有权限的角色', 100, 1, current_timestamp),
       (2, '普通用户', '基本权限的角色', 100, 1, current_timestamp);

insert into iam.permission (id, code, description, resource, action, cid, ctime)
values (1, '用户查看', '查看用户信息的权限', 'user', 'read', 1, current_timestamp),
       (2, '用户创建', '创建用户的权限', 'user', 'create', 1, current_timestamp),
       (3, '用户更新', '更新用户信息的权限', 'user', 'update', 1, current_timestamp),
       (4, '用户删除', '删除用户的权限', 'user', 'delete', 1, current_timestamp);
--
-- 为超级管理员分配所有权限
insert into iam.role_permission (role_id, permission_id, cid, ctime)
values (1, 1, 1, current_timestamp),
       (1, 2, 1, current_timestamp),
       (1, 3, 1, current_timestamp),
       (1, 4, 1, current_timestamp);
--
-- 为普通用户分配查看权限
insert into iam.role_permission (role_id, permission_id, cid, ctime)
values (2, 1, 1, current_timestamp);
--
-- 为现有用户分配角色
insert into iam.user_role (user_id, role_id, cid, ctime)
values (1, 1, 1, current_timestamp), -- 超管用户分配超级管理员角色
       (10000, 2, 1, current_timestamp);
-- 普通用户分配普通用户角色
--
-- 重置序列
alter sequence iam.role_id_seq restart 3;
alter sequence iam.permission_id_seq restart 5;
