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

### 示例 2

```markdown
任务： 为 @hetumind/hetumind-core/src/workflow/*.rs 中的所有 struct 添加构造函数和修改方法

## 构造函数规则

生成 pub fn new(...) -> Self 函数，规则如下：
- 参数： 非Option、非bool、非容器类型（Vec/HashMap/HashSet）的字段
- 限制： 最多5个参数，超过则不生成 new 但生成修改方法
- 初始化： 其他字段使用 Default::default()
- 报告： 执行后打印参数超过5个的 struct 名称

## 修改方法规则

### 基本类型方法

// T 或 Option<T> 类型
pub fn with_field_name(mut self, field_name: impl Into<String>) -> Self
// Option<T> 参数类型不包裹 Option

// 数值、bool、enum 类型直接使用原始类型，不用 impl Into<T>

### 容器类型方法

#### Vec 类型：
pub fn with_options<I, V>(mut self, options: I) -> Self
where
  I: IntoIterator<Item = V>,
  V: Into<Box<NodeProperty>>,
{
  self.options = Some(options.into_iter().map(|v| v.into()).collect());
  self
}

pub fn add_option(mut self, option: impl Into<Box<NodeProperty>>) -> Self {
  self.options.get_or_insert_with(Vec::new).push(option.into());
  self
}

#### HashMap 类型：
pub fn with_routing<I, K, V>(mut self, routing: I) -> Self
where
  I: IntoIterator<Item = (K, V)>,
  K: Into<String>,
  V: Into<JsonValue>,
{
  self.routing = Some(routing.into_iter().map(|(k, v)| (k.into(), v.into())).collect());
  self
}

pub fn add_routing(mut self, key: impl Into<String>, value: impl Into<JsonValue>) -> Self {
  self.routing.get_or_insert_with(HashMap::default).insert(key.into(), value.into());
  self
}

## 特殊处理

- 包裹类型： Box<T>、Arc<T> 等在方法参数中保持包裹类型
- 已有方法： 检查避免重复生成已存在的 new、with_xxx、add_xxx 方法
- newtype 类型的 struct 不需要添加修改方法
```

### 提示词 3

```markdown
任务： 重构 @hetumind/hetumind-nodes/src/ 中所有使用 TypedBuilder 的代码

重构规则

1. 识别目标代码：查找所有使用 ::builder() 方法的代码
2. 替换构造方式：
  - 将 Xxxx::builder().field(value).build() 替换为 Xxxx::new(...).with_field(value)
  - 使用 new() 函数的必需参数
  - 保留所有 with_xxx() 和 add_xxx() 调用
3. 保持功能不变：确保重构后的代码行为完全一致

执行步骤

1. 扫描 hetumind-nodes/src/ 目录，识别所有使用 TypedBuilder 的文件
2. 分析每个使用场景，确定对应的 new() 函数参数
3. 逐个文件进行重构替换
4. 编译验证确保无错误

输出要求

- 列出所有修改的文件和位置
- 报告编译结果
```
