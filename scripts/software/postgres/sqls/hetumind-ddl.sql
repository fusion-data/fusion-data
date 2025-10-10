set
  timezone to 'Asia/Chongqing';

-- 用户表
create table if not exists user_entity (
  -- 用户ID，由程序端使用 uuid v7 生成
  id bigserial constraint user_pk primary key,
  -- 用户邮箱，全局唯一，登录凭证
  email varchar(255) not null,
  -- 手机号，全局唯一，登录凭证，可选。存储为完整格式（含国家/地区码），如 +8613800138000
  phone varchar(16),
  -- 用户名
  name varchar(32),
  -- 用户密码
  password varchar(255) not null,
  -- 用户个性化设置
  personalization_answers jsonb,
  -- 用户设置（系统）
  settings jsonb,
  -- 用户状态。100:正常, 99:禁用
  status int not null default 100,
  -- 多因素（MFA）认证
  mfa_enabled boolean default false not null,
  mfa_secret text,
  mfa_recovery_codes text,
  -- 用户角色
  -- role text not null,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint
);

create unique index if not exists user_uidx_email on user_entity (email);

create unique index if not exists user_uidx_phone on user_entity (phone)
where
  phone is not null;

create table if not exists auth_identity (
  user_id bigint constraint auth_identity_fk_user references user_entity,
  provider_id varchar(64) not null,
  provider_kind varchar(32) not null,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint,
  constraint auth_identity_pk primary key (provider_id, provider_kind)
);

create table if not exists user_api_key (
  id uuid not null constraint user_api_key_pk primary key,
  user_id bigint not null constraint user_api_key_fk_user references user_entity on delete cascade,
  label varchar(100) not null,
  api_key varchar not null,
  scopes jsonb,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint
);

create unique index if not exists user_api_key_uidx_api_key on user_api_key (api_key);

create unique index if not exists user_api_key_uidx_user_id_label on user_api_key (user_id, label);

create table if not exists annotation_tag_entity (
  id uuid constraint annotation_tag_entity_pk primary key,
  name varchar(24) not null,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint
);

create unique index if not exists annotation_tag_entity_uidx_name on annotation_tag_entity (name);

create table if not exists auth_provider_sync_history (
  id bigserial constraint auth_provider_sync_history_pk primary key,
  provider_kind int not null,
  run_mode text not null,
  status text not null,
  started_at timestamptz not null default current_timestamp,
  ended_at timestamptz,
  scanned integer not null,
  created integer not null,
  updated integer not null,
  disabled integer not null,
  error text
);

create table if not exists credential_entity (
  id uuid constraint credential_entity_pk primary key,
  namespace_id varchar(40) not null,
  name varchar(128) not null,
  data text not null,
  kind varchar(128) not null,
  is_managed boolean default false not null,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint,
  deleted_at timestamptz
);

create index if not exists credential_entity_idx_kind on credential_entity (kind);
create unique index if not exists credential_entity_idx_namespace_name on credential_entity (namespace_id, name);

create table if not exists event_destination (
  id uuid constraint event_destination_pk primary key,
  destination jsonb not null,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint
);

create table if not exists installed_package (
  package_name varchar(214) constraint installed_package_pk primary key,
  installed_version varchar(50) not null,
  author_name varchar(70),
  author_email varchar(70),
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint
);

create table if not exists installed_node (
  name varchar(200) constraint installed_node_pk primary key,
  kind varchar(200) not null,
  latest_version integer default 1 not null,
  package varchar(241) not null constraint installed_node_fk_package references installed_package on update cascade on delete cascade
);

create table if not exists invalid_auth_token (token varchar(512) constraint invalid_auth_token_pk primary key, expires_at timestamptz not null);

create table if not exists migrations (id serial constraint migrations_pk primary key, timestamp bigint not null, name varchar not null);

-- project
create table if not exists project (
  id uuid constraint project_pk primary key,
  name varchar(255) not null,
  kind uuid not null,
  icon jsonb,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint
);

create table if not exists folder (
  id uuid constraint folder_pk primary key,
  name varchar(128) not null,
  parent_folder_id uuid constraint folder_fk_parent_folder references folder on delete cascade,
  project_id uuid not null constraint folder_fk_project references project on delete cascade,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint
);

create unique index if not exists folder_uidx_project_id_id on folder (project_id, id);

create table if not exists setting (
  key varchar(255) constraint setting_pk primary key,
  value text not null,
  load_on_startup boolean default false not null
);

create table if not exists shared_credentials (
  credentials_id uuid not null constraint shared_credentials_fk_credentials references credential_entity on delete cascade,
  project_id uuid not null constraint shared_credentials_fk_project references project on delete cascade,
  role text not null,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint,
  constraint shared_credentials_pk primary key (credentials_id, project_id)
);

-- 标签表
create table if not exists tag_entity (
  id uuid constraint tag_entity_pk primary key,
  name varchar(24) not null,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint
);

create unique index if not exists tag_entity_uidx_name on tag_entity (name);

create table if not exists folder_tag (
  folder_id uuid not null constraint folder_tag_fk_folder references folder on delete cascade,
  tag_id uuid not null constraint folder_tag_fk_tag references tag_entity on delete cascade,
  constraint folder_tag_pk primary key (folder_id, tag_id)
);

-- 项目成员表
create table if not exists project_relation (
  project_id uuid not null constraint project_relation_fk_project references project on delete cascade,
  -- 用户ID，TODO owner？
  user_id bigint not null constraint project_relation_fk_user references user_entity on delete cascade,
  -- TODO 角色(权限)，具备此角色的用户可以访问此项目
  role varchar not null,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint,
  constraint project_relation_pk primary key (project_id, user_id)
);

create index if not exists project_relation_idx_user_id on project_relation (user_id);

create index if not exists project_relation_idx_project_id on project_relation (project_id);

create table if not exists variables (
  id uuid constraint variables_pk primary key,
  key varchar(50) not null unique,
  kind varchar(50) default 'string' not null,
  value varchar(255)
);

-- 工作流主表
create table if not exists workflow_entity (
  -- uuid-v7
  id uuid constraint workflow_entity_pk primary key,
  name varchar(128) not null,
  status int not null,
  -- 工作流节点（步骤）
  nodes jsonb not null,
  -- 工作流节点连接
  connections jsonb not null,
  -- 工作流设置
  settings jsonb not null,
  -- 工作流静态数据
  static_data jsonb,
  -- 工作流固定数据
  pin_data jsonb not null,
  -- 工作流版本ID uuid-v7
  version uuid,
  -- 工作流触发次数
  trigger_count bigint default 0 not null,
  -- 工作流元数据
  meta jsonb not null,
  parent_folder_id uuid constraint fk_workflow_parent_folder references folder on delete cascade,
  is_archived boolean default false not null,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint
);

create index if not exists workflow_entity_idx_name on workflow_entity (name);

create table if not exists workflow_tag (
  workflow_id uuid not null constraint fk_workflow_tag_workflow_id references workflow_entity on delete cascade,
  tag_id uuid not null constraint fk_workflow_tag_tag_id references tag_entity on delete cascade,
  constraint workflow_tag_pk primary key (workflow_id, tag_id)
);

create index if not exists workflow_tag_idx_workflow_id on workflow_tag (workflow_id);

-- 工作流执行记录表
-- 记录每次工作流执行的详细信息
create table if not exists execution_entity (
  id uuid constraint execution_entity_pk primary key,
  workflow_id uuid not null constraint fk_execution_entity_workflow_id references workflow_entity on delete cascade,
  finished boolean not null,
  mode varchar not null,
  -- 重试机制
  retry_of varchar,
  retry_success_id varchar,
  -- 执行开始时间
  started_at timestamptz,
  -- 执行结束时间
  stopped_at timestamptz,
  --
  wait_till timestamptz,
  status varchar not null,
  deleted_at timestamptz,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint
);

-- 工作流执行数据表。存储工作流执行过程中的数据
create table if not exists execution_data (
  execution_id uuid constraint execution_data_pk primary key constraint execution_data_fk references execution_entity on delete cascade,
  -- 包含工作流数据
  workflow_data jsonb not null,
  -- 具体执行数据
  data text not null
);

create index if not exists execution_entity_idx_stopped_at_status_deleted_at on execution_entity (stopped_at, status, deleted_at)
where
  (
    (stopped_at is not null)
    and (deleted_at is null)
  );

create index if not exists idx_execution_entity_wait_till_status_deleted_at on execution_entity (wait_till, status, deleted_at)
where
  (
    (wait_till is not null)
    and (deleted_at is null)
  );

create index if not exists idx_execution_entity_workflow_id_started_at on execution_entity (workflow_id, started_at)
where
  (
    (started_at is not null)
    and (deleted_at is null)
  );

create index if not exists execution_entity_idx_deleted_at on execution_entity (deleted_at);

create table if not exists execution_annotation (
  id uuid constraint execution_annotation_pk primary key,
  execution_id uuid not null constraint execution_annotation_fk_execution_entity references execution_entity on delete cascade,
  vote varchar(6),
  note text,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint
);

create unique index if not exists execution_annotation_uidx_execution_id on execution_annotation (execution_id);

create table if not exists execution_annotation_tag (
  annotation_id uuid not null constraint execution_annotation_tag_fk_execution_annotation references execution_annotation on delete cascade,
  tag_id uuid not null constraint execution_annotation_tag_fk_annotation_tag_entity references annotation_tag_entity on delete cascade,
  constraint execution_annotation_tag_pk primary key (annotation_id, tag_id)
);

create index if not exists execution_annotation_tag_idx_tag_id on execution_annotation_tag (tag_id);

create index if not exists execution_annotation_tag_idx_annotation_id on execution_annotation_tag (annotation_id);

create table if not exists execution_metadata (
  id uuid constraint execution_metadata_pk primary key,
  execution_id uuid not null constraint execution_metadata_fk_execution_entity references execution_entity on delete cascade,
  key varchar(255) not null,
  value text not null
);

create unique index if not exists execution_metadata_uidx_execution_id_key on execution_metadata (execution_id, key);

create table if not exists insight_metadata (
  meta_id serial constraint insight_metadata_pk primary key,
  workflow_id uuid constraint insight_metadata_fk_workflow_entity references workflow_entity on delete set null,
  project_id uuid constraint insight_metadata_fk_project references project on delete set null,
  workflow_name varchar(128) not null,
  project_name varchar(255) not null
);

create table if not exists insight_by_period (
  id serial constraint insight_by_period_pk primary key,
  meta_id integer not null constraint insight_by_period_fk_insight_metadata references insight_metadata on delete cascade,
  kind integer not null,
  value integer not null,
  period_unit integer not null,
  period_start timestamptz default current_timestamp
);

comment on column insight_by_period.kind is '0: time_saved_minutes, 1: runtime_milliseconds, 2: success, 3: failure';

comment on column insight_by_period.period_unit is '0: hour, 1: day, 2: week';

create unique index if not exists insight_by_period_uidx_period_start_type_period_unit_meta_id on insight_by_period (period_start, kind, period_unit, meta_id);

create unique index if not exists insight_metadata_uidx_workflow_id on insight_metadata (workflow_id);

create table if not exists insight_raw (
  id serial constraint insight_raw_pk primary key,
  meta_id integer not null constraint insight_raw_fk_insight_metadata references insight_metadata on delete cascade,
  kind integer not null,
  value integer not null,
  created_at timestamptz not null
);

comment on column insight_raw.kind is '0: time_saved_minutes, 1: runtime_milliseconds, 2: success, 3: failure';

create table if not exists processed_data (
  workflow_id uuid not null constraint processed_data_fk_workflow_entity references workflow_entity on delete cascade,
  context varchar(255) not null,
  value text not null,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint,
  constraint processed_data_pk primary key (workflow_id, context)
);

-- 共享工作流表。记录工作流被共享到哪些项目中，以及共享的权限
create table if not exists shared_workflow (
  workflow_id uuid not null constraint shared_workflow_fk_workflow_entity references workflow_entity on delete cascade,
  project_id uuid not null constraint shared_workflow_fk_project references project on delete cascade,
  role text not null,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint,
  constraint shared_workflow_pk primary key (workflow_id, project_id)
);

create table if not exists workflow_history (
  version_id uuid constraint workflow_history_pk primary key,
  workflow_id uuid not null constraint workflow_history_fk_workflow_entity references workflow_entity on delete cascade,
  authors varchar(255) not null,
  nodes jsonb not null,
  connections jsonb not null,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint
);

create index if not exists workflow_history_idx_workflow_id on workflow_history (workflow_id);

create table if not exists workflow_statistics (
  count integer default 0,
  latest_event timestamptz,
  name varchar(128) not null,
  workflow_id uuid not null constraint fk_workflow_statistics_workflow_id references workflow_entity on delete cascade,
  root_count integer default 0,
  constraint workflow_statistics_pk primary key (workflow_id, name)
);

-- 任务队列主表
create table if not exists task_queue (
  id uuid not null constraint task_queue_pk primary key,
  task_kind int not null,
  execution_id uuid not null,
  workflow_id uuid not null constraint task_queue_fk_workflow_entity references workflow_entity on delete cascade,
  priority int not null default 1,
  status int not null default 1,
  payload jsonb not null,
  result jsonb,
  error_message text,
  retry_count int not null default 0,
  max_retries int not null default 3,
  scheduled_at timestamptz not null default current_timestamp,
  started_at timestamptz,
  completed_at timestamptz,
  worker_id uuid,
  heartbeat_at timestamptz,
  metadata jsonb,
  created_at timestamptz not null default current_timestamp,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint
);

-- 索引优化查询性能
create index idx_task_queue_status_priority on task_queue (status, priority desc, scheduled_at)
where
  status in (1, 10);

create index idx_task_queue_execution_id on task_queue (execution_id);

create index idx_task_queue_worker_heartbeat on task_queue (worker_id, heartbeat_at)
where
  status = 10;

create table if not exists test_definition (
  id uuid constraint test_definition_pk primary key,
  name varchar(255) not null,
  workflow_id uuid not null constraint test_definition_fk_workflow_entity references workflow_entity on delete cascade,
  evaluation_workflow_id uuid constraint test_definition_fk_workflow_entity_evaluation references workflow_entity on delete set null,
  annotation_tag_id uuid constraint test_definition_fk_annotation_tag_entity references annotation_tag_entity on delete set null,
  description text,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint,
  mocked_nodes jsonb default '[]'::jsonb not null
);

create index if not exists test_definition_idx_evaluation_workflow_id on test_definition (evaluation_workflow_id);

create index if not exists test_definition_idx_workflow_id on test_definition (workflow_id);

create table if not exists test_metric (
  id uuid constraint test_metric_pk primary key,
  name varchar(255) not null,
  test_definition_id uuid not null constraint test_metric_fk_test_definition references test_definition on delete cascade,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint
);

create index if not exists test_metric_idx_test_definition_id on test_metric (test_definition_id);

create table if not exists test_run (
  id uuid constraint test_run_pk primary key,
  test_definition_id uuid not null constraint test_run_fk_test_definition references test_definition on delete cascade,
  status varchar not null,
  run_at timestamptz,
  completed_at timestamptz,
  metrics json,
  total_cases integer,
  passed_cases integer,
  failed_cases integer,
  error_code varchar(255),
  error_details text,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint,
  constraint test_run_check check (
    case
      when ((status)::text = 'new'::text) then (total_cases is null)
      when ((status)::text = any (array[('cancelled'::character varying)::text, ('error'::character varying)::text])) then (
        (total_cases is null)
        or (total_cases >= 0)
      )
      else (total_cases >= 0)
    end
  ),
  constraint test_run_check1 check (
    case
      when ((status)::text = 'new'::text) then (passed_cases is null)
      when ((status)::text = any (array[('cancelled'::character varying)::text, ('error'::character varying)::text])) then (
        (passed_cases is null)
        or (passed_cases >= 0)
      )
      else (passed_cases >= 0)
    end
  ),
  constraint test_run_check2 check (
    case
      when ((status)::text = 'new'::text) then (failed_cases is null)
      when ((status)::text = any (array[('cancelled'::character varying)::text, ('error'::character varying)::text])) then (
        (failed_cases is null)
        or (failed_cases >= 0)
      )
      else (failed_cases >= 0)
    end
  )
);

create table if not exists test_case_execution (
  id uuid constraint test_case_execution_pk primary key,
  test_run_id uuid not null constraint test_case_execution_fk_test_run_id references test_run on delete cascade,
  past_execution_id uuid constraint test_case_execution_fk_past_execution_id references execution_entity on delete set null,
  execution_id uuid constraint test_case_execution_fk_execution_id references execution_entity on delete set null,
  evaluation_execution_id uuid constraint test_case_execution_fk_evaluation_execution_id references execution_entity on delete set null,
  status varchar not null,
  run_at timestamptz,
  completed_at timestamptz,
  error_code varchar,
  error_details json,
  metrics json,
  created_at timestamptz not null,
  created_by bigint not null,
  updated_at timestamptz,
  updated_by bigint
);

create table if not exists webhook_entity (
  webhook_path varchar not null,
  method varchar not null,
  node varchar not null,
  webhook_id varchar,
  path_length integer,
  workflow_id uuid not null constraint fk_webhook_entity_workflow_id references workflow_entity on delete cascade,
  constraint webhook_entity_pk primary key (webhook_path, method)
);

create index if not exists webhook_entity_idx_webhook_id_method_path_length on webhook_entity (webhook_id, method, path_length);
