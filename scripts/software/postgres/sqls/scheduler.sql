set
    timezone to 'Asia/Chongqing';

-- create schema
create schema if not exists sched;

--
-- sched_node
create table if not exists sched.sched_node
(
    -- 节点ID
    id              varchar(32) not null,
    kind            int         not null,
    -- 节点通信地址
    addr            varchar(64) not null,
    last_check_time timestamptz,
    unhealth_count  int         not null default 0,
    status          int         not null,
    cid             bigint      not null,
    ctime           timestamptz not null,
    mid             bigint,
    mtime           timestamptz,
    constraint sched_node_pk primary key (id)
);

comment on column sched.sched_node.last_check_time is '节点最后检查时间';

--
-- global_path
create table if not exists sched.global_path
(
    path     varchar(1024) not null,
    value    varchar(4096),
    revision bigint        not null default 0,
    constraint lock_path_pk primary key (path)
);

comment on table sched.global_path is '全局路径，用于选主、分布式锁等功能';

--
-- sched_namespace 调度分组
create table if not exists sched.sched_namespace
(
    id        bigserial    not null,
    tenant_id int          not null references iam.tenant (id),
    namespace varchar(512) not null unique,
    node_id   varchar(32) references sched.sched_node (id),
    status    int          not null,
    cid       bigint       not null,
    ctime     timestamptz  not null,
    mid       bigint,
    mtime     timestamptz,
    constraint sched_group_pk primary key (id)
);

create unique index sched_group_uidx_name on sched.sched_namespace (tenant_id, namespace);

comment on column sched.sched_namespace.namespace is '调度分组名，需要租户下唯一。可以按应用或服务进行划分';

comment on column sched.sched_namespace.node_id is '节点ID';

--
-- 流程定义
create table if not exists sched.process_definition
(
    id           bigserial   not null,
    tenant_id    int         not null references iam.tenant (id),
    namespace_id bigint      not null references sched.sched_namespace (id),
    key          varchar     not null,
    description  varchar,
    tags         varchar[]   not null default '{}',
    variables    jsonb,
    data         bytea,
    status       int         not null default 100,
    cid          bigint      not null,
    ctime        timestamptz not null,
    mid          bigint,
    mtime        timestamptz,
    constraint process_definition_pk primary key (id)
);

create unique index process_definition_uidx_key on sched.process_definition (namespace_id, key);

--
-- 触发器定义
create table if not exists sched.trigger_definition
(
    id                 bigserial   not null,
    tenant_id          int         not null references iam.tenant (id),
    namespace_id       bigint      not null references sched.sched_namespace (id),
    key                varchar     not null,
    trigger_kind       int         not null,
    schedule           jsonb       not null,
    variables          jsonb,
    tags               varchar[]   not null default '{}',
    description        varchar,
    executed_count     bigint      not null default 0,
    refresh_occurrence timestamptz not null,
    status             int         not null,
    valid_time         timestamptz,
    invalid_time       timestamptz,
    cid                bigint      not null,
    ctime              timestamptz not null,
    mid                bigint,
    mtime              timestamptz,
    constraint trigger_definition_pk primary key (id)
);

create unique index trigger_definition_uidx_key on sched.trigger_definition (namespace_id, key);

comment on column sched.trigger_definition.refresh_occurrence is '需要重新计算发生时间';

comment on column sched.trigger_definition.valid_time is '触发器生效开始时间，为NULL代表不限制';

comment on column sched.trigger_definition.invalid_time is '触发器无效结束时间，为NULL代表不限制';

--
-- 流程定义与触发器关联表
create table if not exists sched.process_trigger_rel
(
    process_id bigint      not null references sched.process_definition (id),
    trigger_id bigint      not null references sched.trigger_definition (id),
    cid        bigint      not null,
    ctime      timestamptz not null,
    constraint process_trigger_rel_pk primary key (process_id, trigger_id)
);

--
-- 流程实例
create table if not exists sched.process_instance
(
    id             uuid        not null,
    process_id     bigint      not null references sched.process_definition (id),
    trigger_id     bigint references sched.trigger_definition (id),
    status         int         not null default 1,
    retry_count    int         not null default 0,
    execution_time timestamptz not null,
    complete_time  timestamptz,
    cid            bigint      not null,
    ctime          timestamptz not null,
    mid            bigint,
    mtime          timestamptz,
    constraint process_instance_pk primary key (id)
);

comment on column sched.process_instance.execution_time is '计算出的流程实例开始执行时间';

comment on column sched.process_instance.complete_time is '流程实例实际执行完成时间';

--
-- 流程任务
create table if not exists sched.process_task
(
    id                    uuid        not null,
    process_instance_id   uuid        not null references sched.process_instance (id),
    --     process_id            BIGINT      NOT NULL REFERENCES sched.process_definition (id),
    status                int         not null,
    variables_input       jsonb,
    variables_output      jsonb,
    retry_count           int         not null default 0,
    execute_begin_time    timestamptz not null,
    execute_complete_time timestamptz,
    constraint process_task_pk primary key (id)
);

comment on column sched.process_task.variables_input is '任务执行时的输入变量';

comment on column sched.process_task.variables_output is '任务执行完的输出变量';

comment on column sched.process_task.retry_count is '任务重试次数 + 1 = count(process_task_job)';

--
-- 流程任务作业执行记录
create table if not exists sched.process_task_job
(
    id                    uuid        not null,
    task_id               uuid        not null references sched.process_task (id),
    --     process_instance_id   UUID        NOT NULL REFERENCES sched.process_instance (id),
    --     process_id            BIGINT      NOT NULL REFERENCES sched.process_definition (id),
    status                int         not null,
    execute_begin_time    timestamptz not null,
    execute_complete_time timestamptz,
    constraint process_task_log_pk primary key (id)
);
