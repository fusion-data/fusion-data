# History of Prompts

## 重构：使用 hetus 聚合 crate 替代各个单独的 hetu-xxx 导入

项目使用 fusion 库时，只需要导入单一的 hetus 库来替代各个单独的 hetu-xxx 导入。比如：

```rust
use hetus::core::DataError; // ✅
use hetu_core::DataResult;  // ❌
```

请先分析 @crates 中各项目，然后设计迁移方案。

已知问题：

- hetu-core-macros 生成代码时，应该将 `::hetu_core::` 修改为 `::hetus::core::`
