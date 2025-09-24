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
  "name": "test001",
  "config": {
    "timeout": 3600,
    "max_retries": 3,
    "retry_interval": 600,
    "cmd": "bash",
    "args": [],
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
  'http://localhost:9500/api/v1/tasks/item' \
  -H 'accept: application/json' \
  -H 'Content-Type: application/json' \
  -d '{
  "job_id": "019977a0-757e-7310-bc5c-e15bf3c6d49a",
  "config": {
    "timeout": 3600,
    "max_retries": 3,
    "retry_interval": 600,
    "cmd": "bash",
    "args": [],
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
```
