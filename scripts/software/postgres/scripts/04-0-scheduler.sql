SET TIMEZONE TO 'Asia/Chongqing';
\c fusiondata;
\c - fusiondata;
-- create schema
CREATE SCHEMA IF NOT EXISTS sched;
--
-- sched_job
CREATE TABLE IF NOT EXISTS sched.sched_job
(
    id          UUID        NOT NULL,
    "type"      INT         NOT NULL,
    description VARCHAR,
    tags        VARCHAR[]   NOT NULL DEFAULT '{}',
    data        BYTEA       NOT NULL DEFAULT ''::BYTEA,
    cid         BIGINT      NOT NULL,
    ctime       TIMESTAMPTZ NOT NULL,
    mid         BIGINT,
    mtime       TIMESTAMPTZ,
    CONSTRAINT sched_job_pk PRIMARY KEY (id)
);
--
-- sched_trigger
CREATE TABLE IF NOT EXISTS sched.sched_trigger
(
    id          UUID        NOT NULL,
    description VARCHAR,
    "type"      INT         NOT NULL,
    schedule    JSONB       NOT NULL,
    tags        VARCHAR[]   NOT NULL DEFAULT '{}',
    data        BYTEA       NOT NULL DEFAULT ''::BYTEA,
    cid         BIGINT      NOT NULL,
    ctime       TIMESTAMPTZ NOT NULL,
    mid         BIGINT,
    mtime       TIMESTAMPTZ,
    CONSTRAINT sched_trigger_pk PRIMARY KEY (id)
);
--
-- sched_job_trigger
CREATE TABLE IF NOT EXISTS sched.sched_job_trigger
(
    job_id     UUID        NOT NULL,
    trigger_id UUID        NOT NULL,
    cid        BIGINT      NOT NULL,
    ctime      TIMESTAMPTZ NOT NULL,
    CONSTRAINT sched_job_trigger_pk PRIMARY KEY (job_id, trigger_id)
);