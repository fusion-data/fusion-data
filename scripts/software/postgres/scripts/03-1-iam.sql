SET TIMEZONE TO 'Asia/Chongqing';
\c fusiondata;
\c - fusiondata;
CREATE SCHEMA IF NOT EXISTS iam;
--
-- tenant
CREATE TABLE IF NOT EXISTS iam.tenant
(
    id    SERIAL       NOT NULL,
    name  VARCHAR(256) NOT NULL,
    cid   BIGINT       NOT NULL,
    ctime TIMESTAMPTZ  NOT NULL,
    mid   BIGINT,
    mtime TIMESTAMPTZ,
    CONSTRAINT tenant_pk PRIMARY KEY (id)
);
--
-- User
CREATE TABLE IF NOT EXISTS iam.user
(
    id     BIGSERIAL   NOT NULL,
    email  VARCHAR
        CONSTRAINT user_uk_email UNIQUE,
    phone  VARCHAR
        CONSTRAINT user_uk_phone UNIQUE,
    name   VARCHAR,
    status INT         NOT NULL,
    gender INT         NOT NULL,
    cid    BIGINT      NOT NULL,
    ctime  TIMESTAMPTZ NOT NULL,
    mid    BIGINT,
    mtime  TIMESTAMPTZ,
    CONSTRAINT user_pk PRIMARY KEY (id)
);
CREATE TABLE IF NOT EXISTS iam.user_tenant_rel
(
    user_id   BIGINT      NOT NULL REFERENCES iam.user (id),
    tenant_id INT         NOT NULL REFERENCES iam.tenant (id),
    cid       BIGINT      NOT NULL,
    ctime     TIMESTAMPTZ NOT NULL,
    CONSTRAINT user_tenant_rel_pk PRIMARY KEY (user_id, tenant_id)
);
--
-- User Credential
CREATE TABLE IF NOT EXISTS iam.user_credential
(
    id            BIGINT       NOT NULL
        CONSTRAINT user_credential_fk_user REFERENCES iam.user (id),
    encrypted_pwd VARCHAR(255) NOT NULL,
    cid           BIGINT       NOT NULL,
    ctime         TIMESTAMPTZ  NOT NULL,
    mid           BIGINT,
    mtime         TIMESTAMPTZ,
    CONSTRAINT user_credential_pk PRIMARY KEY (id)
);
--
-- Role
CREATE TABLE IF NOT EXISTS iam.role
(
    id          BIGSERIAL   NOT NULL,
    name        VARCHAR(50) NOT NULL,
    description TEXT,
    status      INT         NOT NULL,
    cid         BIGINT      NOT NULL,
    ctime       TIMESTAMPTZ NOT NULL,
    mid         BIGINT,
    mtime       TIMESTAMPTZ,
    CONSTRAINT role_pk PRIMARY KEY (id)
);
--
-- Permission
CREATE TABLE IF NOT EXISTS iam.permission
(
    id          BIGSERIAL    NOT NULL,
    code        VARCHAR(255) NOT NULL
        CONSTRAINT permission_uk UNIQUE,
    description TEXT,
    resource    VARCHAR(255) NOT NULL,
    action      VARCHAR(255) NOT NULL,
    cid         BIGINT       NOT NULL,
    ctime       TIMESTAMPTZ  NOT NULL,
    mid         BIGINT,
    mtime       TIMESTAMPTZ,
    CONSTRAINT permission_pk PRIMARY KEY (id)
);
--
-- User Role Relation
CREATE TABLE IF NOT EXISTS iam.user_role
(
    user_id BIGINT      NOT NULL,
    role_id BIGINT      NOT NULL,
    cid     BIGINT      NOT NULL,
    ctime   TIMESTAMPTZ NOT NULL,
    CONSTRAINT user_role_pk PRIMARY KEY (user_id, role_id),
    CONSTRAINT user_role_fk_user FOREIGN KEY (user_id) REFERENCES iam.user (id),
    CONSTRAINT user_role_fk_role FOREIGN KEY (role_id) REFERENCES iam.role (id)
);
--
-- Role Permission Relation
CREATE TABLE IF NOT EXISTS iam.role_permission
(
    role_id       BIGINT      NOT NULL,
    permission_id BIGINT      NOT NULL,
    cid           BIGINT      NOT NULL,
    ctime         TIMESTAMPTZ NOT NULL,
    CONSTRAINT role_permission_pk PRIMARY KEY (role_id, permission_id),
    CONSTRAINT role_permission_fk_role FOREIGN KEY (role_id) REFERENCES iam.role (id),
    CONSTRAINT role_permission_fk_permission FOREIGN KEY (permission_id) REFERENCES iam.permission (id)
);


--------
-- ABAC
--------

-- 策略表
CREATE TABLE IF NOT EXISTS iam.policy
(
    id          UUID PRIMARY KEY,
    description VARCHAR(255),
    policy      JSONB       NOT NULL,
    status      INT         NOT NULL,
    cid         BIGINT      NOT NULL,
    ctime       TIMESTAMPTZ NOT NULL,
    mid         BIGINT,
    mtime       TIMESTAMPTZ
);
-- -- 属性表
-- CREATE TABLE IF NOT EXISTS iam.attribute
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
-- CREATE TABLE IF NOT EXISTS iam.attribute_value
-- (
--     id           BIGSERIAL PRIMARY KEY,
--     attribute_id BIGINT      NOT NULL REFERENCES iam.attribute (id),
--     entity_id    BIGINT      NOT NULL, -- 关联到具体实体（如用户ID）
--     value        TEXT        NOT NULL,
--     cid          BIGINT      NOT NULL,
--     ctime        TIMESTAMPTZ NOT NULL,
--     mid          BIGINT,
--     mtime        TIMESTAMPTZ
-- );
-- -- 索引
-- CREATE INDEX idx_attribute_entity_type ON iam.attribute (entity_type);
-- CREATE INDEX idx_attribute_value_attribute_id ON iam.attribute_value (attribute_id);
-- CREATE INDEX idx_attribute_value_entity_id ON iam.attribute_value (entity_id);
