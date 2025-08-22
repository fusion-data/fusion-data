# FROM postgres:17
# FROM tensorchord/vchord-postgres:pg17-v0.5.0
FROM pgvector/pgvector:pg17

ENV LANG=zh_CN.utf8
ENV TZ=Asia/Chongqing
ENV PGPORT=45432

RUN localedef -i zh_CN -c -f UTF-8 -A /usr/share/locale/locale.alias zh_CN.UTF-8

EXPOSE $PGPORT

COPY sqls /sqls
COPY --chmod=0755 scripts/*.sh /docker-entrypoint-initdb.d/
COPY --chown=postgres --chmod=0600 .pgpass /var/lib/postgresql/.pgpass
