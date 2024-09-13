#!/bin/sh

pg_setup_postgresql_conf() {
	{
		printf '\n'
		printf "log_statement = 'all'"
		printf '\n'
	} >> "$PGDATA/postgresql.conf"
}

pg_setup_postgresql_conf
