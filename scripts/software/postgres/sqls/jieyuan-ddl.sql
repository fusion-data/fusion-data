-- tenant
create table if not exists iam_tenant (
  id bigserial not null,
  name varchar(255) not null,
  description text,
  status integer not null default 1,
  created_by bigint not null,
  created_at timestamptz not null default now(),
  updated_by bigint,
  updated_at timestamptz,
  logical_deletion timestamptz,
  constraint iam_tenant_pk primary key (id)
);

-- user
create table if not exists iam_user (
  id bigserial not null,
  email text,
  phone text,
  name text not null,
  status integer not null default 1,
  gender integer not null default 0,
  created_by bigint not null,
  created_at timestamptz not null default now(),
  updated_by bigint,
  updated_at timestamptz,
  logical_deletion timestamptz,
  constraint iam_user_pk primary key (id)
);

-- user credential
create table if not exists iam_user_credential (
  id bigint not null,
  tenant_id bigint not null,
  encrypted_pwd text not null,
  token_seq integer not null default 0,
  created_by bigint not null,
  created_at timestamptz not null default now(),
  updated_by bigint,
  updated_at timestamptz,
  constraint iam_user_credential_fk_id foreign key (id) references iam_user (id),
  constraint iam_user_credential_pk primary key (id)
);

-- comment on token_seq column
comment on column iam_user_credential.token_seq is '令牌序列；密码变更时 +1，用于全量作废令牌';

-- tenant user association
create table if not exists iam_tenant_user (
  tenant_id bigint not null,
  user_id bigint not null,
  status smallint not null default 100,
  created_by bigint not null,
  created_at timestamptz not null default now(),
  updated_by bigint,
  updated_at timestamptz,
  constraint iam_tenant_user_fk_tenant_id foreign key (tenant_id) references iam_tenant (id) on delete cascade,
  constraint iam_tenant_user_fk_user_id foreign key (user_id) references iam_user (id) on delete cascade,
  constraint iam_tenant_user_pk primary key (tenant_id, user_id)
);

-- policy
create table if not exists iam_policy (
  id bigserial not null,
  tenant_id bigint not null,
  description text,
  policy jsonb not null,
  status int not null default 100,
  created_by bigint not null,
  created_at timestamptz not null default now(),
  updated_by bigint,
  updated_at timestamptz,
  logical_deletion timestamptz,
  constraint iam_policy_pk primary key (id)
);

-- permission
create table if not exists iam_permission (
  id bigserial not null,
  tenant_id bigint not null,
  code bigint not null,
  description text not null,
  resource text not null,
  action text not null,
  created_by bigint not null,
  created_at timestamptz not null default now(),
  updated_by bigint,
  updated_at timestamptz,
  logical_deletion timestamptz,
  constraint iam_permission_pk primary key (id)
);

-- role
create table if not exists iam_role (
  id bigserial not null,
  tenant_id bigint not null,
  name text not null,
  description text,
  status int not null default 100,
  created_by bigint not null,
  created_at timestamptz not null default now(),
  updated_by bigint,
  updated_at timestamptz,
  logical_deletion timestamptz,
  constraint iam_role_pk primary key (id)
);

-- user role
create table if not exists iam_user_role (
  user_id bigint not null,
  tenant_id bigint not null,
  role_id bigint not null,
  created_by bigint not null,
  created_at timestamptz not null default now(),
  constraint iam_user_role_fk_role_id foreign key (role_id) references iam_role (id),
  constraint iam_user_role_fk_user_id foreign key (user_id) references iam_user (id),
  constraint iam_user_role_pk primary key (user_id, role_id)
);

-- 统一唯一约束（中心负责）
create unique index if not exists iam_user_uidx_email on iam_user (email)
where
  email is not null;

create unique index if not exists iam_user_uidx_phone on iam_user (phone)
where
  phone is not null;

-- tenant 相关索引
create index if not exists iam_role_idx_tenant_id on iam_role (tenant_id);

create index if not exists iam_permission_idx_tenant_id on iam_permission (tenant_id);

create index if not exists iam_policy_idx_tenant_id on iam_policy (tenant_id);

create index if not exists iam_tenant_user_idx_status on iam_tenant_user (status);
