set
    timezone to 'Asia/Chongqing';

-- create schema
create schema if not exists fusiondata;
set search_path to fusiondata;

--
-- tenant
create table if not exists tenant
(
    id    serial       not null,
    name  varchar(256) not null,
    cid   bigint       not null,
    ctime timestamptz  not null,
    mid   bigint,
    mtime timestamptz,
    constraint tenant_pk primary key (id)
);

--
-- User
create table if not exists "user"
(
    id     bigserial   not null,
    email  varchar
        constraint user_uk_email unique,
    phone  varchar
        constraint user_uk_phone unique,
    name   varchar,
    status int         not null,
    gender int         not null,
    cid    bigint      not null,
    ctime  timestamptz not null,
    mid    bigint,
    mtime  timestamptz,
    constraint user_pk primary key (id)
);

create table if not exists user_tenant_rel
(
    user_id   bigint      not null references "user" (id),
    tenant_id int         not null references tenant (id),
    cid       bigint      not null,
    ctime     timestamptz not null,
    constraint user_tenant_rel_pk primary key (user_id, tenant_id)
);

--
-- User Credential
create table if not exists user_credential
(
    id            bigint       not null
        constraint user_credential_fk_user references "user" (id),
    encrypted_pwd varchar(255) not null,
    cid           bigint       not null,
    ctime         timestamptz  not null,
    mid           bigint,
    mtime         timestamptz,
    constraint user_credential_pk primary key (id)
);

--
-- Role
create table if not exists role
(
    id          bigserial   not null,
    name        varchar(50) not null,
    description text,
    status      int         not null,
    cid         bigint      not null,
    ctime       timestamptz not null,
    mid         bigint,
    mtime       timestamptz,
    constraint role_pk primary key (id)
);

--
-- Permission
create table if not exists permission
(
    id          bigserial    not null,
    code        varchar(255) not null
        constraint permission_uk unique,
    description text,
    resource    varchar(255) not null,
    action      varchar(255) not null,
    cid         bigint       not null,
    ctime       timestamptz  not null,
    mid         bigint,
    mtime       timestamptz,
    constraint permission_pk primary key (id)
);

--
-- User Role Relation
create table if not exists user_role
(
    user_id bigint      not null,
    role_id bigint      not null,
    cid     bigint      not null,
    ctime   timestamptz not null,
    constraint user_role_pk primary key (user_id, role_id),
    constraint user_role_fk_user foreign key (user_id) references "user" (id),
    constraint user_role_fk_role foreign key (role_id) references role (id)
);

--
-- Role Permission Relation
create table if not exists role_permission
(
    role_id       bigint      not null,
    permission_id bigint      not null,
    cid           bigint      not null,
    ctime         timestamptz not null,
    constraint role_permission_pk primary key (role_id, permission_id),
    constraint role_permission_fk_role foreign key (role_id) references role (id),
    constraint role_permission_fk_permission foreign key (permission_id) references permission (id)
);

--------
-- ABAC
--------
-- 策略表
create table if not exists policy
(
    id          uuid primary key,
    description varchar(255),
    policy      jsonb       not null,
    status      int         not null,
    cid         bigint      not null,
    ctime       timestamptz not null,
    mid         bigint,
    mtime       timestamptz
);

-- -- 属性表
-- CREATE TABLE IF NOT EXISTS attribute
-- (
--     id          BIGSERIAL PRIMARY KEY,
--     name        VARCHAR(255) NOT NULL,
--     description TEXT,
--     entity_type VARCHAR(50)  NOT NULL, -- 如：user, resource, environment
--     data_type   VARCHAR(50)  NOT NULL, -- 如：string, integer, boolean, date
--     cid         BIGINT       NOT NULL,
--     ctime       TIMESTAMPTZ  NOT NULL,
--     mid         BIGINT,
--     mtime       TIMESTAMPTZ
-- );-- 属性值表
-- CREATE TABLE IF NOT EXISTS attribute_value
-- (
--     id           BIGSERIAL PRIMARY KEY,
--     attribute_id BIGINT      NOT NULL REFERENCES attribute (id),
--     entity_id    BIGINT      NOT NULL, -- 关联到具体实体（如用户ID）
--     value        TEXT        NOT NULL,
--     cid          BIGINT      NOT NULL,
--     ctime        TIMESTAMPTZ NOT NULL,
--     mid          BIGINT,
--     mtime        TIMESTAMPTZ
-- );
-- -- 索引
-- CREATE INDEX idx_attribute_entity_type ON attribute (entity_type);
-- CREATE INDEX idx_attribute_value_attribute_id ON attribute_value (attribute_id);
-- CREATE INDEX idx_attribute_value_entity_id ON attribute_value (entity_id);
