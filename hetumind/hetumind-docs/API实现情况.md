# API 实现情况

**API 清单**

- 统一前缀与鉴权
  - 基础前缀：`/api`
  - 公开鉴权路由：`/api/auth/*`（不要求 Token）
  - 业务路由：`/api/v1/*`（要求 `Authorization: Bearer <token>`）

| 路径                                | 方法   | 请求参数/体                                                                                                                                                                                                                                                                   | 响应结构                                                                                       | 功能描述                         | 状态          |
| ----------------------------------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | -------------------------------- | ------------- |
| `/api/auth/signin`                  | POST   | `SigninRequest { account: string, password: string }`（支持邮箱或手机号）                                                                                                                                                                                                     | `SigninResponse { token: string, token_type: 'Bearer' }`                                       | 登录，返回访问令牌               | 已实现        |
| `/api/auth/signup`                  | POST   | `SignupRequest { email: string, password: string }`                                                                                                                                                                                                                           | `200 OK`                                                                                       | 注册新用户                       | 已实现        |
| `/api/v1/workflows/`                | POST   | `WorkflowForCreate { id?: WorkflowId, name: string, status?: WorkflowStatus, nodes?: json, connections?: json, settings?: json, static_data?: json, pin_data?: json, version_id?: WorkflowId, meta?: json }`                                                                  | `IdUuidResult { id: Uuid }`                                                                    | 创建或导入工作流                 | 已实现        |
| `/api/v1/workflows/query`           | POST   | `WorkflowForQuery { options: Page, filter: WorkflowFilter }`（`WorkflowFilter { name?: OpValString, status?: OpValInt32, ... }`）                                                                                                                                             | `PageResult<Workflow>`                                                                         | 查询工作流列表                   | 已实现        |
| `/api/v1/workflows/validate`        | POST   | `ValidateWorkflowRequest { id?: WorkflowId, workflow?: Workflow }`                                                                                                                                                                                                            | `ValidateWorkflowResponse { is_valid: bool, errors?: ValidationError[] }`                      | 校验工作流定义是否可激活         | 已实现        |
| `/api/v1/workflows/{id}`            | GET    | 路径参数 `id: WorkflowId`                                                                                                                                                                                                                                                     | `Workflow`                                                                                     | 获取工作流详情                   | 已实现        |
| `/api/v1/workflows/{id}`            | PUT    | 路径参数；`WorkflowForUpdate { name?: string, status?: WorkflowStatus, nodes?: json, connections?: json, settings?: json, static_data?: json, pin_data?: json, version_id?: WorkflowId, meta?: json, parent_folder_id?: string, is_archived?: bool, field_mask?: FieldMask }` | `IdUuidResult { id: Uuid }`                                                                    | 更新工作流                       | 已实现        |
| `/api/v1/workflows/{id}`            | DELETE | 路径参数                                                                                                                                                                                                                                                                      | `200 OK`                                                                                       | 删除工作流                       | 已实现        |
| `/api/v1/workflows/{id}/execute`    | POST   | 路径参数；`ExecuteWorkflowRequest { input_data?: ParameterMap }`                                                                                                                                                                                                              | `ExecutionIdResponse { execution_id: ExecutionId }`                                            | 触发执行工作流                   | 已实现        |
| `/api/v1/workflows/{id}/activate`   | POST   | 路径参数                                                                                                                                                                                                                                                                      | `200 OK`                                                                                       | 激活工作流                       | 已实现        |
| `/api/v1/workflows/{id}/deactivate` | POST   | 路径参数                                                                                                                                                                                                                                                                      | `200 OK`                                                                                       | 停用工作流                       | 已实现        |
| `/api/v1/workflows/{id}/duplicate`  | POST   | 路径参数                                                                                                                                                                                                                                                                      | `IdUuidResult { id: Uuid }`                                                                    | 复制工作流，返回新 ID            | 已实现        |
| `/api/v1/executions/query`          | POST   | `ExecutionForQuery { options: Page, filter: ExecutionFilter }`                                                                                                                                                                                                                | `PageResult<ExecutionResponse>`（`ExecutionResponse = Execution`）                             | 查询执行历史                     | 已实现        |
| `/api/v1/executions/{id}`           | GET    | 路径参数 `id: ExecutionId`                                                                                                                                                                                                                                                    | `ExecutionResponse`                                                                            | 获取执行详情                     | 已实现        |
| `/api/v1/executions/{id}/cancel`    | POST   | 路径参数                                                                                                                                                                                                                                                                      | `200 OK`                                                                                       | 取消执行                         | 已实现        |
| `/api/v1/executions/{id}/retry`     | POST   | 路径参数                                                                                                                                                                                                                                                                      | `200 OK`                                                                                       | 重试执行                         | 已实现        |
| `/api/v1/executions/{id}/logs`      | GET    | 路径参数                                                                                                                                                                                                                                                                      | `ExecutionLogResponse = Vec<ExecutionData>`                                                    | 获取执行日志                     | 已实现        |
| `/api/v1/credentials/`              | POST   | `CredentialForInsert { namespace_id: string, name: string, data: string, kind: CredentialKind, is_managed?: bool, id?: Uuid }`（`data` 为 JWE 编码前的原文 JSON）                                                                                                             | `IdUuidResult { id: Uuid }`                                                                    | 创建凭据（后端自动 JWE 加密）    | 已实现        |
| `/api/v1/credentials/query`         | POST   | `CredentialForQuery { page: Page, filters: CredentialFilter[] }`（如 `namespace_id/name/kind/...` 等 `OpVal*`）                                                                                                                                                               | `PageResult<CredentialEntity>`                                                                 | 查询凭据列表（不含敏感数据）     | 已实现        |
| `/api/v1/credentials/verify`        | POST   | `VerifyCredentialRequest { data: CredentialData, kind: CredentialKind }`（`CredentialData { data: string, test_connection?: bool }`，`data` 为原文 JSON 字符串）                                                                                                              | `CredentialVerifyResult { success: bool, message: string, verify_time: DateTime }`             | 验证未保存的凭据                 | 已实现/需改进 |
| `/api/v1/credentials/{id}`          | GET    | 路径参数 `id: Uuid`                                                                                                                                                                                                                                                           | `CredentialWithDecryptedData { credential: CredentialEntity, decrypted_data: CredentialData }` | 获取凭据详情（含解密数据）       | 已实现        |
| `/api/v1/credentials/{id}`          | PUT    | 路径参数；`CredentialForUpdate { namespace_id?: string, name?: string, data?: string, kind?: CredentialKind, is_managed?: bool }`                                                                                                                                             | `200 OK`                                                                                       | 更新凭据（自动加密敏感字段）     | 已实现        |
| `/api/v1/credentials/{id}`          | DELETE | 路径参数                                                                                                                                                                                                                                                                      | `200 OK`                                                                                       | 删除凭据                         | 已实现        |
| `/api/v1/credentials/{id}/verify`   | POST   | 路径参数                                                                                                                                                                                                                                                                      | `CredentialVerifyResult`                                                                       | 验证已保存凭据（使用已解密数据） | 已实现/需改进 |
| `/api/v1/users/item/{id}`           | GET    | 路径参数 `id: i64`                                                                                                                                                                                                                                                            | `Option<UserEntity>`                                                                           | 获取用户详情                     | 已实现        |
| `/api/v1/users/item/{id}`           | PUT    | 路径参数；`UserForUpdate { email?: string, phone?: string, name?: string, status?: UserStatus, update_mask?: FieldMask }`                                                                                                                                                     | `200 OK`                                                                                       | 更新用户                         | 已实现        |
| `/api/v1/users/query`               | POST   | `UserForPage { page: Page, filter: UserFilter }`（如 `email/phone/name/status/...`）                                                                                                                                                                                          | `PageResult<UserEntity>`                                                                       | 查询用户列表                     | 已实现        |

**API 状态分析与质量评估**

| 模块   | 端点                                              | 状态          | 质量评估                                                          | 改进建议                                                                                                                                                                                                                                        |
| ------ | ------------------------------------------------- | ------------- | ----------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 鉴权   | `/api/auth/signin`, `/api/auth/signup`            | 已实现        | 登录/注册流程完整，返回 `Bearer` Token；缺少 Refresh Token 与登出 | - 增加 `/auth/refresh` 返回短期 Access + 长期 Refresh Token<br>- 增加 `/auth/signout` 并加入 Token 黑名单机制（已有 InvalidAuthToken 表支持）<br>- 增加登录限流与失败审计                                                                       |
| 工作流 | 创建/查询/校验/获取/更新/删除/执行/激活/停用/复制 | 已实现        | 路由齐备；校验流程包含基础/连通性检查；执行返回 `execution_id`    | - 校验增强：节点类型定义缺失校验、端口匹配、参数 Schema 校验<br>- 执行 API 增强：支持异步队列模式字段（执行模式、优先级）、返回状态轮询 URL<br>- 更新 API 支持 `field_mask` 全量/部分更新一致性说明                                             |
| 执行   | 查询/详情/取消/重试/日志                          | 已实现        | 日志返回 `Vec<ExecutionData>`；取消/重试已挂载                    | - 日志分页与流式（SSE/WebSocket）增强，支持按节点过滤与时间窗口<br>- 增加 `/executions/{id}/status` 轻量状态查询，减少详情开销<br>- 取消/重试的幂等性与状态机约束（禁止已完成重试等）                                                           |
| 凭据   | 创建/查询/详情/更新/删除/校验(未保存/已保存)      | 已实现/需改进 | 加解密采用 JWE ECDH-ES；列表不含敏感字段；校验逻辑为占位          | - 完成 `verify_oauth2/verify_authenticate/verify_generic_auth` 的实际校验实现（当前为占位返回 true）<br>- `CredentialData.data` 建议统一为结构化 JSON（目前为字符串），并提供 Schema 约束<br>- 增加引用检查与删除保护（被工作流引用时禁止删除） |
| 用户   | 获取/更新/查询                                    | 已实现        | 基本 CRUD 完整；密码更新逻辑在 Svc 层（未暴露路由）               | - 暴露 `/users/item/{id}/password`（PUT）请求体 `UserForUpdatePassword { old_password?/code?, password }`<br>- 增加用户角色与权限字段，并在路由层进行授权控制                                                                                   |

**未实现/建议补充的 API（清单）**

| 路径                                  | 方法 | 建议请求体                  | 说明/用途                          | 优先级 |
| ------------------------------------- | ---- | --------------------------- | ---------------------------------- | ------ |
| `/api/auth/refresh`                   | POST | `{ refresh_token: string }` | 刷新 Access Token                  | 高     |
| `/api/auth/signout`                   | POST | 无或 `{ token }`            | 登出并加入黑名单                   | 中     |
| `/api/v1/executions/{id}/status`      | GET  | 路径参数                    | 轻量获取执行状态（避免拉大量详情） | 中     |
| `/api/v1/executions/{id}/logs/stream` | GET  | 路径参数（SSE/WebSocket）   | 流式订阅执行日志                   | 中     |
| `/api/v1/credentials/{id}/references` | GET  | 路径参数                    | 查询凭据被哪些工作流引用           | 中     |
| `/api/v1/users/item/{id}/password`    | PUT  | `UserForUpdatePassword`     | 用户密码更新流程                   | 高     |

**数据结构补充（关键类型字段）**

- `WorkflowForCreate`
  - `id?: WorkflowId`, `name: string`, `status?: WorkflowStatus`, `nodes?: json`, `connections?: json`, `settings?: json`, `static_data?: json`, `pin_data?: json`, `version_id?: WorkflowId`, `meta?: json`
- `WorkflowForUpdate`
  - `name?: string`, `status?: WorkflowStatus`, `nodes?: json`, `connections?: json`, `settings?: json`, `static_data?: json`, `pin_data?: json`, `version_id?: WorkflowId`, `meta?: json`, `parent_folder_id?: string`, `is_archived?: bool`, `field_mask?: FieldMask`
- `ValidateWorkflowRequest`
  - `id?: WorkflowId`, `workflow?: Workflow`（两者至少一个）
- `ValidateWorkflowResponse`
  - `is_valid: bool`, `errors?: ValidationError[]`
- `ExecuteWorkflowRequest`
  - `input_data?: ParameterMap`
- `ExecutionIdResponse`
  - `execution_id: ExecutionId`
- `ExecutionLogResponse`
  - `Vec<ExecutionData>`（支持 JSON 数据块）
- `CredentialForInsert`
  - `namespace_id: string`, `name: string`, `data: string`（原文 JSON 字符串）, `kind: CredentialKind`, `is_managed?: bool`, `id?: Uuid`
- `CredentialForUpdate`
  - `namespace_id?: string`, `name?: string`, `data?: string`, `kind?: CredentialKind`, `is_managed?: bool`
- `CredentialForQuery`
  - `page: Page`, `filters: CredentialFilter[]`（`namespace_id/name/data/kind/...` 带 `OpVal*` 操作符）
- `CredentialWithDecryptedData`
  - `credential: CredentialEntity`, `decrypted_data: CredentialData { data: string, test_connection?: bool }`
- `VerifyCredentialRequest`
  - `data: CredentialData`, `kind: CredentialKind`
- `CredentialVerifyResult`
  - `success: bool`, `message: string`, `verify_time: DateTime`
- `UserForPage`
  - `page: Page`, `filter: UserFilter`（`email/phone/name/status/...`）
- `UserForUpdate`
  - `email?: string`, `phone?: string`, `name?: string`, `status?: UserStatus`, `update_mask?: FieldMask`
- `SigninRequest/SignupRequest/SigninResponse`
  - `SigninRequest { account, password }`、`SignupRequest { email, password }`、`SigninResponse { token, token_type: 'Bearer' }`

**整体建议（落地方向）**

- 认证与安全
  - 增加 Refresh Token 与登出；无效令牌黑名单（已有表）；登录限流与审计。
- 执行与日志
  - 引入状态轻量查询、日志分页与流式；重试与取消的状态机约束。
- 凭据与引用安全
  - 强化校验实现，统一数据结构为 JSON Schema；引用保护与删除前检查。
- 校验质量
  - 工作流校验扩展（节点定义缺失、端口类型、参数校验）；返回结构化错误细节。
