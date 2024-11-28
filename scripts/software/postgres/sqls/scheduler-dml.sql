set
    timezone to 'Asia/Chongqing';

-- create schema
create schema if not exists fusiondata;
set search_path to fusiondata;

--------------------------------
-- initial data
--------------------------------
insert into sched_namespace (tenant_id, namespace, status, cid, ctime)
values (1, 'default', 100, 1, now());
