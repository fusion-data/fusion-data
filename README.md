# fusion-data

## 项目介绍

fusion-data 是一个基于 Rust 语言开发的 **融合数据** 平台，当前正在进行中的项目有：

- [modelsql](crates/libs/modelsql/): 基于 [sea-query](https://github.com/SeaQL/sea-query/) 开发的数据库访问层
- [hetuflow](hetuflow/): **河图流动** 分布式作业调度系统
- [hetumind](hetumind/): **河图智思** AI Agent/Flow 平台

## 开发环境

开发环境详细配置请见：[development-zh](./documents/development-zh.md) 说明。

## 🛠 许可证

- **开源版本**：采用 [Apache License 2.0](./LICENSE.txt) 许可证
- **商业版本**：如需在闭源或专有应用中使用，可获取[商业许可证](./LICENSE-COMMERCIAL.txt)

~~如需避免 Apache-2.0 的限制，请[购买商业许可证](https://hetumind.com/pricing)。~~

~~📩 联系方式：[sales@hetumind.com](mailto:sales@hetumind.com)~~

## 致谢

本项目从以下优秀项目中汲取了大量灵感和代码：

- [modql](https://crates.io/crates/modql)
- [spring](https://crates.io/crates/spring)
- [Hash Wheel Timer](https://crates.io/crates/hierarchical_hash_wheel_timer)
- 等等，感谢所有为开源社区做出贡献的项目和个人。详细可见 [Cargo.toml](Cargo.toml) 配置文件中的依赖部分
