-- tenant
create table if not exists iam_tenant (
  id bigserial not null,
  name varchar(255) not null,
  description text,
  status integer not null default 99, -- 99: Disabled, 100: Active
  created_by bigint not null,
  created_at timestamptz not null default now(),
  updated_by bigint,
  updated_at timestamptz,
  constraint iam_tenant_pk primary key (id)
);

-- namespace
create table if not exists iam_namespace (
  id bigserial not null,
  tenant_id bigint not null,
  name varchar(255) not null,
  description text,
  status integer not null default 99, -- 99: Disabled, 100: Active
  created_by bigint not null,
  created_at timestamptz not null default now(),
  updated_by bigint,
  updated_at timestamptz,
  constraint iam_namespace_fk_tenant foreign key (tenant_id) references iam_tenant (id),
  constraint iam_namespace_fk_created_by foreign key (created_by) references iam_user (id),
  constraint iam_namespace_fk_updated_by foreign key (updated_by) references iam_user (id),
  constraint iam_namespace_pk primary key (id)
);

-- namespace indexes
create index if not exists iam_namespace_idx_tenant_id on iam_namespace(tenant_id);
create index if not exists iam_namespace_idx_created_by on iam_namespace(created_by);
create index if not exists iam_namespace_idx_updated_by on iam_namespace(updated_by);
create index if not exists iam_namespace_idx_status on iam_namespace(status);
create unique index if not exists iam_namespace_idx_tenant_name on iam_namespace(tenant_id, name) where status = 100;

-- user
create table if not exists iam_user (
  id bigserial not null,
  email text,
  phone text,
  name text not null,
  status integer not null default 1,
  gender integer not null default 0,
  permission_boundary_policy_id bigint,
  created_by bigint not null,
  created_at timestamptz not null default now(),
  updated_by bigint,
  updated_at timestamptz,
  logical_deletion timestamptz,
  constraint iam_user_pk primary key (id),
  constraint iam_user_fk_permission_boundary foreign key (permission_boundary_policy_id) references iam_policy (id)
);

-- user indexes
create index if not exists iam_user_idx_permission_boundary_policy_id on iam_user(permission_boundary_policy_id);
create index if not exists iam_user_idx_created_by on iam_user(created_by);
create index if not exists iam_user_idx_updated_by on iam_user(updated_by);
create index if not exists iam_user_idx_status on iam_user(status);

-- Unified unique constraints (Central responsibility)
create unique index if not exists iam_user_uidx_email on iam_user (email)
where
  email is not null;

create unique index if not exists iam_user_uidx_phone on iam_user (phone)
where
  phone is not null;

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

-- user credential indexes
create index if not exists iam_user_credential_idx_tenant_id on iam_user_credential(tenant_id);
create index if not exists iam_user_credential_idx_created_by on iam_user_credential(created_by);
create index if not exists iam_user_credential_idx_updated_by on iam_user_credential(updated_by);

comment on column iam_user_credential.token_seq is 'Token sequence; increments by 1 on password change, used to invalidate all tokens';

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

-- tenant user association indexes
create index if not exists iam_tenant_user_idx_status on iam_tenant_user (status);
create index if not exists iam_tenant_user_idx_created_by on iam_tenant_user (created_by);
create index if not exists iam_tenant_user_idx_updated_by on iam_tenant_user (updated_by);

-- policy (混合架构：resource中不包含tenant_id，由运行时根据用户上下文注入)
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

-- policy indexes
create index if not exists iam_policy_idx_created_by on iam_policy(created_by);
create index if not exists iam_policy_idx_updated_by on iam_policy(updated_by);
create index if not exists iam_policy_idx_logical_deletion on iam_policy(logical_deletion) where logical_deletion is not null;

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

-- permission indexes
create index if not exists iam_permission_idx_created_by on iam_permission(created_by);
create index if not exists iam_permission_idx_updated_by on iam_permission(updated_by);
create index if not exists iam_permission_idx_logical_deletion on iam_permission(logical_deletion) where logical_deletion is not null;

-- role
create table if not exists iam_role (
  id bigserial not null,
  tenant_id bigint not null,
  name text not null,
  description text,
  status int not null default 99, -- 99: Disabled, 100: Active
  created_by bigint not null,
  created_at timestamptz not null default now(),
  updated_by bigint,
  updated_at timestamptz,
  constraint iam_role_pk primary key (id)
);

-- role indexes
create index if not exists iam_role_idx_created_by on iam_role(created_by);
create index if not exists iam_role_idx_updated_by on iam_role(updated_by);
create index if not exists iam_role_idx_status on iam_role(status);
create unique index if not exists iam_role_idx_tenant_name on iam_role(tenant_id, name) where status = 100;

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

-- user role indexes
create index if not exists iam_user_role_idx_tenant_id on iam_user_role(tenant_id);
create index if not exists iam_user_role_idx_created_by on iam_user_role(created_by);
create index if not exists iam_user_role_idx_user_tenant on iam_user_role(user_id, tenant_id);

-- policy attachment
create table if not exists iam_policy_attachment (
  id bigserial not null,
  tenant_id bigint not null,
  principal_type integer not null, -- 1: user, 2: role
  principal_id bigint not null,
  policy_id bigint not null,
  attachment_type integer not null default 1, -- 1: direct, 2: inherited
  created_by bigint not null,
  created_at timestamptz not null default now(),
  constraint iam_policy_attachment_pk primary key (id),
  constraint iam_policy_attachment_fk_tenant_id foreign key (tenant_id) references iam_tenant (id) on delete cascade,
  constraint iam_policy_attachment_fk_policy_id foreign key (policy_id) references iam_policy (id) on delete cascade
);

-- policy attachment indexes
create index if not exists iam_policy_attachment_idx_principal_id on iam_policy_attachment(principal_id);
create index if not exists iam_policy_attachment_idx_created_by on iam_policy_attachment(created_by);

-- session policy
create table if not exists iam_session_policy (
  id bigserial not null,
  token_id varchar(255) not null,
  tenant_id bigint not null,
  user_id bigint not null,
  policy_id bigint not null,
  expires_at timestamptz not null,
  created_at timestamptz not null default now(),
  constraint iam_session_policy_pk primary key (id),
  constraint iam_session_policy_fk_tenant_id foreign key (tenant_id) references iam_tenant (id) on delete cascade,
  constraint iam_session_policy_fk_policy_id foreign key (policy_id) references iam_policy (id) on delete cascade
);

-- session policy indexes
create index if not exists iam_session_policy_idx_user_id on iam_session_policy (user_id);
create index if not exists iam_session_policy_idx_token_id on iam_session_policy (token_id);
create index if not exists iam_session_policy_idx_expires_at on iam_session_policy (expires_at);

-- policy attachment existing indexes
create index if not exists iam_policy_attachment_idx_tenant_id on iam_policy_attachment (tenant_id);
create index if not exists iam_policy_attachment_idx_principal on iam_policy_attachment (principal_type, principal_id);

-- tenant related indexes
create index if not exists iam_role_idx_tenant_id on iam_role (tenant_id);
create index if not exists iam_permission_idx_tenant_id on iam_permission (tenant_id);
create index if not exists iam_policy_idx_tenant_id on iam_policy (tenant_id);

-- IAM Resource Mapping table (混合架构：统一使用路径码，resource_tpl为简化格式不含tenant_id)
create table iam_resource_mapping (
    id BIGSERIAL PRIMARY KEY,
    mapping_code varchar(100) unique,
    service VARCHAR(50) NOT NULL,
    path_pattern VARCHAR(1024) NOT NULL,
    method VARCHAR(10) NOT NULL,
    action VARCHAR(100) NOT NULL,
    resource_tpl VARCHAR(500) NOT NULL,
    mapping_params JSONB,
    enabled BOOLEAN DEFAULT true,
    tenant_id bigint,  -- 支持租户隔离的映射配置
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_by BIGINT NOT NULL,
    updated_by BIGINT,
    description TEXT,
    UNIQUE(service, path_pattern, method)
);

create index iam_resource_mapping_idx_lookup ON iam_resource_mapping(service, method, enabled, tenant_id);
create index iam_resource_mapping_idx_pattern ON iam_resource_mapping(service, path_pattern, tenant_id);
create index iam_resource_mapping_idx_updated_at ON iam_resource_mapping(updated_at);
create index iam_resource_mapping_idx_code ON iam_resource_mapping(mapping_code);
create index iam_resource_mapping_idx_service_path ON iam_resource_mapping(service, path_pattern, method, tenant_id);
create index iam_resource_mapping_idx_tenant_id ON iam_resource_mapping(tenant_id);
create index iam_resource_mapping_idx_created_by ON iam_resource_mapping(created_by);

-- Resource Mapping Cache table
create table resource_mapping_cache (
    cache_key VARCHAR(255) PRIMARY KEY,
    service VARCHAR(50) NOT NULL,
    mapping_response JSONB NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

create index resource_mapping_cache_idx_expires_at ON resource_mapping_cache(expires_at);
create index resource_mapping_cache_idx_service ON resource_mapping_cache(service);
