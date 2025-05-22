-- 创建用户表
create table users (
  id bigint primary key,
  username varchar not null,
  email varchar not null,
  password_hash varchar not null,
  role varchar not null,
  cid bigint,
  ctime timestamptz,
  mid bigint,
  mtime timestamptz
);

-- 创建凭证表
create table credentials (
  id bigint primary key,
  user_id int references users (id),
  name varchar not null,
  type varchar not null,
  encrypted_data varchar not null,
  cid bigint,
  ctime timestamptz,
  mid bigint,
  mtime timestamptz
);

-- 创建工作流表
create table workflows (
  id bigint primary key,
  user_id int references users (id),
  name varchar not null,
  description text,
  definition jsonb,
  is_active boolean,
  cid bigint,
  ctime timestamptz,
  mid bigint,
  mtime timestamptz
);

-- 创建工作流版本表
create table workflow_versions (
  id bigint primary key,
  workflow_id int references workflows (id),
  version_number int not null,
  definition jsonb,
  cid bigint references users (id),
  ctime timestamptz
);

-- 创建工作流执行表
create table workflow_executions (
  id bigint primary key,
  workflow_id int references workflows (id),
  workflow_version_id int references workflow_versions (id),
  status varchar not null,
  triggered_by varchar not null,
  start_time timestamptz,
  end_time timestamptz,
  input_data jsonb,
  output_data jsonb,
  error_message text,
  cid bigint,
  ctime timestamptz
);

-- 创建执行日志表
create table execution_logs (
  id bigint primary key,
  execution_id int references workflow_executions (id),
  node_id_in_workflow int not null,
  node_name varchar not null,
  status varchar not null,
  input_data jsonb,
  output_data jsonb,
  error_message text,
  cid bigint,
  ctime timestamptz
);

-- 创建计划任务表
create table scheduled_tasks (
  id bigint primary key,
  workflow_id int references workflows (id),
  cron_expression varchar not null,
  next_run_time timestamptz,
  is_active boolean,
  last_run_status varchar,
  cid bigint,
  ctime timestamptz
);

-- 创建节点表
create table nodes (
  id bigint primary key,
  name varchar not null,
  type varchar not null,
  description text,
  icon varchar,
  parameters_schema jsonb,
  package_url varchar,
  cid bigint,
  ctime timestamptz
);

-- 创建提示模板表
create table prompt_templates (
  id bigint primary key,
  user_id int references users (id),
  name varchar not null,
  template_text text not null,
  variables jsonb,
  cid bigint,
  ctime timestamptz,
  mid bigint,
  mtime timestamptz
);
