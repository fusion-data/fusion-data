#!/bin/sh
# Execute sql commands

psql -d template1 -f /sqls/init.sql
psql -f /sqls/ddl.sql
psql -U fusiondata -d fusiondata -f /sqls/hetuflow-ddl.sql
psql -U fusiondata -d fusiondata -f /sqls/hetuflow-dml.sql
psql -U fusiondata -d fusiondata -f /sqls/hetumind-ddl.sql
psql -U fusiondata -d fusiondata -f /sqls/hetumind-dml.sql
