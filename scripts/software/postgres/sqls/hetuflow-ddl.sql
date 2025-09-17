-- distributed_lock 表
create table if not exists distributed_lock (
  id varchar(255) primary key, -- 锁标识
  value varchar(255) not null, -- 锁的值（可用于表示锁的持有者，比如使用：节点ID）
  locked_at timestamptz not null default now(), -- 锁获取时间
  expires_at timestamptz not null default now() + interval '60 seconds', -- 锁过期时间
  token bigserial, -- 独立序列
  constraint chk_distributed_lock_expires_at check (expires_at >= locked_at) -- 过期时间必须大于锁的时间
);

-- sched_server 表
create table sched_server (
  id varchar(40) primary key,
  name varchar(255) not null,
  address varchar(255) not null,
  bind_namespaces uuid[] not null default '{}',
  status int not null default 100, -- 见 ServerStatus
  description text,
  last_heartbeat timestamptz not null default now(),
  created_by bigint not null,
  created_at timestamptz not null default now(),
  updated_by bigint,
  updated_at timestamptz,
  deleted_at timestamptz
);

-- Agent 管理表 (sched_agent)
create table sched_agent (
  id varchar(40) primary key, -- Agent ID，由客户端生成
  description text,
  address varchar(255) not null,
  status int not null default 100, -- 见 AgentStatus
  capabilities jsonb not null, -- Agent 能力描述
  last_heartbeat timestamptz not null default now()
);

-- indexes for sched_agent
create index if not exists idx_sched_agent_status on sched_agent (status);

create index if not exists idx_sched_agent_last_heartbeat on sched_agent (last_heartbeat);

-- 作业定义表 (sched_job)
create table sched_job (
  id uuid primary key,
  namespace_id uuid not null default '00000000-0000-0000-0000-000000000000', -- namespace_id 由 fusion-iam 管理
  name varchar(255) not null,
  description text,
  -- 运行作业时添加的自定义环境变量
  environment jsonb,
  config jsonb, -- TaskConfig
  status int not null default 1, -- 见 JobStatus
  created_by bigint not null,
  created_at timestamptz not null default now(),
  updated_by bigint,
  updated_at timestamptz,
  deleted_at timestamptz
);

-- 调度策略表 (sched_schedule)
create table sched_schedule (
  id uuid primary key,
  job_id uuid not null references sched_job (id) on delete cascade,
  name varchar(255),
  description text,
  schedule_kind int not null, -- 见 ScheduleKind 枚举
  start_time timestamptz,
  end_time timestamptz,
  -- 其他特定于 schedule 的参数可以放这里
  status int not null default 1, -- 见 ScheduleStatus 枚举
  cron_expression varchar(100), -- ScheduleKind::Cron 时有效
  interval_secs int, -- ScheduleKind::Interval 时有效
  max_count int, -- ScheduleKind::Interval 时有效
  next_run_at timestamptz, -- 计算出的下一次执行时间
  created_by bigint not null,
  created_at timestamptz not null default now(),
  updated_by bigint,
  updated_at timestamptz
);

-- 任务计划表 (sched_task)
create table sched_task (
  id uuid primary key,
  job_id uuid not null references sched_job (id),
  namespace_id uuid not null default '00000000-0000-0000-0000-000000000000', -- namespace_id 由 fusion-iam 管理
  priority int not null default 0, -- 任务优先级，数值越大优先级越高
  status int not null default 1, -- 见 TaskStatus 枚举
  schedule_id uuid references sched_schedule (id), -- 为空时表示手动触发：Event, Flow
  scheduled_at timestamptz not null, -- 计划执行时间
  schedule_kind int not null, -- 见 ScheduleKind 枚举
  completed_at timestamptz, -- 任务完成时间，具体的任务明细见对应的 sched_task_instance 表
  parameters jsonb not null default '{}'::jsonb, -- 任务参数
  environment jsonb, -- 任务环境变量
  config jsonb, -- 任务配置
  retry_count int not null default 0, -- 重试次数
  max_retries int not null default 3, -- 最大重试次数
  locked_at timestamptz,
  lock_version int not null default 0,
  dependencies jsonb, -- 任务依赖关系
  created_by bigint not null,
  created_at timestamptz not null default now(),
  updated_by bigint,
  updated_at timestamptz
);

-- 创建索引优化查询性能
create index if not exists idx_sched_task_status_scheduled on sched_task (status, scheduled_at);

create index if not exists idx_sched_task_lock_timeout on sched_task (locked_at)
where
  status in (10, 20);

-- 任务执行实例表 (sched_task_instance)
create table sched_task_instance (
  id uuid primary key,
  task_id uuid not null references sched_task (id),
  job_id uuid not null references sched_job (id), -- 绑定的 Job ID，用于任务分发
  agent_id varchar(40) not null references sched_agent (id), -- 绑定的 Agent ID，用于任务执行
  status int not null default 1, -- 见 TaskInstanceStatus 枚举
  started_at timestamptz not null, -- 任务实例开始（计划）时间，实际运行时可能会有微小的偏差
  completed_at timestamptz,
  output text,
  error_message text,
  exit_code int,
  metrics jsonb, -- TaskMetrics
  created_at timestamptz default now(),
  updated_by bigint,
  updated_at timestamptz
);

-- indexes for sched_task_instance
create index if not exists idx_sched_task_instance_task_id on sched_task_instance (task_id);

create index if not exists idx_sched_task_instance_agent_id on sched_task_instance (agent_id);

create index if not exists idx_sched_task_instance_status on sched_task_instance (status);
