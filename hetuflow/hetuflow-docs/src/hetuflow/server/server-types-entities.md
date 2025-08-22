# hetuflow-server 数据类型和实体定义

## 数据模型实现

### 数据模型架构

基于 **modelsql** ORM 的分层数据模型设计，采用以下架构模式：

- **Entity 层**: 使用 Rust 结构体定义数据实体，支持字段扩展和过滤器
- **BMC 层**: Database Basic Model Controller，提供类型安全的 CRUD 操作抽象
- **ModelManager**: 统一的数据库连接和操作管理
- **Service 层**: 业务逻辑层，使用 BMC 进行数据库操作，错误转换为 DataError

该架构确保了数据库访问的类型安全、错误处理的一致性和代码的可维护性。
