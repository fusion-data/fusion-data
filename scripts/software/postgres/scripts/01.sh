#!/bin/sh
# Execute sql commands

psql -d template1 -f /sqls/init.sql
psql -f /sqls/ddl.sql
psql -U fusiondata -d fusiondata -f /sqls/iam.sql
psql -U fusiondata -d fusiondata -f /sqls/iam-dml.sql
psql -U fusiondata -d fusiondata -f /sqls/scheduler.sql
psql -U fusiondata -d fusiondata -f /sqls/scheduler-dml.sql
