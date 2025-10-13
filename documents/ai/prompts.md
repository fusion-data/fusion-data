# 常用提示词

## 编程类

### 提示词增强

* 检查上面的方案，请进行必要的补充和优化。然后输出是否还有优化建议和需要澄清的问题？
* 只更新设计文档，不要进行任何代码实现。若有疑问或更好的方案，请输出疑问或方案让我审核！

### 实现测试

* 按照数据结构（struct, enum）生成格式正确的测试数据，使用 serde 时注意序列化风格：snake_case、camelCase、UPPER_CASE、CamelCase、lowercase、是否有添加 repr 序列化为 i32）
* 添加必要的日志输出（使用 log crate）

## 专项： n8n

### 分析 n8n node

分析 **Email Trigger (IMAP) ** node 的技术实现，若有必要的话使用 mermaid 绘制架构或数据流转图。将分析后的结果使用 Markdown 格式写入 @specs/nodes/core/ 目录中。

* 只需要分析此 node 最新版本的功能，不用考虑兼容历史版本
* 定位并分析节点的核心实现文件和相关配置
* 分析节点的 execute 方法实现，处理数据的核心算
* 分析节点的 Input, Output 数据集，数据处理机制和性能特征
* 分析节点的错误处理机制
* 绘制节点的数据流转图
* 绘制类继承/实现关系图
* 绘制工作流执行序列图
* 识别设计模式和架构决策
* 分析使用场景和最佳实践
* 与同类节点进行对比分析
* 生成完整的技术分析文档

请优化上述提示词，并输出更新后的提示词

### 基于XXX技术分析文档，实现当前项目功能

<这里粘贴 XXX技术分析文档>
----
结合以上对 n8n 的 **Edit Image** node 技术实现分析，实现符合 hetumind 项目风格的 Rust 版本 EditImageNode node。要求：

- 复用已有的错误处理模式
- 复用现有的数据结构（hetumind-core 及 hetumind-nodes 两个项目）
- 遵循当前项目 Rust 编程最佳实践，对参数使用 snake_case 风格命名
- 注意 Arc 的使用以支持并发访问
- 定义结构化的数据类型优先（如：`pub struct Config`）；若需要直接使用 JSON 对象，请使用 `serde_json::json!` 宏进行构造
- 当使用到第 3 方 crate 时，应先学习所依赖库的正确使用方法，避免猜测使用方法
- 参考 @/hetumind/hetumind-nodes/src/core/if_node, @/hetumind/hetumind-nodes/src/core/merge 的实现方式进行实现，包括代码风格和文件结构等
- 在 @/hetumind/hetumind-nodes/src/core 中实现 hetumind 版本的 node
- 编写单元测试验证 node 的功能实现是否完整、正确，使用 serde 宏注解的数据结构时，注意序列化风格，如： snake_case, camelCase, CamelCase, UPPER_CASE 等。使用 `cargo test` 命令进行验证。

----

任务：深入分析 n8n Manual Trigger node 的完整技术实现

具体要求：

1. 核心机制分析：
- 分析 ManualTrigger 代码中的 execute 方法实现逻辑
- 分析示例数据生成机制
2. 数据结构分析：
- 分析 NodeExecutionData 接口的实现
3. 架构图表设计：
- 创建 Mermaid sequence diagram 展示错误触发流程
- 创建 Mermaid flowchart 展示组件间的交互关系
- 标注关键数据结构在流程中的传递
4. 文档输出要求：
- 在 /specs/nodes/triggers/ 目录下创建 manual-trigger-analysis.md
- 包含代码片段分析、架构图表、数据结构说明
- 提供完整的错误处理机制技术文档

输出格式：
- 使用标准的 Markdown 格式
- 包含适当的代码高亮
- Mermaid 图表要有清晰的注释
- 技术术语要准确，必要时提供解释
