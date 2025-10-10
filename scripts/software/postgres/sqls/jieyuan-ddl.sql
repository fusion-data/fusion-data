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
  encrypted_pwd text not null,
  created_by bigint not null,
  created_at timestamptz not null default now(),
  updated_by bigint,
  updated_at timestamptz,
  constraint iam_user_credential_fk_id foreign key (id) references iam_user (id),
  constraint iam_user_credential_pk primary key (id)
);

-- policy
create table if not exists iam_policy (
  id bigserial not null,
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
  role_id bigint not null,
  created_by bigint not null,
  created_at timestamptz not null default now(),
  constraint iam_user_role_fk_role_id foreign key (role_id) references iam_role (id),
  constraint iam_user_role_fk_user_id foreign key (user_id) references iam_user (id),
  constraint iam_user_role_pk primary key (user_id, role_id)
);
