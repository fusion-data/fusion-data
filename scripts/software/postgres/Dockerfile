FROM postgres:16

ENV LANG=zh_CN.utf8
ENV TZ=Asia/Chongqing
ENV PGPORT=45432

RUN localedef -i zh_CN -c -f UTF-8 -A /usr/share/locale/locale.alias zh_CN.UTF-8

EXPOSE $PGPORT

COPY scripts/*.sql /docker-entrypoint-initdb.d/
COPY --chmod=0755 scripts/*.sh /docker-entrypoint-initdb.d/
COPY --chmod=0600 .pgpass /root/.pgpass
