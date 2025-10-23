# fusion-data

## 项目介绍

fusion-data 是一个基于 Rust 生态开发的 **融合数据** 平台，当前版本 0.1.0，采用 Rust 2024 Edition 构建。平台包含以下核心项目：

### 核心库 (crates/fusions/)

- **fusion-core**: 应用程序框架，提供组件系统、配置管理、异步运行时等核心功能
- **fusion-web**: 基于 Axum 的 HTTP 服务器框架
- **fusion-db**: 数据库访问层
- **fusion-grpc**: gRPC 工具库
- **fusion-security**: 安全组件
- **fusion-common**: 共享工具库
- **fusion-ai**: AI 集成工具
- **fusion-core-macros**: 核心派生宏
- **fusionsql**: 基于 [sea-query](https://github.com/SeaQL/sea-query/) 开发的数据库 ORM
- **fusionsql-core**: 核心类型、特性和数据库抽象
- **fusionsql-macros**: 模型定义派生宏
- **fusions**: 所有 fusion 库的元包

### 应用项目

- **[hetuflow](hetuflow/)**: **河图流动** 分布式任务调度系统

  - `hetuflow-core`: 共享模型、协议和作业定义
  - `hetuflow-server`: 中央调度服务器，提供 gRPC/Web API
  - `hetuflow-agent`: 分布式执行代理，带任务运行器
  - `hetuflow-test`: 集成测试和测试工具
  - `hetuflow-web`: Web 界面和仪表板
  - `hetuflow-docs`: 文档和示例

- **[hetumind](hetumind/)**: **河图智思** 和流程编排、AI Agent 平台，集成 LLM 功能

  - `hetumind-core`: 核心 AI 功能和代理编排
  - `hetumind-nodes`: 节点执行框架，提供全面的工作流节点
  - `hetumind-context`: 上下文管理和状态持久化
  - `hetumind-studio`: Web 工作室界面，支持多租户代理设计
  - `hetumind-cli`: 代理管理命令行工具
  - `hetumind-docs`: 文档和教程

- **[jieyuan](jieyuan/)**: **界垣** 访问控制和身份验证工具，采用模块化 IAM 系统
  - `jieyuan-core`: 核心访问控制模型、OAuth 身份验证、策略引擎和 IAM 资源映射
  - `jieyuan-server`: 中央化 IAM 服务器
  - **IAM 资源映射**: 通过托管 API 路径映射实现零配置权限控制
  - **模块化访问控制**: 将身份验证和授权分离到专门模块
  - **OAuth 模块**: 独立的 OAuth 2.0 + PKCE 身份验证服务
  - **策略引擎**: 基于角色的访问控制，支持细粒度权限
  - **远程授权**: 集中化策略评估，支持自动租户注入

## 技术特性

### 架构特点

- **多租户架构**: 完整的租户数据隔离和访问控制
- **异步优先**: 基于 Tokio 的全异步架构
- **类型安全**: 利用 Rust 类型系统提供编译时保证
- **零安全代码**: 工作空间 lint 强制执行零安全代码策略
- **高性能**: 使用 ahash 替代标准库哈希集合，提供 2-3x 性能提升
- **模块化设计**: 清晰的关注点分离和定义良好的 API

### 数据库支持

- **PostgreSQL**: 主数据库，支持 pgvector 向量扩展
- **SQLite**: 轻量级数据库支持
- **FusionSQL ORM**: 基于 sea-query 和 SQLx 的类型安全 ORM
- **向量数据库**: 支持 pgvector 进行向量存储和检索

### AI/LLM 集成

- **多模型支持**: 集成多种 LLM 提供商
- **Agent 工作流**: 可视化代理设计和执行
- **节点执行框架**: 丰富的工作流节点类型
- **上下文管理**: 持久化上下文和状态管理

### 安全特性

- **OAuth 2.0 + PKCE**: 现代化身份验证流程
- **JWT 令牌**: 安全的令牌验证和刷新
- **细粒度权限**: 基于策略的访问控制
- **多租户隔离**: 自动租户上下文注入
- **审计日志**: 完整的授权决策审计

## 开发环境

### 系统要求

- **Rust**: 版本 ≥ 1.90 (通过 rustup 管理，推荐中国用户使用 rsproxy)
- **Node.js**: 版本 ≥ 22 (通过 nvm 管理)
- **pnpm**: Node.js 依赖包管理器
- **Docker & Docker Compose**: 数据库和服务管理
- **Git**: 版本控制

### 快速开始

```bash
# 克隆项目
git clone https://github.com/fusion-data/fusion-data.git
cd fusion-data

# 启动数据库服务
docker-compose up -d

# 构建项目
cargo build

# 运行测试
cargo test

# 启动服务
cargo run --bin hetuflow-server
cargo run --bin hetumind-studio
cargo run --bin jieyuan-server
```

### 常用命令

```bash
# 基础构建和检查
cargo check
cargo build

# 代码格式化和检查
cargo fmt
cargo clippy --workspace --all-targets --all-features --no-deps -- -D warnings

# 运行测试
cargo test

# 特定测试场景
cargo test -p <crate-name> --lib              # 仅库和文档测试
cargo test -p <crate-name> --bins             # 仅二进制测试
cargo test -p <crate-name> --lib --bins       # 库和二进制测试（无集成）

# 特定配置文件构建
cargo build --release                         # 优化发布构建
cargo build --profile bench                   # 基准测试配置

# 更新依赖
cargo update
cargo tree                                   # 查看依赖树
```

### 数据库服务

使用 docker 启动开发用 PostgreSQL

```bash
docker-compose up -d      # 启动服务
docker-compose ps         # 检查状态
docker-compose logs -f    # 跟踪日志
docker-compose down       # 停止服务
docker-compose down -v    # 停止并清理卷

# 特定服务命令
docker-compose restart postgres
```

## 项目结构

```
fusion-data/
├── crates/fusions/          # 核心库
│   ├── fusion-core/         # 应用程序框架
│   ├── fusion-web/          # Web 框架
│   ├── fusion-db/           # 数据库访问层
│   ├── fusionsql/           # ORM 实现
│   └── ...
├── hetuflow/                # 分布式任务调度
│   ├── hetuflow-core/       # 核心模型和协议
│   ├── hetuflow-server/     # 调度服务器
│   ├── hetuflow-agent/      # 执行代理
│   └── hetuflow-web/        # Web 程序
├── hetumind/                # AI Agent 平台
│   ├── hetumind-core/       # 核心 AI 功能
│   ├── hetumind-studio/     # Web 工作室
│   ├── hetumind-nodes/      # 工作流节点
│   ├── hetumind-context/    # 上下文管理
│   └── hetumind-web/        # Web 程序
├── jieyuan/                 # 访问控制
│   ├── jieyuan-core/        # 核心 IAM 模型
│   └── jieyuan-server/      # IAM 服务器
├── packages/                # TypeScript/JavaScript 包
└── documents/               # 项目文档、技术资料
```

## 配置管理

项目使用 TOML 配置文件，支持环境变量覆盖：

- `FUSION_CONFIG_FILE`: 指定配置文件路径
- 默认配置位于各项目的 `resources/` 目录
- 支持多环境配置和特性标志

## 文档

- **开发环境配置**: [development-zh](./documents/development-zh.md)
- **IAM 技术规范**: [iam.md](./documents/iam.md)
- **Hetumind-Jieyuan 集成**: [hetumind-jieyuan-integration.md](./documents/hetumind-jieyuan-integration.md)
- **OAuth 文档**: [oauth.md](./documents/oauth.md)

## 致谢

本项目从以下优秀项目中汲取了大量灵感和代码：

- [modql](https://crates.io/crates/modql)
- [spring](https://crates.io/crates/spring)
- [Hash Wheel Timer](https://crates.io/crates/hierarchical_hash_wheel_timer)
- 等等，感谢所有为开源社区做出贡献的项目和个人。详细可见 [Cargo.toml](Cargo.toml) 配置文件中的依赖部分

## 许可证

本项目许可见 [LICENSE](LICENSE)。
