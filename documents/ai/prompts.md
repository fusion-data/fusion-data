# 常用提示词

## 编程类

开头的敬语：

你是一名资深的软件架构师，拥有丰富的软件架构设计和开发经验，擅长架构设计、领域驱动建模，熟悉 Rust、Typescript
等编程语言。让我们来继续完善

### 提示词增强

- 检查上面的方案，请进行必要的补充和优化。然后输出是否还有优化建议和需要澄清的问题？

- 只更新设计文档，不要进行任何代码实现。若有疑问或更好的方案，请输出疑问或方案让我审核！

### Coding

- 复用现有的数据结构（hetumind-core 及 hetumind-nodes 两个项目）
- 复用现有的错误处理模式
- 遵循当前项目 Rust 编程最佳实践，对参数使用 snake_case 风格命名
- 注意 Arc 的使用以支持并发访问
- 定义结构化的数据类型优先（如：`pub struct Config`）；若需要直接使用 JSON 对象，请使用 `serde_json::json!` 宏进行构造
- 使用 `serde` 的 struct, enum，当手动构建 JSON 字符串时注意序列化方式。比如：`#[serde(rename_all = "snake_case")]` 需要使用 snake_case 风格

### 实现测试

- 按照数据结构（struct, enum）生成格式正确的测试数据，使用 `serde` 时注意序列化风格：`snake_case`、`camelCase`、`UPPER_CASE`、`CamelCase`、`lowercase` 等，或是否有添加 `#[repr(i32)]` 序列化为 i32 数值类型）
- 添加必要的日志输出（使用 log crate）

## 示例

### 示例 1

```markdown
你是一名经验丰富的资深软件开发工程师，擅长架构设计、领域驱动建模，熟悉 Rust、Typescript 等编程语言。仔细阅读 @documents/oauth.md 文档设计，规划任务实现方案目标，完成相关编码工作。

建议任务顺序：

1. 更新 SQL DDL 定义
2. 更新 jieyuan 相关项目
3. 更新 hetumind 相关项目

注意事项：

- 复用现有的数据结构
- 复用现有的错误处理模式
- 遵循当前项目 Rust 编程最佳实践，对参数使用 snake_case 风格命名
- 注意 Arc 的使用以支持并发访问
- 使用 `serde` 的 struct, enum，当手动构建 JSON 字符串时注意序列化方式。比如：`#[serde(rename_all = "snake_case")]` 需要使用 snake_case 风格
- 完成 SQL DDL 文件更新后暂停，由我手动执行 SQL 语句后再通知你继续执行后续任务
- 若有任何疑问或需要澄清的地方，请输出疑问或方案让我审核！
```
