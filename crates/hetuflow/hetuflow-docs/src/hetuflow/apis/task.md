# Job and Task

## Job

### Create Job

Request:

```shell
curl -X 'POST' \
  'http://localhost:9500/api/v1/jobs/item' \
  -H 'accept: application/json' \
  -H 'Content-Type: application/json' \
  -d '{
  "name": "test002",
  "config": {
    "timeout": 3600,
    "max_retries": 3,
    "retry_interval": 600,
    "cmd": "uv",
    "args": ["run", "python", "-c", "print('您好，河图！')"],
    "capture_output": true,
    "max_output_size": 40960,
    "labels": {
      "env": "dev"
    }
  },
  "status": 1
}'
```

Response:

```json
{
  "id": "019977a0-757e-7310-bc5c-e15bf3c6d49a"
}
```

## Task

### Create Task

Request:

```shell
curl -X 'POST' \
  'http://localhost:9500/api/v1/tasks/create' \
  -H 'accept: application/json' \
  -H 'Content-Type: application/json' \
  -d '{
  "schedule_kind": 4,
  "job_id": "01997a4c-0e0e-7112-9fcb-97f8059d155f",
  "status": 1
}'
```

Response:

```json
{
  "id": "01997a55-8397-7821-8c17-d9dde38e6ec5"
}
```
