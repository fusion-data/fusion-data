# SQLs

## Init ddl & dml

### jieyuan

```
psql -h localhost -U fusiondata -d fusiondata -f scripts/software/postgres/sqls/jieyuan-ddl.sql
```

## 清除脚本

### jieyuan

```sql
drop table if exists iam_user_role cascade;
drop table if exists iam_role cascade;
drop table if exists iam_permission cascade;
drop table if exists iam_policy cascade;
drop table if exists iam_user_credential cascade;
drop table if exists iam_user cascade;
```

### hetuflow

```sql
drop table if exists sched_server cascade;
```
