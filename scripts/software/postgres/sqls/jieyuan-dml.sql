insert into
  iam_tenant (id, name, description, status, created_by, created_at)
values
  (1, 'platform', 'Platform Super Tenant', 100, 1, now()),
  (2, 'test', 'Test Tenant', 100, 1, now());

alter sequence iam_tenant_id_seq restart
with
  3;

insert into
  iam_user (id, email, name, status, gender, created_by, created_at)
values
  (1, 'root@jieyuan.com', 'Super Admin', 100, 0, 1, now());

alter sequence iam_user_id_seq restart
with
  2;

insert into
  iam_user_credential (id, encrypted_pwd, created_by, created_at)
values
  (1, '#1#$argon2id$v=19$m=19456,t=2,p=1$M0/qtArYbdQHGoBmr2AOxQ$tY6C94NFcncPLOPyfDklRB72YzIHAX7zQb10KV74Bns', 1, now());

insert into
  iam_tenant_user (tenant_id, user_id, status, created_by, created_at)
values
  (1, 1, 100, 1, now());
