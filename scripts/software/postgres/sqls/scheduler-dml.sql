set
  timezone to 'Asia/Chongqing';

--------------------------------
-- initial data
--------------------------------
insert into
  sched.sched_namespace (tenant_id, namespace, status, cid, ctime)
values
  (1, 'default', 100, 1, now());
