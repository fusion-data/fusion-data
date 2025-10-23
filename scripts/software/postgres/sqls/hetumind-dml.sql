set
  timezone to 'Asia/Chongqing';

-- 注意：新的 user_entity 表结构中 id 字段不再使用序列，而是使用 jieyuan.iam_user.id
-- 这里使用一个默认的 tenant_id (bigint)，对应 iam_tenant.id，实际使用时应该从令牌 claims 中获取
insert into
  user_entity (id, email, "name", "password", status, mfa_enabled, tenant_id, created_at, created_by)
values
  (
    1,
    'admin@hetumind.com',
    'Admin',
    '#1#$argon2id$v=19$m=19456,t=2,p=1$CnoakTKSq9TZxn7TD/1Hmw$Yhff4M7dEMFl3zqNoPGnuOb1uRAcdsrqwXybdZZkeYg',
    100,
    false,
    1,
    current_timestamp,
    1
  );
