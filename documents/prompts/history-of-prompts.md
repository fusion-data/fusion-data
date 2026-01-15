# History of Prompts

## 重构：使用 fusions 聚合 crate 替代各个单独的 fusion-xxx 导入

项目使用 fusion 库时，只需要导入单一的 fusions 库来替代各个单独的 fusion-xxx 导入。比如：

```rust
use fusions::core::DataError; // ✅
use fusion_core::DataResult;  // ❌
```

请先分析 @crates/fusions 中各项目，然后设计迁移方案。

已知问题：
- fusion-core-macros 生成代码时，应该将 `::fusion_core::` 修改为 `::fusions::core::`
