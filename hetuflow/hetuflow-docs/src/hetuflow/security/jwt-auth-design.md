# hetuflow JWT 认证机制技术方案设计

## 1. 概述

本文档设计了一套基于 JWT (JSON Web Token) 的 Agent 与 Server 连接认证机制，确保只有经过授权的 Agent 能够连接到 hetuflow-server，提升系统安全性。

### 1.1 设计目标

- **安全性**: 防止未授权的 Agent 连接到 Server
- **简单性**: 最小化配置复杂度，易于部署和维护
- **兼容性**: 与现有 WebSocket 通信协议无缝集成
- **灵活性**: 支持 Token 有效期配置和管理

### 1.2 技术要求

- 基于现有 WebSocket 连接和 AgentRegisterRequest 流程
- Server 端提供 JWT Token 生成 API
- Agent 端通过配置文件设置 JWT Token
- 数据库访问无需认证
- 默认生成永久有效 Token，支持自定义有效期
- 使用 josekit 库作为 JWT 实现，提供完整的 JOSE 标准支持

### 1.3 依赖库选择

本方案选择 **josekit** 作为 JWE 实现库，相比其他 Rust JWT 库具有以下优势： <mcreference link="https://docs.rs/josekit" index="4">4</mcreference>

- **完整的 JOSE 支持**：支持 JWT、JWS、JWE、JWA、JWK 等完整的 JOSE 标准
- **强加密支持**：支持 ECDH-ES 密钥协商和 A256GCM 内容加密算法
- **类型安全**：提供强类型的 API 接口，减少运行时错误
- **高性能**：优化的实现，适合高并发场景
- **活跃维护**：持续更新和维护，支持最新的安全标准

#### 安全升级说明

**从 JWT 升级到 JWE**：

- **JWT (JSON Web Token)**：仅提供签名验证，Token 内容可被解码查看
- **JWE (JSON Web Encryption)**：提供完整加密保护，Token 内容完全不可见
- **安全优势**：防止 Token 内容泄露，保护敏感信息如过期时间、agent_id 等

#### Cargo.toml 依赖配置

```toml
[dependencies]
josekit = "0.10.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
```

## 2. 架构设计

### 2.1 认证流程图

```mermaid
sequenceDiagram
    participant Admin as 管理员
    participant API as Server JWT API
    participant Agent as hetuflow-agent
    participant Server as hetuflow-server
    participant WS as WebSocket Handler
    participant Auth as JWT验证器

    Note over Admin,Auth: Token生成阶段
    Admin->>API: POST /api/v1/auth/generate-token
    API->>API: 生成JWT Token
    API-->>Admin: 返回JWT Token字符串
    Admin->>Agent: 配置Token到app.toml

    Note over Agent,Auth: Agent连接认证阶段
    Agent->>Server: WebSocket连接请求
    Server-->>Agent: WebSocket连接建立
    Agent->>WS: AgentRegisterRequest (包含JWT Token)
    WS->>Auth: 验证JWT Token
    Auth-->>WS: Token验证结果

    alt Token有效
        WS->>Server: 处理Agent注册
        Server-->>Agent: AgentRegistered响应
    else Token无效
        WS-->>Agent: 认证失败，关闭连接
    end
```

### 2.2 组件架构

```mermaid
graph TB
    subgraph "hetuflow-server"
        subgraph "JWT认证模块"
            JWTGen[JWT生成器]
            JWTValidator[JWT验证器]
            JWTConfig[JWT配置]
        end

        subgraph "API模块"
            AuthAPI[认证API端点]
        end

        subgraph "WebSocket模块"
            WSHandler[WebSocket处理器]
            AgentManager[Agent管理器]
        end
    end

    subgraph "hetuflow-agent"
        AgentConfig[Agent配置]
        WSClient[WebSocket客户端]
    end

    subgraph "管理员"
        Admin[系统管理员]
    end

    Admin --> AuthAPI
    AuthAPI --> JWTGen
    JWTGen --> JWTConfig

    AgentConfig --> WSClient
    WSClient --> WSHandler
    WSHandler --> JWTValidator
    JWTValidator --> JWTConfig
    WSHandler --> AgentManager

    style JWTGen fill:#e8f5e8
    style JWTValidator fill:#f3e5f5
    style AuthAPI fill:#e1f5fe
    style AgentConfig fill:#fff3e0
```

## 3. 技术实现方案

### 3.1 JWE Token 结构设计

#### 3.1.1 JWE Header

```json
{
  "alg": "ECDH-ES",
  "enc": "A256GCM",
  "typ": "JWE",
  "cty": "JWT"
}
```

#### 3.1.2 加密前的 JWT Payload（敏感信息完全加密保护）

```json
{
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "iss": "hetuflow-server",
  "aud": "hetuflow-system",
  "iat": 1640995200,
  "exp": null,
  "agent_id": "550e8400-e29b-41d4-a716-446655440000",
  "permissions": ["agent:connect", "task:execute"],
  "jti": "unique-token-id"
}
```

#### 3.1.3 字段说明

- `sub`: 主题，存储特定的 agent_id（UUID 格式）
- `iss`: 签发者，固定为 "hetuflow-server"
- `aud`: 受众，固定为 "hetuflow-system"
- `iat`: 签发时间戳
- `exp`: 过期时间戳，null 表示永不过期
- `agent_id`: Agent ID（与 sub 字段一致）
- `permissions`: 权限列表
- `jti`: Token 唯一标识

#### 3.1.4 安全特性

- **完全加密**：所有 Payload 内容通过 A256GCM 算法加密，外部无法解析
- **专属性**：每个 Token 的 `sub` 字段存储特定的 `agent_id`
- **双重验证**：服务端同时验证连接参数和 Token 中的 `agent_id`
- **防篡改**：ECDH-ES + A256GCM 提供完整性和机密性保护

### 3.2 Server 端实现

#### 3.2.1 JWE 配置结构

```rust
// hetuflow-server/src/security/jwe_config.rs
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct JweConfig {
    /// JWE 加密私钥（ECDH-ES 算法），建议使用环境变量或安全配置
    pub private_key: String,
    /// JWE 加密公钥（ECDH-ES 算法）
    pub public_key: String,
    /// 默认Token有效期（秒），None表示永不过期
    pub expiry_seconds: Option<u64>,
    /// Token签发者
    pub issuer: String,
    /// Token受众
    pub audience: String,
    /// 密钥协商算法
    pub key_agreement_alg: String,
    /// 内容加密算法
    pub content_encryption_alg: String,
}

impl Default for JweConfig {
    fn default() -> Self {
        Self {
            private_key: "your-ecdh-private-key-here-change-in-production".to_string(),
            public_key: "your-ecdh-public-key-here-change-in-production".to_string(),
            expiry_seconds: None, // 永不过期
            issuer: "hetuflow-server".to_string(),
            audience: "hetuflow-system".to_string(),
            key_agreement_alg: "ECDH-ES".to_string(),
            content_encryption_alg: "A256GCM".to_string(),
        }
    }
}
```

#### 3.2.2 JWE 服务实现

```rust
// hetuflow-server/src/security/jwe_service.rs
use josekit::jwt::{self, JwtPayload};
use josekit::jwe::{ECDH_ES, A256GCM, JweHeader};
use josekit::jwk::Jwk;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use fusion_core::DataError;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentClaims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub iat: u64,
    pub exp: Option<u64>,
    pub agent_id: String,  // 必须字段，不再是 Option
    pub permissions: Vec<String>,
    pub jti: String,
}

pub struct JweService {
    config: JweConfig,
    encrypter: ECDH_ES,
    decrypter: ECDH_ES,
    private_jwk: Jwk,
    public_jwk: Jwk,
}

impl JweService {
    pub fn new(config: JweConfig) -> Result<Self, DataError> {
        // 创建ECDH-ES私钥用于JWE解密
        let private_jwk = Jwk::from_bytes(config.private_key.as_bytes())
            .map_err(|e| DataError::server_error(format!("Private JWK creation error: {}", e)))?;

        // 创建ECDH-ES公钥用于JWE加密
        let public_jwk = Jwk::from_bytes(config.public_key.as_bytes())
            .map_err(|e| DataError::server_error(format!("Public JWK creation error: {}", e)))?;

        let encrypter = ECDH_ES.encrypter_from_jwk(&public_jwk)
            .map_err(|e| DataError::server_error(format!("JWE encrypter creation error: {}", e)))?;

        let decrypter = ECDH_ES.decrypter_from_jwk(&private_jwk)
            .map_err(|e| DataError::server_error(format!("JWE decrypter creation error: {}", e)))?;

        Ok(Self {
            config,
            encrypter,
            decrypter,
            private_jwk,
            public_jwk,
        })
    }
    /// 生成Agent JWE Token（加密保护）
    pub fn generate_agent_token(
        &self,
        agent_id: String,  // 必须提供 agent_id
        expiry_seconds: Option<u64>,
    ) -> Result<String, DataError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| DataError::server_error(format!("Time error: {}", e)))?
            .as_secs();

        let exp = expiry_seconds
            .or(self.config.expiry_seconds)
            .map(|seconds| now + seconds);

        // 创建JWT payload
        let mut payload = JwtPayload::new();
        payload.set_subject(&agent_id);  // 使用 agent_id 作为 subject
        payload.set_issuer(&self.config.issuer);
        payload.set_audience(vec![self.config.audience.clone()]);
        payload.set_issued_at(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(now));

        if let Some(exp_time) = exp {
            payload.set_expires_at(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(exp_time));
        }

        // 添加自定义声明
        payload.set_claim("agent_id", Some(serde_json::Value::String(agent_id.clone())))
            .map_err(|e| DataError::server_error(format!("Set agent_id claim error: {}", e)))?;

        let permissions = vec!["agent:connect".to_string(), "task:execute".to_string()];
        payload.set_claim("permissions", Some(serde_json::Value::Array(
            permissions.into_iter().map(serde_json::Value::String).collect()
        ))).map_err(|e| DataError::server_error(format!("Set permissions claim error: {}", e)))?;

        // 添加唯一标识符
        let jti = uuid::Uuid::new_v4().to_string();
        payload.set_claim("jti", Some(serde_json::Value::String(jti)))
            .map_err(|e| DataError::server_error(format!("Set jti claim error: {}", e)))?;

        // 创建JWE header
        let mut header = JweHeader::new();
        header.set_algorithm(&self.config.key_agreement_alg);
        header.set_content_encryption(&self.config.content_encryption_alg);
        header.set_content_type("JWT");

        // 生成JWE token（加密保护）
        jwt::encode_with_encrypter(&payload, &header, &self.encrypter)
            .map_err(|e| DataError::server_error(format!("JWE encoding error: {}", e)))
    }

    /// 验证Agent JWE Token（解密并验证）
    pub fn validate_agent_token(&self, token: &str, expected_agent_id: &str) -> Result<AgentClaims, DataError> {
        // 解密并验证JWE token
        let (payload, _header) = jwt::decode_with_decrypter(token, &self.decrypter)
            .map_err(|e| DataError::unauthorized(format!("JWE validation error: {}", e)))?;

        // 验证issuer
        if payload.issuer() != Some(&self.config.issuer) {
            return Err(DataError::unauthorized("Invalid issuer"));
        }

        // 验证audience
        if let Some(audiences) = payload.audience() {
            if !audiences.contains(&self.config.audience) {
                return Err(DataError::unauthorized("Invalid audience"));
            }
        } else {
            return Err(DataError::unauthorized("Missing audience"));
        }

        // 提取并验证 agent_id
        let token_agent_id = payload.claim("agent_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DataError::unauthorized("Missing agent_id in token"))?;

        // 双重验证：Token中的agent_id必须与连接请求中的agent_id一致
        if token_agent_id != expected_agent_id {
            return Err(DataError::unauthorized(
                format!("Agent ID mismatch: token contains '{}', but connection request has '{}'",
                       token_agent_id, expected_agent_id)
            ));
        }

        // 验证subject也应该与agent_id一致
        if payload.subject() != Some(expected_agent_id) {
            return Err(DataError::unauthorized("Subject mismatch with agent_id"));
        }

        let permissions = payload.claim("permissions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let jti = payload.claim("jti")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        // 构造AgentClaims结构
        let claims = AgentClaims {
            sub: payload.subject().unwrap_or(expected_agent_id).to_string(),
            iss: payload.issuer().unwrap_or(&self.config.issuer).to_string(),
            aud: self.config.audience.clone(),
            iat: payload.issued_at()
                .map(|t| t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs())
                .unwrap_or(0),
            exp: payload.expires_at()
                .map(|t| t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs()),
            agent_id: token_agent_id.to_string(),
            permissions,
            jti,
        };

        Ok(claims)
    }
}
```

#### 3.2.3 认证 API 端点（仅本机访问）

```rust
// hetuflow-server/src/endpoint/api/v1/auth.rs
use axum::{extract::{ConnectInfo, State}, http::StatusCode, response::Json, routing::post, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;
use crate::security::jwe_service::JweService;
use fusion_core::DataError;

#[derive(Debug, Deserialize)]
pub struct GenerateTokenRequest {
    /// 必须的Agent ID（JWE Token专属绑定）
    pub agent_id: String,
    /// Token有效期（秒），None表示使用默认配置
    pub expiry_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct GenerateTokenResponse {
    pub token: String,
    pub agent_id: String,
    pub expires_at: Option<u64>,
    pub issued_at: u64,
}

/// 检查请求是否来自本机
fn is_localhost(addr: &SocketAddr) -> bool {
    match addr.ip() {
        std::net::IpAddr::V4(ipv4) => ipv4.is_loopback(),
        std::net::IpAddr::V6(ipv6) => ipv6.is_loopback(),
    }
}

/// 生成Agent JWE Token（仅允许本机访问）
async fn generate_agent_token(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(jwe_service): State<Arc<JweService>>,
    Json(request): Json<GenerateTokenRequest>,
) -> Result<Json<GenerateTokenResponse>, DataError> {
    // 安全检查：仅允许本机访问
    if !is_localhost(&addr) {
        return Err(DataError::forbidden(
            "Token generation is only allowed from localhost"
        ));
    }

    // 验证 agent_id 格式（必须是有效的 UUID）
    Uuid::parse_str(&request.agent_id)
        .map_err(|_| DataError::bad_request("agent_id must be a valid UUID"))?;

    let token = jwe_service.generate_agent_token(
        request.agent_id.clone(),
        request.expiry_seconds,
    )?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let expires_at = request.expiry_seconds
        .map(|seconds| now + seconds);

    Ok(Json(GenerateTokenResponse {
        token,
        agent_id: request.agent_id,
        expires_at,
        issued_at: now,
    }))
}

pub fn auth_routes() -> Router<Arc<JweService>> {
    Router::new()
        .route("/generate-token", post(generate_agent_token))
}

// 在应用启动时创建JweService实例
pub fn create_jwe_service(config: JweConfig) -> Result<Arc<JweService>, DataError> {
    let jwe_service = JweService::new(config)?;
    Ok(Arc::new(jwe_service))
}
```

#### 3.2.4 WebSocket 认证集成

```rust
// 修改 hetuflow-core/src/protocol.rs 中的 AgentRegisterRequest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegisterRequest {
    pub agent_id: String,
    pub capabilities: AgentCapabilities,
    pub address: String,
    /// JWT认证Token
    pub jwt_token: String,
}
```

```rust
// 修改 hetuflow-server/src/service/agent_manager.rs 中的注册处理逻辑
use crate::security::jwe_service::JweService;

impl AgentManager {
    // 在handle_register方法中添加JWT验证
    async fn handle_register_with_auth(
        &self,
        agent_id: &str,
        payload: &AgentRegisterRequest,
        jwe_service: &JweService,
    ) -> Result<AgentRegisterResponse, DataError> {
        // 1. 验证JWE Token（包含双重agent_id验证）
        let claims = jwe_service.validate_agent_token(&payload.jwt_token, agent_id)?;

        // 2. Agent ID匹配验证已在validate_agent_token中完成

        // 3. 检查权限
        if !claims.permissions.contains(&"agent:connect".to_string()) {
            return Err(DataError::unauthorized("Insufficient permissions for agent connection"));
        }

        // 4. 继续原有的注册逻辑
        self.handle_register(agent_id, payload).await
    }
}
```

### 3.3 Agent 端实现

#### 3.3.1 配置文件扩展

```toml
# hetuflow-agent/resources/app.toml
[hetuflow.agent]
# ... 现有配置 ...

# JWT认证配置
jwt_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

#### 3.3.2 Agent 配置结构扩展

```rust
// 修改 hetuflow-agent/src/config.rs 或相应配置文件
#[derive(Debug, Clone, serde::Deserialize)]
pub struct HetuflowAgentSetting {
    // ... 现有字段 ...

    /// JWT认证Token
    pub jwt_token: String,
}
```

#### 3.3.3 WebSocket 客户端认证集成

```rust
// 修改 hetuflow-agent/src/service/ws_runner.rs 中的register_agent方法
impl WsRunner {
    async fn register_agent(
        &self,
        ws_tx: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        address: String,
    ) -> Result<(), DataError> {
        let capabilities = AgentCapabilities {
            max_concurrent_tasks: self.setting.process.max_concurrent_processes,
            tags: self.setting.tags.iter().map(|tag| (tag.clone(), Default::default())).collect(),
            metadata: self.setting.metadata.clone(),
        };

        let register_req = AgentRegisterRequest {
            agent_id: self.setting.agent_id.clone(),
            capabilities,
            address,
            jwt_token: self.setting.jwt_token.clone(), // 添加JWT Token
        };

        let message = serde_json::to_string(&WebSocketEvent::new(EventKind::AgentRegister, register_req))
            .unwrap();

        ws_tx
            .send(Message::Text(message.into()))
            .await
            .map_err(|e| DataError::server_error(format!("Send message to Server error: {}", e)))
    }
}
```

## 4. 配置管理

### 4.1 Server 端配置

````toml
# hetuflow-server/resources/app.toml
[hetuflow.server.jwe]
# JWE 加密私钥（ECDH-ES 算法，生产环境建议使用环境变量）
private_key = "${JWE_PRIVATE_KEY:-----BEGIN PRIVATE KEY-----\nMIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQg...\n-----END PRIVATE KEY-----}"
# JWE 加密公钥（ECDH-ES 算法）
public_key = "${JWE_PUBLIC_KEY:-----BEGIN PUBLIC KEY-----\nMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE...\n-----END PUBLIC KEY-----}"
# 默认Token有效期（秒），null表示永不过期
expiry_seconds = null
# Token签发者
issuer = "hetuflow-server"
# Token受众
audience = "hetuflow-agent"
# 密钥协商算法
key_agreement_alg = "ECDH-ES"
# 内容加密算法
content_encryption_alg = "A256GCM"

# API 服务绑定配置
[hetuflow.server.api]
# 认证接口仅允许本机访问
auth_bind = "127.0.0.1:8080"
# 其他接口允许外部访问
general_bind = "0.0.0.0:8080"
# Token受众
audience = "hetuflow-system"

[fusion.web]
enable = true
server_addr = "0.0.0.0:9500"```

### 4.2 Agent 端配置

```toml
# hetuflow-agent/resources/app.toml
[hetuflow.agent]
# ... 现有配置 ...

# JWE认证Token（从Server端API获取，完全加密保护）
jwe_token = "eyJhbGciOiJFQ0RILUVTIiwiZW5jIjoiQTI1NkdDTSIsInR5cCI6IkpXRSIsImN0eSI6IkpXVCJ9.encrypted_key.iv.ciphertext.tag"
````

### 4.3 密钥生成脚本

```bash
#!/bin/bash
# generate_jwe_keys.sh - 生成 ECDH-ES 密钥对

echo "正在生成 JWE 密钥对..."

# 生成私钥
openssl ecparam -genkey -name prime256v1 -noout -out jwe_private_key.pem

# 生成公钥
openssl ec -in jwe_private_key.pem -pubout -out jwe_public_key.pem

echo "JWE 密钥对已生成："
echo "私钥：jwe_private_key.pem"
echo "公钥：jwe_public_key.pem"
echo ""
echo "请将密钥内容复制到配置文件中，并确保私钥安全存储！"
echo "注意：私钥泄露将导致所有 Token 被破解！"
```

## 5. 部署和使用流程

### 5.1 JWE Token 生成流程

1. **生成密钥对**：使用提供的脚本生成 ECDH-ES 密钥对

   ```bash
   chmod +x generate_jwe_keys.sh
   ./generate_jwe_keys.sh
   ```

2. **配置 Server**：将生成的密钥对配置到 hetuflow-server 的 app.toml 中

3. **启动 Server**：确保 hetuflow-server 正常运行

4. **生成 Token**：调用 API 生成 JWE Token（仅本机访问）

   ```bash
   curl -X POST http://127.0.0.1:8080/api/v1/auth/generate-token \
     -H "Content-Type: application/json" \
     -d '{
       "agent_id": "550e8400-e29b-41d4-a716-446655440000",
       "expiry_seconds": null
     }'
   ```

   **响应示例**：

   ```json
   {
     "token": "eyJhbGciOiJFQ0RILUVTIiwiZW5jIjoiQTI1NkdDTSIsInR5cCI6IkpXRSIsImN0eSI6IkpXVCJ9...",
     "agent_id": "550e8400-e29b-41d4-a716-446655440000",
     "expires_at": null,
     "issued_at": 1704153600
   }
   ```

5. **配置 Agent**：将返回的 Token 配置到对应 Agent 的 app.toml 文件中

6. **启动 Agent**：Agent 将使用配置的 Token 进行认证连接

### 5.2 安全验证机制

- **专属性验证**：每个 Token 只能由特定的 agent_id 使用
- **双重验证**：
  - WebSocket 连接参数中的 agent_id
  - JWE Token 解密后 sub 字段中的 agent_id
  - 两者必须完全一致才能通过验证
- **完全加密**：Token 内容完全加密，外部无法解析

### 5.3 Token 管理

- **Token 轮换**：定期生成新 Token 并更新对应 Agent 配置
- **Token 撤销**：通过更改 Server 端私钥使所有现有 Token 失效
- **权限控制**：通过 Token 中的 permissions 字段控制 Agent 权限
- **专属绑定**：每个 Token 与特定 agent_id 绑定，无法跨 Agent 使用

## 6. 安全考虑

### 6.1 密钥管理

- **生产环境**：使用环境变量或安全配置管理系统存储 JWE 密钥对
- **密钥轮换**：定期更换 ECDH-ES 密钥对，使所有现有 Token 失效
- **密钥强度**：使用 P-256 椭圆曲线（prime256v1），提供 128 位安全强度
- **私钥保护**：私钥必须严格保密，泄露将导致所有 Token 被破解
- **密钥分离**：公钥可以相对安全地存储，私钥需要最高级别保护

### 6.2 Token 安全

- **完全加密**：JWE 提供完整的内容加密，Token 内容完全不可见
- **传输安全**：确保 WebSocket 连接使用 WSS（TLS 加密）
- **存储安全**：Agent 配置文件应设置适当的文件权限（600 或更严格）
- **Token 专属性**：每个 Token 与特定 agent_id 绑定，防止跨 Agent 使用
- **双重验证**：连接参数和 Token 内容的 agent_id 必须一致
- **Token 撤销**：通过更换私钥实现全局 Token 撤销

### 6.3 网络安全

- **API 访问限制**：JWE Token 生成 API 严格限制仅本机访问（127.0.0.1）
- **防火墙配置**：限制 WebSocket 端口的访问来源
- **监控告警**：记录认证失败事件并设置告警
- **DDoS 防护**：对认证接口实施速率限制

### 6.4 加密算法安全

- **ECDH-ES**：椭圆曲线 Diffie-Hellman 临时静态密钥协商
- **A256GCM**：AES-256-GCM 认证加密，提供机密性和完整性
- **抗量子准备**：考虑未来升级到抗量子加密算法

## 7. 错误处理

### 7.1 认证失败场景

- **Token 格式错误**：返回 400 Bad Request

  - JWE Token 格式不正确
  - 缺少必要的 JWE 组件（header、encrypted_key、iv、ciphertext、tag）

- **Token 解密失败**：返回 401 Unauthorized

  - 密钥不匹配或已更换
  - Token 被篡改或损坏

- **Token 过期**：返回 401 Unauthorized

  - Token 中的 exp 字段已过期

- **Agent ID 验证失败**：返回 403 Forbidden

  - 连接参数中的 agent_id 与 Token 中的 sub 字段不匹配
  - Token 中缺少 agent_id 声明
  - agent_id 格式不是有效的 UUID

- **权限不足**：返回 403 Forbidden

  - Token 中缺少必要的权限

- **API 访问限制**：返回 403 Forbidden
  - 非本机访问 Token 生成接口

### 7.2 错误响应格式

```json
{
  "error": {
    "code": "JWE_AUTHENTICATION_FAILED",
    "message": "Agent ID mismatch: token contains '550e8400-e29b-41d4-a716-446655440001', but connection request has '550e8400-e29b-41d4-a716-446655440000'",
    "details": {
      "timestamp": "2024-01-01T12:00:00Z",
      "expected_agent_id": "550e8400-e29b-41d4-a716-446655440000",
      "token_agent_id": "550e8400-e29b-41d4-a716-446655440001",
      "error_type": "AGENT_ID_MISMATCH"
    }
  }
}
```

### 7.3 常见错误代码

- `JWE_DECRYPTION_FAILED`：JWE Token 解密失败
- `AGENT_ID_MISMATCH`：Agent ID 不匹配
- `INVALID_AGENT_ID_FORMAT`：Agent ID 格式无效
- `TOKEN_EXPIRED`：Token 已过期
- `INSUFFICIENT_PERMISSIONS`：权限不足
- `LOCALHOST_ACCESS_ONLY`：仅允许本机访问

## 8. 监控和日志

### 8.1 关键事件记录

- JWE Token 生成事件（记录 agent_id 和生成时间）
- Agent 认证成功/失败事件（包含双重验证结果）
- Token 解密失败事件（可能的攻击尝试）
- Agent ID 不匹配事件（安全违规）
- Token 过期事件
- 权限检查失败事件
- 非本机访问 Token 生成接口事件

### 8.2 监控指标

- 认证成功率（按 agent_id 分组）
- JWE Token 使用情况和生命周期
- 认证失败次数和失败类型分布
- Agent 连接状态和活跃度
- Token 解密失败频率（安全指标）
- API 访问限制触发次数

## 9. 测试策略

### 9.1 单元测试

- JWE Token 生成和解密功能测试
- ECDH-ES 密钥协商测试
- A256GCM 加密/解密测试
- Agent ID 双重验证逻辑测试
- Token 过期处理测试
- 权限检查逻辑测试

### 9.2 集成测试

- Agent 注册认证完整流程测试
- JWE Token 专属性验证测试
- 认证失败场景测试（各种错误类型）
- Token 轮换流程测试
- 本机访问限制测试

### 9.3 安全测试

- JWE Token 篡改检测测试
- 跨 Agent Token 使用防护测试
- 重放攻击防护测试
- 权限绕过测试
- 密钥泄露影响评估测试
- 暴力破解防护测试

## 10. 总结

本技术方案设计了一套完整的基于 JWE 的 Agent 认证机制，相比传统 JWT 方案具有以下显著优势：

### 10.1 安全性升级

- **完全加密保护**：采用 JWE 标准，Token 内容完全加密，外部无法解析
- **强加密算法**：使用 ECDH-ES + A256GCM，提供业界领先的加密强度
- **专属性保障**：每个 Token 与特定 agent_id 绑定，防止跨 Agent 使用
- **双重验证机制**：连接参数和 Token 内容的 agent_id 必须一致

### 10.2 技术特性

- **标准兼容**：基于 JOSE 标准，具有良好的互操作性
- **高性能**：椭圆曲线加密算法，计算效率高
- **易于管理**：支持密钥轮换和全局 Token 撤销
- **完整错误处理**：详细的错误分类和处理机制

### 10.3 部署优势

- **本机安全**：Token 生成接口仅允许本机访问
- **配置简单**：提供密钥生成脚本和配置模板
- **无缝集成**：与现有 WebSocket 通信协议完美集成
- **监控完善**：全面的日志记录和监控指标

该方案在确保最高级别安全性的同时，保持了 hetuflow 系统的简洁性和高性能特性，为分布式 Agent 架构提供了企业级的安全保障。
