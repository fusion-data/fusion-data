# Fusion Data 数据平台

- [商业智能](./bi.md)
- [数据治理](./governance.md)
- [调度系统](./scheduler.md)
- [数据集成](./integration.md)
- [身份认证和授权](./iam.md)

## 整体架构

- 整体架构:
  - 使用微服务架构,将5个子系统解耦
  - 采用API网关统一管理服务接口
  - 使用消息队列实现系统间异步通信
- 技术选择:
  - Rust: 性能关键的组件,如数据处理引擎、调度系统核心
  - Python: 快速开发和数据分析相关功能
  - 数据库: PostgreSQL (关系型)和 ClickHouse (列式存储)
  - 消息队列: Kafka
  - 容器化: Docker 和 Kubernetes
- 子系统设计:
  - 商业智能 (BI):
    - 使用Python构建数据可视化和报表生成功能
    - 集成开源BI工具如Superset
    - 使用Rust开发数据质量检测引擎
    - Python实现元数据管理和数据血缘分析
  - 调度系统:
    - Rust实现核心调度逻辑,确保高性能
    - 集成Apache Airflow作为工作流管理工具
  - 数据集成:
    - Rust开发高性能ETL引擎
    - Python编写各种数据源连接器
  - 身份认证和授权:
    - 使用Rust实现核心认证逻辑
    - 集成开源IAM解决方案如Keycloak
  - 数据存储:
    - 使用数据湖架构,结合对象存储和分布式文件系统
    - 采用Delta Lake等技术实现ACID事务
  - 可扩展性:
    - 设计插件系统,允许使用Python快速开发新功能
    - 使用Kubernetes实现服务的弹性伸缩
  - 监控和日志:
    - 集成Prometheus和Grafana进行系统监控
    - 使用ELK栈进行日志管理

这个设计结合了Rust的高性能和Python的开发效率,可以构建一个强大而灵活的数据平台。根据具体需求,您可以进一步细化每个子系统的设计。
