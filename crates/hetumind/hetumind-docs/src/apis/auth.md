# API for Hetumind

## Authorization

### Sign Up

```shell
curl -v -X POST http://127.0.0.1:50000/api/auth/signup \
  -H 'Content-Type: application/json' \
  -d '{"email": "admin@hetumind-studio.com", "password": "Hetumind.2025"}'
```

### Sign In

```shell
curl -v -X POST http://127.0.0.1:50000/api/auth/signin \
  -H 'Content-Type: application/json' \
  -d '{"account": "admin@hetumind-studio.com", "password": "Hetumind.2025"}'
```

### Get user by id

```shell
curl -v -X POST http://127.0.0.1:50000/api/v1/users/query \
  -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0..gcKHCVUjVnvngIQXe8m5wg.Nle9OQY1WRgbALRQKADBjwnItaSHnTL8gWqywA1aLM4.9BMLbnR-SDivEMI-qKqJuQ' \
  -H 'Content-Type: application/json' \
  -d '{"options":{},"filter":{}}'
```

### Get user pagination

```shell
curl -v -X POST http://127.0.0.1:50000/api/v1/user/page \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0..48bv9uAp95OEXAk5bmL39Q.-oPybhSbdPhGuLokGZ8OjhSXmBInj8Ldw2F7awacid4.68l2IHywRLvB8yw3Qhjn3Q" \
  -H "Content-Type: application/json" \
  -d '{
    "options": {},
    "filter": {}
  }'
```
