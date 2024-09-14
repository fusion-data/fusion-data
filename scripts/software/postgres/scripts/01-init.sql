SET TIMEZONE TO 'Asia/Chongqing';
\c template1;
-- create extension adminpack;
----------------------------------------
-- #functions
----------------------------------------
-- 将数组反序
CREATE OR REPLACE FUNCTION array_reverse(anyarray) RETURNS anyarray AS
$$
SELECT ARRAY(
               SELECT $1[i]
               FROM GENERATE_SUBSCRIPTS($1, 1) AS s (i)
               ORDER BY i DESC
       );
$$ LANGUAGE 'sql' STRICT
                  IMMUTABLE;
----------------------------------------
-- #functions
----------------------------------------
----------------------------------------
-- init tables, views, sequences  begin
----------------------------------------
----------------------------------------
-- init tables, views, sequences  end
----------------------------------------
-- change tables, views, sequences owner to massdata
-- DO
-- $$
--     DECLARE
--         r record;
--     BEGIN
--         FOR r IN SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'
--             LOOP
--                 EXECUTE 'alter table ' || r.table_name || ' owner to massdata;';
--             END LOOP;
--     END
-- $$;
-- DO
-- $$
--     DECLARE
--         r record;
--     BEGIN
--         FOR r IN select sequence_name from information_schema.sequences where sequence_schema = 'public'
--             LOOP
--                 EXECUTE 'alter sequence ' || r.sequence_name || ' owner to massdata;';
--             END LOOP;
--     END
-- $$;
-- DO
-- $$
--     DECLARE
--         r record;
--     BEGIN
--         FOR r IN select table_name from information_schema.views where table_schema = 'public'
--             LOOP
--                 EXECUTE 'alter table ' || r.table_name || ' owner to massdata;';
--             END LOOP;
--     END
-- $$;
-- grant all privileges on all tables in schema public to massdata;
-- grant all privileges on all sequences in schema public to massdata;
-- 批量 grant/ revoke 用户权限
CREATE OR REPLACE FUNCTION g_or_v(
    g_or_v TEXT,
    -- 输入 grant or revoke 表示赋予或回收
    own NAME,
    -- 指定用户 owner
    target NAME,
    -- 赋予给哪个目标用户 grant privilege to who?
    objtyp TEXT,
    --  对象类别: 表, 物化视图, 视图 object type 'r', 'v' or 'm', means table,view,materialized view
    exp TEXT[],
    --  排除哪些对象, 用数组表示, excluded objects
    priv TEXT --  权限列表, privileges, ,splits, like 'select,insert,update'
) RETURNS VOID AS
$$
DECLARE
    nsp     NAME;
    rel     NAME;
    sql     TEXT;
    tmp_nsp NAME := '';
BEGIN
    FOR nsp,
        rel IN
        SELECT t2.nspname,
               t1.relname
        FROM pg_class t1,
             pg_namespace t2
        WHERE t1.relkind = objtyp
          AND t1.relnamespace = t2.oid
          AND t1.relowner = (SELECT oid
                             FROM pg_roles
                             WHERE rolname = own)
        LOOP
            IF (
                   tmp_nsp = ''
                       OR tmp_nsp <> nsp
                   )
                AND LOWER(g_or_v) = 'grant' THEN -- auto grant schema to target user
                sql := 'GRANT usage on schema "' || nsp || '" to ' || target;
                EXECUTE sql;
                RAISE NOTICE '%',
                    sql;
            END IF;
            tmp_nsp := nsp;
            IF (
                exp IS NOT NULL
                    AND nsp || '.' || rel = ANY (exp)
                ) THEN
                RAISE NOTICE '% excluded % .',
                    g_or_v,
                    nsp || '.' || rel;
            ELSE
                IF LOWER(g_or_v) = 'grant' THEN
                    sql := g_or_v || ' ' || priv || ' on "' || nsp || '"."' || rel || '" to ' || target;
                ELSIF LOWER(g_or_v) = 'revoke' THEN
                    sql := g_or_v || ' ' || priv || ' on "' || nsp || '"."' || rel || '" from ' || target;
                ELSE
                    RAISE NOTICE 'you must enter grant or revoke';
                END IF;
                RAISE NOTICE '%',
                    sql;
                EXECUTE sql;
            END IF;
        END LOOP;
END;
$$ LANGUAGE plpgsql;
