# Trae 规则

1. AI 回复始终使用中文，包括：Chat, Inline Chat, Builder

## Rust 规则

1. 优先使用标准库的锁类型，尽量避免使用 tokio 的锁
   ```rust
   use std::sync::{Mutex, RwLock};
   ```
2. 日期时间类型使用 `ultimate_common` 库的 `OffsetDateTime`，不要使用 `chrono::Utc` 库
   ```rust
   use ultimate_common::time::{OffsetDateTime, now_offset, now_utc};

   let now = now_offset(); // FixedOffset 日期时间
   let now_utc = now_utf(); // Utc 日期时间
   ```
   默认使用 `now_offset` 生成 `OffsetDateTime` 日期时间

