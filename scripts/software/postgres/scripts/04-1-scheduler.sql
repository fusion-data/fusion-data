SET TIMEZONE TO 'Asia/Chongqing';
\c fusiondata;
\c - fusiondata;
-- create schema
CREATE SCHEMA IF NOT EXISTS sched;
--
-- sched_node
CREATE TABLE IF NOT EXISTS sched.sched_node
(
    -- 节点ID
    id              VARCHAR(36) NOT NULL,
    kind            INT         NOT NULL,
    -- 节点通信地址
    addr            VARCHAR(64) NOT NULL,
    last_check_time TIMESTAMPTZ,
    status          INT         NOT NULL,
    cid             BIGINT      NOT NULL,
    ctime           TIMESTAMPTZ NOT NULL,
    mid             BIGINT,
    mtime           TIMESTAMPTZ,
    CONSTRAINT sched_node_pk PRIMARY KEY (id)
);
COMMENT ON COLUMN sched.sched_node.last_check_time IS '节点最后检查时间';
--
-- sched_locks
CREATE TABLE IF NOT EXISTS sched.sched_lock
(
    node_id   VARCHAR(36) NOT NULL REFERENCES sched.sched_node (id),
    lock_kind INT         NOT NULL,
    CONSTRAINT sched_locks_pk PRIMARY KEY (node_id, lock_kind)
);
--
-- sched_group 调度分组
CREATE TABLE IF NOT EXISTS sched.sched_namespace
(
    id        SERIAL       NOT NULL,
    tenant_id INT          NOT NULL REFERENCES iam.tenant (id),
    namespace VARCHAR(512) NOT NULL UNIQUE,
    node_id   VARCHAR(36) REFERENCES sched.sched_node (id),
    status    INT          NOT NULL,
    cid       BIGINT       NOT NULL,
    ctime     TIMESTAMPTZ  NOT NULL,
    mid       BIGINT,
    mtime     TIMESTAMPTZ,
    CONSTRAINT sched_group_pk PRIMARY KEY (id)
);
CREATE UNIQUE INDEX sched_group_uidx_name ON sched.sched_namespace (tenant_id, namespace);
COMMENT ON COLUMN sched.sched_namespace.namespace IS '调度分组名，需要租户下唯一。可以按应用或服务进行划分';
COMMENT ON COLUMN sched.sched_namespace.node_id IS '节点ID';
--
-- 流程定义
CREATE TABLE IF NOT EXISTS sched.process_definition
(
    id           BIGSERIAL   NOT NULL,
    tenant_id    INT         NOT NULL REFERENCES iam.tenant (id),
    namespace_id INT         NOT NULL REFERENCES sched.sched_namespace (id),
    key          VARCHAR     NOT NULL,
    description  VARCHAR,
    tags         VARCHAR[]   NOT NULL DEFAULT '{}',
    variables    JSONB,
    data         BYTEA,
    cid          BIGINT      NOT NULL,
    ctime        TIMESTAMPTZ NOT NULL,
    mid          BIGINT,
    mtime        TIMESTAMPTZ,
    CONSTRAINT process_definition_pk PRIMARY KEY (id)
);
CREATE UNIQUE INDEX process_definition_uidx_key ON sched.process_definition (namespace_id, key);
--
-- 触发器定义
CREATE TABLE IF NOT EXISTS sched.trigger_definition
(
    id               BIGSERIAL   NOT NULL,
    tenant_id        INT         NOT NULL REFERENCES iam.tenant (id),
    namespace_id     INT         NOT NULL REFERENCES sched.sched_namespace (id),
    key              VARCHAR     NOT NULL,
    description      VARCHAR,
    "type"           INT         NOT NULL,
    tags             VARCHAR[]   NOT NULL DEFAULT '{}',
    variables        JSONB,
    status           INT         NOT NULL,
    valid_begin_time TIMESTAMPTZ,
    valid_end_time   TIMESTAMPTZ,
    cid              BIGINT      NOT NULL,
    ctime            TIMESTAMPTZ NOT NULL,
    mid              BIGINT,
    mtime            TIMESTAMPTZ,
    CONSTRAINT trigger_definition_pk PRIMARY KEY (id)
);
CREATE UNIQUE INDEX trigger_definition_uidx_key ON sched.trigger_definition (namespace_id, key);
COMMENT ON COLUMN sched.trigger_definition.valid_begin_time IS '触发器有效开始时间，为NULL代表不限制';
COMMENT ON COLUMN sched.trigger_definition.valid_end_time IS '触发器有效结束时间，为NULL代表不限制';
-- 简单调度
CREATE TABLE IF NOT EXISTS sched.trigger_simple_schedule
(
    id              UUID        NOT NULL,
    trigger_id      BIGINT      NOT NULL REFERENCES sched.trigger_definition (id),
    interval_type   INT         NOT NULL,
    "interval"      INTERVAL    NOT NULL,
    first_delay     INTERVAL    NOT NULL,
    execution_count INT,
    cid             BIGINT      NOT NULL,
    ctime           TIMESTAMPTZ NOT NULL,
    mid             BIGINT,
    mtime           TIMESTAMPTZ,
    CONSTRAINT trigger_simple_schedule_pk PRIMARY KEY (id)
);
COMMENT ON COLUMN sched.trigger_simple_schedule.interval_type IS '间隔类型，1: 固定速率，2: 固定延迟';
-- CRON 调度
CREATE TABLE IF NOT EXISTS sched.trigger_cron_schedule
(
    id         UUID        NOT NULL,
    trigger_id BIGINT      NOT NULL REFERENCES sched.trigger_definition (id),
    cron       VARCHAR     NOT NULL,
    cid        BIGINT      NOT NULL,
    ctime      TIMESTAMPTZ NOT NULL,
    mid        BIGINT,
    mtime      TIMESTAMPTZ,
    CONSTRAINT trigger_cron_schedule_pk PRIMARY KEY (id)
);
--
-- 流程定义与触发器关联表
CREATE TABLE IF NOT EXISTS sched.process_trigger_rel
(
    process_id BIGINT      NOT NULL REFERENCES sched.process_definition (id),
    trigger_id BIGINT      NOT NULL REFERENCES sched.trigger_definition (id),
    cid        BIGINT      NOT NULL,
    ctime      TIMESTAMPTZ NOT NULL,
    CONSTRAINT process_trigger_rel_pk PRIMARY KEY (process_id, trigger_id)
);
--
-- 流程实例
CREATE TABLE IF NOT EXISTS sched.process_instance
(
    id                    UUID        NOT NULL,
    process_id            BIGINT      NOT NULL REFERENCES sched.process_definition (id),
    trigger_id            BIGINT REFERENCES sched.trigger_definition (id),
    status                INT         NOT NULL,
    retry_count           INT         NOT NULL DEFAULT 0,
    execute_begin_time    TIMESTAMPTZ NOT NULL,
    execute_complete_time TIMESTAMPTZ,
    cid                   BIGINT      NOT NULL,
    ctime                 TIMESTAMPTZ NOT NULL,
    mid                   BIGINT,
    mtime                 TIMESTAMPTZ,
    CONSTRAINT process_instance_pk PRIMARY KEY (id)
);
COMMENT ON COLUMN sched.process_instance.execute_begin_time IS '计算出的流程实例开始执行时间';
COMMENT ON COLUMN sched.process_instance.execute_complete_time IS '流程实例实际执行完成时间';
--
-- 流程任务
CREATE TABLE IF NOT EXISTS sched.process_task
(
    id                    UUID        NOT NULL,
    process_instance_id   UUID        NOT NULL REFERENCES sched.process_instance (id),
--     process_id            BIGINT      NOT NULL REFERENCES sched.process_definition (id),
    status                INT         NOT NULL,
    variables_input       JSONB,
    variables_output      JSONB,
    retry_count           INT         NOT NULL DEFAULT 0,
    execute_begin_time    TIMESTAMPTZ NOT NULL,
    execute_complete_time TIMESTAMPTZ,
    CONSTRAINT process_task_pk PRIMARY KEY (id)
);
COMMENT ON COLUMN sched.process_task.variables_input IS '任务执行时的输入变量';
COMMENT ON COLUMN sched.process_task.variables_output IS '任务执行完的输出变量';
COMMENT ON COLUMN sched.process_task.retry_count IS '任务重试次数 + 1 = count(process_task_job)';
--
-- 流程任务作业执行记录
CREATE TABLE IF NOT EXISTS sched.process_task_job
(
    id                    UUID        NOT NULL,
    task_id               UUID        NOT NULL REFERENCES sched.process_task (id),
--     process_instance_id   UUID        NOT NULL REFERENCES sched.process_instance (id),
--     process_id            BIGINT      NOT NULL REFERENCES sched.process_definition (id),
    status                INT         NOT NULL,
    execute_begin_time    TIMESTAMPTZ NOT NULL,
    execute_complete_time TIMESTAMPTZ,
    CONSTRAINT process_task_log_pk PRIMARY KEY (id)
);
