# Hetuflow JWE Token 认证机制部署指南

## 概述

Hetuflow 支持基于 JWE (JSON Web Encryption) Token 的认证机制，用于保护 Agent 与 Server 之间的通信安全。本文档详细说明了如何配置和部署 JWE 认证功能。

## 认证流程

```mermaid
sequenceDiagram
    participant Admin as 管理员
    participant Server as Hetuflow Server
    participant Agent as Hetuflow Agent

    Admin->>Server: 1. 配置 JWE 密钥对
    Admin->>Server: 2. 启动 Server
    Admin->>Admin: 3. 生成 Agent JWE Token
    Admin->>Agent: 4. 配置 Agent JWE Token
    Agent->>Server: 5. WebSocket 连接 + JWE Token
    Server->>Server: 6. 验证 JWE Token
    Server->>Agent: 7. 注册成功/失败响应
```

## 密钥生成

### 使用脚本生成密钥

项目提供了便捷的密钥生成脚本：

```bash
# 进入 Hetuflow 项目目录
cd crates/hetuflow

# 生成密钥对
./scripts/generate-jwe-keys.sh

# 或指定输出目录
./scripts/generate-jwe-keys.sh /path/to/keys

# 查看配置格式
./scripts/generate-jwe-keys.sh --show

# 生成配置文件示例
./scripts/generate-jwe-keys.sh --config

# 执行所有操作
./scripts/generate-jwe-keys.sh --all
```

### 手动生成密钥

如果需要手动生成密钥，可以使用 OpenSSL：

```bash
# 生成 ECDH-ES P-256 私钥
openssl ecparam -genkey -name prime256v1 -noout -out jwe-private.pem

# 从私钥提取公钥
openssl ec -in jwe-private.pem -pubout -out jwe-public.pem
```

## Server 端配置

### 配置文件设置

在 `crates/hetuflow-bins/resources/hetuflow.toml` 中添加 JWE 配置：

```toml
[hetuflow.jwe]
# 私钥（用于解密 JWE Token）
private_key = "-----BEGIN PRIVATE KEY-----\nMIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQg...\n-----END PRIVATE KEY-----"
# 公钥（用于加密 JWE Token）
public_key = "-----BEGIN PUBLIC KEY-----\nMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE...\n-----END PUBLIC KEY-----"
# 密钥协商算法
key_agreement_algorithm = "ECDH-ES"
# 内容加密算法
content_encryption_algorithm = "A256GCM"
# Token 有效期（秒）
token_ttl = 3600
```

### 环境变量配置（推荐用于生产环境）

```bash
# 设置 JWE 私钥
export HETUFLOW__JWE__PRIVATE_KEY="-----BEGIN PRIVATE KEY-----\nMIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQg...\n-----END PRIVATE KEY-----"

# 设置 JWE 公钥
export HETUFLOW__JWE__PUBLIC_KEY="-----BEGIN PUBLIC KEY-----\nMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE...\n-----END PUBLIC KEY-----"

# 设置 Token 有效期
export HETUFLOW__JWE__TOKEN_TTL=3600
```

## Agent 端配置

### 获取 JWE Token

首先需要从 Server 端获取 JWE Token。可以通过以下方式：

1. **管理 API**（推荐）：通过 Server 提供的管理接口生成 Token
   ```shell
   curl -X 'POST' \
     'http://localhost:9500/api/v1/auth/generate-token' \
     -H 'accept: application/json' \
     -H 'Content-Type: application/json' \
     -d '{
       "agent_id": "agent001",
       "permissions": []
     }'
   ```
2. **命令行工具**：使用 Server 端的 CLI 工具生成 Token
3. **手动生成**：使用 JWE 库手动生成 Token

### 配置文件设置

在 `crates/hetuflow-agent/resources/hetuflow-agent.toml` 中配置 JWE Token：

```toml
[hetuflow.agent]
# 其他配置...
name = "my-agent"
labels = ["linux", "x86_64"]

# JWE Token（由 Server 端生成）
jwe_token = "eyJhbGciOiJFQ0RILUVTK0EyNTZLVyIsImVuYyI6IkEyNTZHQ00iLCJ0eXAiOiJKV0UifQ..."
```

### 环境变量配置

```bash
# 设置 JWE Token
export HETUFLOW__AGENT__JWE_TOKEN="eyJhbGciOiJFQ0RILUVTK0EyNTZLVyIsImVuYyI6IkEyNTZHQ00iLCJ0eXAiOiJKV0UifQ..."
```

## 部署步骤

### 1. 准备密钥

```bash
# 生成密钥对
./scripts/generate-jwe-keys.sh /etc/hetuflow/keys

# 设置适当的文件权限
chmod 600 /etc/hetuflow/keys/jwe-private.pem
chmod 644 /etc/hetuflow/keys/jwe-public.pem
```

### 2. 配置 Server

```bash
# 方式1：使用配置文件
cp /etc/hetuflow/keys/server-jwe-config.toml /path/to/hetuflow-server/resources/app.toml

# 方式2：使用环境变量
export HETUFLOW__JWE__PRIVATE_KEY="$(cat /etc/hetuflow/keys/jwe-private.pem | tr '\n' '\\n')"
export HETUFLOW__JWE__PUBLIC_KEY="$(cat /etc/hetuflow/keys/jwe-public.pem | tr '\n' '\\n')"
```

### 3. 启动 Server

```bash
# 启动 Hetuflow Server
./hetuflow-server
```

### 4. 生成 Agent Token

```bash
# 使用 Server API 生成 Token（示例）
curl -X POST http://localhost:9500/api/v1/admin/agents/token \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "550e8400-e29b-41d4-a716-446655440000",
    "permissions": ["task:execute", "heartbeat:send"]
  }'
```

### 5. 配置 Agent

```bash
# 配置 Agent JWE Token
export HETUFLOW__AGENT__JWE_TOKEN="<从步骤4获取的Token>"

# 启动 Agent
./hetuflow-agent
```

## 安全最佳实践

### 密钥管理

1. **私钥保护**：

   - 私钥文件权限设置为 600
   - 不要将私钥提交到版本控制系统
   - 使用密钥管理系统（如 HashiCorp Vault）存储私钥

2. **密钥轮换**：
   - 定期轮换密钥对
   - 支持多个公钥同时有效（向后兼容）
   - 逐步淘汰旧密钥

### Token 管理

1. **Token 有效期**：

   - 设置合理的 Token 有效期（建议 1-24 小时）
   - 实现 Token 自动刷新机制

2. **权限控制**：

   - 为不同 Agent 分配不同权限
   - 实现最小权限原则

3. **Token 撤销**：
   - 实现 Token 黑名单机制
   - 支持紧急撤销 Token

### 网络安全

1. **传输加密**：

   - 使用 TLS/SSL 加密 WebSocket 连接
   - 配置证书验证

2. **网络隔离**：
   - 使用防火墙限制访问
   - 配置网络分段

## 故障排除

### 常见错误

1. **Token 验证失败**：

   ```
   ERROR: JWE token validation failed: Invalid signature
   ```

   - 检查密钥配置是否正确
   - 确认 Token 未过期
   - 验证密钥格式

2. **密钥格式错误**：

   ```
   ERROR: Failed to parse private key: Invalid PEM format
   ```

   - 检查 PEM 格式是否正确
   - 确认换行符转义正确

3. **连接被拒绝**：
   ```
   ERROR: WebSocket connection rejected: Authentication required
   ```
   - 检查 Agent 是否配置了 JWE Token
   - 确认 Server 端 JWE 配置已启用

### 调试方法

1. **启用调试日志**：

   ```toml
   [fusion.log]
   log_level = "debug"
   log_targets = [
     "hetuflow_server=debug",
     "hetuflow_agent=debug"
   ]
   ```

2. **验证 Token 内容**：

   ```bash
   # 使用 JWT 调试工具查看 Token 内容
   echo "<JWE_TOKEN>" | base64 -d | jq .
   ```

3. **测试密钥对**：
   ```bash
   # 验证密钥对是否匹配
   openssl ec -in jwe-private.pem -pubout | diff - jwe-public.pem
   ```

## 监控和审计

### 监控指标

- Token 验证成功/失败次数
- Token 过期事件
- 异常认证尝试
- Agent 连接状态

### 审计日志

- Agent 注册事件
- Token 生成和使用记录
- 认证失败事件
- 权限变更记录

## 参考资料

- [RFC 7516: JSON Web Encryption (JWE)](https://tools.ietf.org/html/rfc7516)
- [RFC 7518: JSON Web Algorithms (JWA)](https://tools.ietf.org/html/rfc7518)
- [ECDH-ES Key Agreement Algorithm](https://tools.ietf.org/html/rfc7518#section-4.6)
- [OpenSSL 文档](https://www.openssl.org/docs/)
