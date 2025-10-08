# Hetuflow SDK WASM API Implementation Guide

本指南详细说明如何为 Hetuflow SDK WebAssembly 绑定实现实际的 API 方法。

## 概述

我们已经成功为 HetuflowClient 添加了 `wasm_bindgen` 支持，并实现了基础的 WasmAgentsApi。现在需要继续实现其他 API 并连接到真实的 Rust API 实现。

## 当前状态

### ✅ 已完成

1. **基础架构**

   - WASM 模块结构 (`src/wasm.rs`)
   - 序列化/反序列化支持
   - TypeScript 类型定义
   - 基础客户端包装类

2. **WasmAgentsApi 实现**

   - 基础方法结构
   - Promise 基础的异步 API
   - 错误处理
   - JSON 序列化支持

3. **示例和文档**
   - 更新的 HTML 示例页面
   - 完整的使用文档

### 🔄 进行中

- 其他 API 的实现（Jobs, Tasks, Schedules 等）
- 连接到真实的 Rust API 实现

## 实现步骤

### 第一步：理解架构

当前的 WASM 架构包括：

1. **WasmHetuflowClient** - 主客户端包装类
2. **WasmConfig** - 配置管理
3. **API 包装类** - 每个 Rust API 对应的 WASM 包装
4. **序列化模块** - 处理 Rust 和 JavaScript 之间的数据转换

### 第二步：API 实现模式

每个 API 包装类都遵循相同的模式：

```rust
#[wasm_bindgen]
pub struct WasmXxxApi {
    _client: std::marker::PhantomData<HetuflowClient>,
}

impl WasmXxxApi {
    fn new(_client: &HetuflowClient) -> Self {
        Self {
            _client: std::marker::PhantomData,
        }
    }
}

#[wasm_bindgen]
impl WasmXxxApi {
    #[wasm_bindgen]
    pub fn query(&self, params: JsValue) -> Promise {
        future_to_promise(async move {
            // 实际实现逻辑
        })
    }

    // 其他方法: get, create, update, delete
}
```

### 第三步：连接真实 API

要将 WASM API 连接到真实的 Rust API 实现，需要：

1. **序列化输入参数**

   ```rust
   let query_params = serialization::from_js_value::<QueryType>(&params)?;
   ```

2. **调用 Rust API**

   ```rust
   let result = self.inner.xxx_api().method(query_params).await?;
   ```

3. **序列化输出结果**
   ```rust
   let js_result = serialization::to_js_value(&result)?;
   Ok(js_result)
   ```

## 具体实现示例

### Agents API 完整实现

```rust
#[wasm_bindgen]
impl WasmAgentsApi {
    #[wasm_bindgen]
    pub fn query(&self, params: JsValue) -> Promise {
        future_to_promise(async move {
            // 1. 反序列化查询参数
            let query_params = serialization::from_js_value::<AgentForQuery>(&params)
                .map_err(|e| JsValue::from_str(&format!("Invalid query parameters: {}", e)))?;

            // 2. 调用实际的 Agents API
            let agents_api = crate::apis::AgentsApi::new(&self.inner);
            let result = agents_api.query(query_params).await
                .map_err(|e| JsValue::from_str(&format!("API error: {}", e)))?;

            // 3. 序列化结果
            serialization::to_js_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
        })
    }

    #[wasm_bindgen]
    pub fn create(&self, data: JsValue) -> Promise {
        future_to_promise(async move {
            let agent_data = serialization::from_js_value::<AgentForCreate>(&data)?;
            let agents_api = crate::apis::AgentsApi::new(&self.inner);
            let result = agents_api.create(agent_data).await?;
            serialization::to_js_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
        })
    }

    // 类似地实现 get, update, delete 方法
}
```

### 错误处理策略

```rust
// 定义 WASM 特定的错误类型
#[wasm_bindgen]
pub struct WasmError;

#[wasm_bindgen]
impl WasmError {
    pub fn from_sdk_error(error: String) -> JsError {
        JsError::new(&error)
    }

    pub fn network_error(message: String) -> JsError {
        JsError::new(&format!("Network Error: {}", message))
    }

    pub fn validation_error(message: String) -> JsError {
        JsError::new(&format!("Validation Error: {}", message))
    }
}
```

## 实现优先级

### 高优先级（核心功能）

1. **WasmJobsApi** - 作业管理

   - query, create, get, update, delete
   - 额外方法: start, stop, pause, resume

2. **WasmTasksApi** - 任务管理

   - query, create, get, update, delete
   - 任务依赖关系处理

3. **WasmSchedulesApi** - 调度管理
   - query, create, get, update, delete
   - enable, disable 方法

### 中优先级

4. **WasmTaskInstancesApi** - 任务实例管理

   - query, get, create, update, delete
   - retry, cancel, getLogs 方法

5. **WasmServersApi** - 服务器管理
   - query, create, get, update, delete
   - connect, disconnect, getStatus 方法

### 低优先级

6. **WasmSystemApi** - 系统操作

   - info, health, metrics, version
   - shutdown, restart 方法

7. **WasmGatewayApi** - 网关操作

   - route, getRoutes, addRoute, removeRoute

8. **WasmAuthApi** - 认证操作
   - login, logout, refresh, verify
   - getToken, setToken, getCurrentUser

## 技术挑战和解决方案

### 1. 序列化挑战

**问题**: Rust 模型类型和 JavaScript 对象之间的转换

**解决方案**:

- 使用 `serde-wasm-bindgen` 进行序列化
- 为复杂类型创建自定义序列化器
- 使用 `JsValue` 作为中间表示

### 2. 异步处理

**问题**: Rust async/await 和 JavaScript Promises 的转换

**解决方案**:

- 使用 `wasm-bindgen-futures::future_to_promise`
- 确保所有异步操作都正确包装
- 处理超时和取消操作

### 3. 错误处理

**问题**: Rust 错误类型和 JavaScript Error 的转换

**解决方案**:

- 创建统一的错误转换机制
- 保持错误信息的完整性
- 提供错误分类和上下文

### 4. 类型安全

**问题**: 确保类型安全的同时提供灵活的 JavaScript API

**解决方案**:

- 完整的 TypeScript 类型定义
- 运行时类型验证
- 渐进式类型检查

## 开发工作流

### 1. 实现新 API

```bash
# 1. 实现新的 API 包装类
# 编辑 src/wasm.rs

# 2. 添加 TypeScript 类型定义
# 编辑 hetuflow_sdk.d.ts

# 3. 测试编译
cargo check --target wasm32-unknown-unknown --features with-wasm

# 4. 构建 WASM 包
wasm-pack build --target web --features with-wasm

# 5. 测试功能
# 使用 example.html 或创建专门的测试
```

### 2. 调试技巧

```rust
// 在 WASM 中使用 console.log 进行调试
web_sys::console::log_1(&"Debug message".into());

// 检查序列化结果
let json_str = serialization::to_json_string(&params)?;
web_sys::console::log_1(&format!("Serialized: {}", json_str).into());
```

### 3. 性能优化

```rust
// 避免不必要的序列化/反序列化
// 重用 JsValue 对象
// 使用批量操作减少网络请求
```

## 测试策略

### 1. 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_agents_query() {
        // 测试查询功能
    }
}
```

### 2. 集成测试

```javascript
// 在浏览器中测试
describe('WasmAgentsApi', () => {
  test('should query agents', async () => {
    const result = await client.agents().query({ page: 1, limit: 10 });
    expect(result).toBeDefined();
  });
});
```

### 3. 端到端测试

```javascript
// 使用实际的服务器测试
test('should create and retrieve agent', async () => {
  const agentData = { name: 'Test Agent' };
  const created = await client.agents().create(agentData);
  const retrieved = await client.agents().get(created.id);
  expect(retrieved.name).toBe('Test Agent');
});
```

## 部署和发布

### 1. 构建

```bash
# 开发构建
wasm-pack build --target web --features with-wasm

# 生产构建
wasm-pack build --target web --features with-wasm --release
```

### 2. 发布

```bash
# 发布到 npm
cd pkg
npm publish

# 或者作为私有包发布
npm publish --registry=https://your-registry.com
```

### 3. 版本管理

```bash
# 更新版本号
npm version patch  # 0.1.1 -> 0.1.2
npm version minor  # 0.1.1 -> 0.2.0
npm version major  # 0.1.1 -> 1.0.0
```

## 最佳实践

### 1. 代码组织

- 将每个 API 的实现放在独立的部分
- 使用注释清晰地标记每个方法
- 保持一致的命名约定

### 2. 文档

- 为所有公共方法提供文档注释
- 包含使用示例
- 说明参数和返回值类型

### 3. 错误处理

- 提供有意义的错误消息
- 包含错误代码和建议
- 记录常见错误情况

### 4. 性能

- 避免不必要的内存分配
- 重用对象和缓冲区
- 优化网络请求

### 5. 兼容性

- 测试不同的浏览器
- 处理 WASM 不支持的情况
- 提供降级方案

## 下一步

1. **完成剩余 API 实现** - 按照优先级实现所有 API
2. **添加更多功能** - 流式支持、批量操作等
3. **性能优化** - 减少包大小、提高执行效率
4. **增强错误处理** - 更详细的错误信息和恢复机制
5. **扩展测试覆盖** - 更全面的测试套件

## 参考资源

- [WebAssembly 官方文档](https://webassembly.org/)
- [wasm-bindgen 指南](https://rustwasm.github.io/wasm-bindgen/)
- [serde-wasm-bindgen 文档](https://docs.rs/serde-wasm-bindgen/)
- [Rust 异步编程](https://rust-lang.github.io/async-book/)

---

_此文档将随着实现进度持续更新。_
