# Trae Rules

## Common Rules

AI responses should always use Chinese, including: Chat, Inline Chat, Builder

## Rust Rules

### Standard Library

Prefer standard library lock types, avoid using tokio locks when possible

```rust
use std::sync::{Mutex, RwLock};
```

### Date, Time, DateTime with chrono

Use `OffsetDateTime` from `ultimate_common` library for date-time types, do not use `chrono::Utc` library

```rust
use ultimate_common::time::{OffsetDateTime, now_offset, now_utc};

let now = now_offset(); // FixedOffset date-time
let now_utc = now_utc(); // Utc date-time
```

Use `now_offset` by default to generate `OffsetDateTime` date-time
