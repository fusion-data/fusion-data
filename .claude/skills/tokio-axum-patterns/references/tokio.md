# Tokio Async Runtime Patterns

基于社区最佳实践和 [Tokio 官方指南](https://tokio.rs/tokio/tutorial)。

## Runtime Setup

### Multi-threaded Runtime (默认)
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 默认：多线程运行时
    // 工作线程数 = CPU 核心数
}

// 等价于
fn main() -> Result<()> {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            // ...
        })
}
```

### Current-thread Runtime
```rust
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // 单线程运行时
    // 适用于 I/O 密集型、低延迟场景
}
```

### 自定义配置
```rust
use tokio::runtime::Builder;

let runtime = Builder::new_multi_thread()
    .worker_threads(4)
    .max_blocking_threads(16)
    .thread_name("my-worker")
    .enable_all()
    .build()?;

runtime.block_on(async {
    // ...
});
```

## Task Spawning

### tokio::spawn
```rust
use tokio::task::spawn;

// 启动新任务
let handle = spawn(async {
    // 异步工作
    42
});

// 等待结果
let result = handle.await??;  // Result<Result<T>, JoinError>
```

### spawn_blocking
```rust
use tokio::task::spawn_blocking;

// 在阻塞线程池中执行
let result = spawn_blocking(|| {
    // CPU 密集型或阻塞操作
    std::fs::read_to_string("large_file.txt")
}).await??;
```

### 任务取消
```rust
use tokio::task::spawn;
use tokio_util::sync::CancellationToken;

let token = CancellationToken::new();
let child_token = token.child_token();

let handle = spawn(async move {
    loop {
        if child_token.is_cancelled() {
            break;
        }
        // 工作
    }
});

// 取消
token.cancel();
```

## Channels

### mpsc (多生产者，单消费者)
```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel(100);

// 生产者
spawn(async move {
    for i in 0..10 {
        tx.send(i).await.unwrap();
    }
});

// 消费者
while let Some(value) = rx.recv().await {
    println!("Received: {}", value);
}
```

### oneshot (单次发送)
```rust
use tokio::sync::oneshot;

let (tx, rx) = oneshot::channel();

spawn(async move {
    tx.send(42).unwrap();
});

let result = rx.await.unwrap();
```

### broadcast (广播)
```rust
use tokio::sync::broadcast;

let (tx, mut rx1) = broadcast::channel(16);
let mut rx2 = tx.subscribe();

tx.send("hello").unwrap();

println!("rx1: {}", rx1.recv().await.unwrap());
println!("rx2: {}", rx2.recv().await.unwrap());
```

### watch (监视)
```rust
use tokio::sync::watch;

let (tx, mut rx) = watch::channel(0);

spawn(async move {
    for i in 1..=10 {
        tx.send_replace(i);
        sleep(Duration::from_millis(100)).await;
    }
});

while rx.changed().await.is_ok() {
    println!("Changed: {}", *rx.borrow());
}
```

## Synchronization

### Mutex
```rust
use tokio::sync::Mutex;
use std::sync::Arc;

let data = Arc::new(Mutex::new(vec![]));

let data_clone = Arc::clone(&data);
spawn(async move {
    let mut guard = data_clone.lock().await;
    guard.push(42);
});

let guard = data.lock().await;
println!("{:?}", *guard);
```

### RwLock
```rust
use tokio::sync::RwLock;

let data = RwLock::new(vec![]);

// 读锁
let read_guard = data.read().await;
println!("{:?}", *read_guard);
drop(read_guard);

// 写锁
let mut write_guard = data.write().await;
write_guard.push(42);
```

### Semaphore
```rust
use tokio::sync::Semaphore;

let semaphore = Semaphore::new(3);  // 最多 3 个并发

async fn limited_work(semaphore: &Semaphore) {
    let permit = semaphore.acquire().await.unwrap();

    // 工作（最多 3 个并发）
    sleep(Duration::from_secs(1)).await;

    drop(permit);  // 释放
}
```

## select!

```rust
use tokio::{select, signal};

async fn run() {
    let (tx, mut rx) = mpsc::channel(100);

    loop {
        select! {
            // 接收消息
            Some(msg) = rx.recv() => {
                println!("Received: {}", msg);
            }

            // 超时
            _ = sleep(Duration::from_secs(5)) => {
                println!("Timeout");
            }

            // Ctrl+C
            _ = signal::ctrl_c() => {
                println!("Shutdown");
                break;
            }
        }
    }
}
```

### select! with biased
```rust
select! {
    biased;  // 按顺序检查，不随机

    _ = rx1.recv() => { }
    _ = rx2.recv() => { }
}
```

## Time

### sleep
```rust
use tokio::time::{sleep, Duration};

sleep(Duration::from_secs(1)).await;
```

### timeout
```rust
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(5),
    async_operation()
).await;

match result {
    Ok(value) => println!("Success: {:?}", value),
    Err(_) => println!("Timeout"),
}
```

### interval
```rust
use tokio::time::{interval, Duration};

let mut ticker = interval(Duration::from_secs(1));

loop {
    ticker.tick().await;
    println!("Tick!");
}
```

## Best Practices

1. **避免阻塞**: 不要在 async 中使用阻塞操作，使用 `spawn_blocking`
2. **合理分片**: 使用 `chunks` 或 `buffer` 批量处理
3. **背压控制**: 使用有界 channel 防止内存溢出
4. **优雅关闭**: 使用 `CancellationToken` 或 `select!` 处理关闭信号
5. **错误处理**: 使用 `JoinHandle` 的 `await` 处理任务 panic

## References

- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Tokio Docs](https://docs.rs/tokio)
- [Async Rust Patterns](https://rust-lang.github.io/async-book/)
