-- ============================================================================
-- 数据库迁移脚本：service_path_mapping -> iam_resource_mapping
-- 迁移版本：v1.0.0 -> v1.1.0
-- 创建时间：2025-01-19
-- 描述：重命名资源映射表，添加租户隔离支持，更新缓存表
-- ============================================================================

-- 步骤 1: 创建新的 iam_resource_mapping 表
CREATE TABLE iam_resource_mapping (
    id BIGSERIAL PRIMARY KEY,
    mapping_code VARCHAR(100) UNIQUE,
    service VARCHAR(50) NOT NULL,
    path_pattern VARCHAR(1024) NOT NULL,
    method VARCHAR(10) NOT NULL,
    action VARCHAR(100) NOT NULL,
    resource_tpl VARCHAR(500) NOT NULL,
    mapping_params JSONB,
    enabled BOOLEAN DEFAULT true,
    tenant_id BIGINT,  -- 新增租户隔离支持
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_by BIGINT NOT NULL,
    updated_by BIGINT,
    description TEXT,
    UNIQUE(service, path_pattern, method)
);

-- 步骤 2: 创建新的 resource_mapping_cache 表
CREATE TABLE resource_mapping_cache (
    cache_key VARCHAR(255) PRIMARY KEY,
    service VARCHAR(50) NOT NULL,
    mapping_response JSONB NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 步骤 3: 创建新表的索引
-- IAM Resource Mapping 索引
CREATE INDEX idx_iam_resource_mapping_lookup ON iam_resource_mapping(service, method, enabled, tenant_id);
CREATE INDEX idx_iam_resource_mapping_pattern ON iam_resource_mapping(service, path_pattern, tenant_id);
CREATE INDEX idx_iam_resource_mapping_updated_at ON iam_resource_mapping(updated_at);
CREATE INDEX idx_iam_resource_mapping_code ON iam_resource_mapping(mapping_code);
CREATE INDEX idx_iam_resource_mapping_service_path ON iam_resource_mapping(service, path_pattern, method, tenant_id);
CREATE INDEX idx_iam_resource_mapping_tenant_id ON iam_resource_mapping(tenant_id);

-- Resource Mapping Cache 索引
CREATE INDEX idx_resource_mapping_cache_expires_at ON resource_mapping_cache(expires_at);
CREATE INDEX idx_resource_mapping_cache_service ON resource_mapping_cache(service);

-- 步骤 4: 数据迁移（从 service_path_mapping 到 iam_resource_mapping）
INSERT INTO iam_resource_mapping (
    id,
    mapping_code,
    service,
    path_pattern,
    method,
    action,
    resource_tpl,
    mapping_params,
    enabled,
    tenant_id,
    created_at,
    updated_at,
    created_by,
    updated_by,
    description
)
SELECT
    id,
    path_code AS mapping_code,
    service,
    path_pattern,
    method,
    action,
    resource_tpl,
    path_params AS mapping_params,
    enabled,
    NULL AS tenant_id,  -- 新增字段，初始为 NULL，后续通过管理界面配置
    created_at,
    updated_at,
    created_by,
    updated_by,
    description
FROM service_path_mapping;

-- 步骤 5: 迁移缓存数据（如果 path_lookup_cache 表存在）
-- 注意：这里假设原有缓存表结构，需要根据实际情况调整
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'path_lookup_cache') THEN
        INSERT INTO resource_mapping_cache (
            cache_key,
            service,
            mapping_response,
            expires_at,
            created_at
        )
        SELECT
            cache_key,
            service,
            lookup_response AS mapping_response,
            expires_at,
            created_at
        FROM path_lookup_cache;
    END IF;
END $$;

-- 步骤 6: 数据迁移验证
SELECT
    'service_path_mapping' as table_name,
    COUNT(*) as record_count
FROM service_path_mapping
UNION ALL
SELECT
    'iam_resource_mapping' as table_name,
    COUNT(*) as record_count
FROM iam_resource_mapping
UNION ALL
SELECT
    'path_lookup_cache' as table_name,
    COUNT(*) as record_count
FROM path_lookup_cache
WHERE EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'path_lookup_cache')
UNION ALL
SELECT
    'resource_mapping_cache' as table_name,
    COUNT(*) as record_count
FROM resource_mapping_cache;

-- 步骤 7: 创建外键约束（如果需要与租户表关联）
-- ALTER TABLE iam_resource_mapping
-- ADD CONSTRAINT iam_resource_mapping_fk_tenant_id
-- FOREIGN KEY (tenant_id) REFERENCES iam_tenant(id);

-- 步骤 8: 重命名原表为备份（安全措施）
ALTER TABLE service_path_mapping RENAME TO service_path_mapping_backup_v1_0;

-- 如果存在 path_lookup_cache 表，也进行备份
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'path_lookup_cache') THEN
        ALTER TABLE path_lookup_cache RENAME TO path_lookup_cache_backup_v1_0;
    END IF;
END $$;

-- 步骤 9: 创建视图以保持向后兼容（可选，过渡期使用）
CREATE OR REPLACE VIEW service_path_mapping AS
SELECT
    id,
    mapping_code AS path_code,
    service,
    path_pattern,
    method,
    action,
    resource_tpl,
    mapping_params AS path_params,
    enabled,
    tenant_id,
    created_at,
    updated_at,
    created_by,
    updated_by,
    description
FROM iam_resource_mapping;

-- 如果原缓存表存在，创建兼容视图
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'path_lookup_cache_backup_v1_0') THEN
        CREATE OR REPLACE VIEW path_lookup_cache AS
        SELECT
            cache_key,
            service,
            mapping_response AS lookup_response,
            expires_at,
            created_at
        FROM resource_mapping_cache;
    END IF;
END $$;

-- ============================================================================
-- 迁移完成确认
-- ============================================================================

SELECT 'Migration to iam_resource_mapping completed successfully!' as status;

-- ============================================================================
-- 回滚脚本（如果需要回滚）
-- ============================================================================

/*
-- 回滚步骤 1: 删除视图
DROP VIEW IF EXISTS service_path_mapping;
DROP VIEW IF EXISTS path_lookup_cache;

-- 回滚步骤 2: 恢复原表名
ALTER TABLE service_path_mapping_backup_v1_0 RENAME TO service_path_mapping;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'path_lookup_cache_backup_v1_0') THEN
        ALTER TABLE path_lookup_cache_backup_v1_0 RENAME TO path_lookup_cache;
    END IF;
END $$;

-- 回滚步骤 3: 删除新表
DROP TABLE IF EXISTS iam_resource_mapping;
DROP TABLE IF EXISTS resource_mapping_cache;
*/

-- ============================================================================
-- 清理脚本（迁移完成并验证后执行）
-- ============================================================================

/*
-- 清理步骤 1: 删除向后兼容视图
DROP VIEW IF EXISTS service_path_mapping;
DROP VIEW IF EXISTS path_lookup_cache;

-- 清理步骤 2: 删除备份表
DROP TABLE IF EXISTS service_path_mapping_backup_v1_0;
DROP TABLE IF EXISTS path_lookup_cache_backup_v1_0;
*/

-- ============================================================================
-- 验证脚本
-- ============================================================================

-- 验证数据完整性
SELECT
    'Data Integrity Check' as check_type,
    CASE
        WHEN (SELECT COUNT(*) FROM iam_resource_mapping) = (SELECT COUNT(*) FROM service_path_mapping_backup_v1_0)
        THEN 'PASS - Record counts match'
        ELSE 'FAIL - Record count mismatch'
    END as result;

-- 验证关键字段映射
SELECT
    'Field Mapping Check' as check_type,
    CASE
        WHEN EXISTS (
            SELECT 1 FROM iam_resource_mapping irm
            JOIN service_path_mapping_backup_v1_0 spm ON irm.id = spm.id
            WHERE irm.mapping_code != spm.path_code
               OR irm.service != spm.service
               OR irm.action != spm.action
        )
        THEN 'FAIL - Field mapping mismatch'
        ELSE 'PASS - Field mapping correct'
    END as result;

-- 验证索引创建
SELECT
    indexname as index_name
FROM pg_indexes
WHERE tablename = 'iam_resource_mapping'
ORDER BY indexname;
