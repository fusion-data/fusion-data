# fusiondata


## Development environment with Docker

```bash
docker compose up -d --build && docker compose logs -f db
```

## Opentelemetry

配置以下环境变量：

```sh
export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://localhost:4317
export OTEL_TRACES_SAMPLER=always_on
export OTEL_SERVICE_NAME=fusion-iam
```
