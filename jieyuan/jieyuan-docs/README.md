# 界垣

## Access Token

- JSON WEB Token（JWT 令牌）:
- JSON WEB Encryption 令牌）: 对 JWT 的 payload 进行加密，应首选。但要考虑到客户端对加解密库（及 publich key）的支持情况。若客户端不需要关心 payload 内容，也许使用 不透明令牌即可。
- Opaque Access Token（不透明令牌）: 适合 Session 类型访问令牌。
  - 在撰写本文时，获取安全不透明令牌的最佳实践是使用加密安全的随机字符串生成器生成至少 128 位熵的令牌。

## API Calls

### Authentication

#### Sign Up

```shell
curl -X 'POST' \
  'http://localhost:9500/api/auth/signup' \
  -H 'accept: */*' \
  -H 'Content-Type: application/json' \
  -d '{
  "email": "developer@hetumind.com",
  "password": "2025.Developer"
}'
```

#### Relate Tenant

```shell
curl -X 'POST' \
  'http://localhost:9500/api/auth/relate-tenant' \
  -H 'accept: */*' \
  -H 'Content-Type: application/json' \
  -d '{
  "tenant_id": "tenant-1"
}'
```

#### Sign In

```shell
curl -X 'POST' \
  'http://localhost:9500/api/auth/signin' \
  -H 'accept: */*' \
  -H 'Content-Type: application/json' \
  -d '{
  "email": "root@jieyuan.com",
  "password": "2025.Developer",
  "tenant_id": 1
}'
```
