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
  permission_boundary_policy_id bigint,
  created_by bigint not null,
  created_at timestamptz not null default now(),
  updated_by bigint,
  updated_at timestamptz,
  logical_deletion timestamptz,
  constraint iam_user_pk primary key (id),
  constraint iam_user_fk_permission_boundary foreign key (permission_boundary_policy_id) references iam_policy (id)
);

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
create index if not exists iam_tenant_user_idx_status on iam_tenant_user (status);

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
create index if not exists iam_policy_attachment_idx_tenant_id on iam_policy_attachment (tenant_id);
create index if not exists iam_policy_attachment_idx_principal on iam_policy_attachment (principal_type, principal_id);

create index if not exists iam_session_policy_idx_token_id on iam_session_policy (token_id);
create index if not exists iam_session_policy_idx_expires_at on iam_session_policy (expires_at);

-- tenant related indexes
create index if not exists iam_role_idx_tenant_id on iam_role (tenant_id);
create index if not exists iam_permission_idx_tenant_id on iam_permission (tenant_id);
create index if not exists iam_policy_idx_tenant_id on iam_policy (tenant_id);

-- IAM Resource Mapping table
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
    tenant_id bigint,  -- Added tenant isolation support
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_by BIGINT NOT NULL,
    updated_by BIGINT,
    description TEXT,
    UNIQUE(service, path_pattern, method)
);

-- Resource Mapping Cache table
create table resource_mapping_cache (
    cache_key VARCHAR(255) PRIMARY KEY,
    service VARCHAR(50) NOT NULL,
    mapping_response JSONB NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 索引优化
create index idx_iam_resource_mapping_lookup ON iam_resource_mapping(service, method, enabled, tenant_id);
create index idx_iam_resource_mapping_pattern ON iam_resource_mapping(service, path_pattern, tenant_id);
create index idx_iam_resource_mapping_updated_at ON iam_resource_mapping(updated_at);
create index idx_iam_resource_mapping_code ON iam_resource_mapping(mapping_code);
create index idx_iam_resource_mapping_service_path ON iam_resource_mapping(service, path_pattern, method, tenant_id);
create index idx_iam_resource_mapping_tenant_id ON iam_resource_mapping(tenant_id);

-- Resource Mapping Cache indexes
create index idx_resource_mapping_cache_expires_at ON resource_mapping_cache(expires_at);
create index idx_resource_mapping_cache_service ON resource_mapping_cache(service);
