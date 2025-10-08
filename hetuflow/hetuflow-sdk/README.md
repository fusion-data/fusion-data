# Hetuflow SDK

[![Crates.io](https://img.shields.io/crates/v/hetuflow-sdk.svg)](https://crates.io/crates/hetuflow-sdk)
[![Documentation](https://docs.rs/hetuflow-sdk/badge.svg)](https://docs.rs/hetuflow-sdk)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

Rust SDK for Hetuflow distributed task scheduling and workflow orchestration system.

## Features

- **Native Support**: Full support for native Rust applications with Tokio runtime
- **WASM Support**: Compile to WebAssembly for browser and Node.js environments
- **Type Safety**: Full type safety using models from hetuflow-core
- **Async/Await**: Async first design with both native and WASM support
- **Error Handling**: Comprehensive error handling with detailed error messages
- **Retry Logic**: Built-in retry mechanisms for network failures
- **OpenAPI Compatible**: Generated from the official Hetuflow server API

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
hetuflow-sdk = "0.1.0"
```

For WASM support:

```toml
[dependencies]
hetuflow-sdk = { version = "0.1.0", features = ["with-wasm"] }
```

## Quick Start

### Native Rust

```rust
use hetuflow_sdk::HetuflowClient;
use fusion_common::page::Page;
use hetuflow_core::models::{JobForCreate, TaskConfig};
use hetuflow_core::types::JobStatus;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = HetuflowClient::new("http://localhost:8080")?;

    // List agents with pagination
    let mut query = hetuflow_core::models::AgentForQuery::default();
    query.page = Page { page: Some(1), limit: Some(10), offset: Some(0), order_bys: None };

    let agents = client.agents().query(query).await?;
    println!("Found {} agents", agents.page.total);

    // Create a job
    let job = JobForCreate {
        name: "example-job".to_string(),
        namespace_id: Some("default".to_string()),
        config: Some(TaskConfig {
            cmd: hetuflow_core::models::ExecuteCommand::Bash,
            args: vec!["echo".to_string(), "Hello, Hetuflow!".to_string()],
            timeout: 300,
            max_retries: 3,
            retry_interval: 60,
            capture_output: true,
            max_output_size: 1024 * 1024,
            ..Default::default()
        }),
        status: Some(JobStatus::Enabled),
        ..Default::default()
    };

    let result = client.jobs().create(job).await?;
    println!("Created job with ID: {}", result.id);

    Ok(())
}
```

### WASM (Browser)

```rust
use hetuflow_sdk::HetuflowClient;
use wasm_bindgen_futures::spawn_local;

fn main() {
    let client = HetuflowClient::new("http://localhost:8080")?;

    spawn_local(async move {
        match client.system().health().await {
            Ok(health) => web_sys::console::log_1(&format!("Health: {:?}", health).into()),
            Err(e) => web_sys::console::error_1(&format!("Error: {}", e).into()),
        }
    });
}
```

## API Coverage

The SDK provides full coverage of the Hetuflow API with modern pagination and filtering:

- **Agents API**: Manage and monitor agents

  - `query(agent_query)` - Query agents with filtering and pagination
  - `create(agent)` - Create new agent
  - `update(id, update)` - Update existing agent
  - `delete(id)` - Delete agent

- **Jobs API**: Create, update, and manage jobs

  - `query(job_query)` - Query jobs with filtering and pagination
  - `create(job)` - Create new job
  - `update(id, update)` - Update existing job
  - `delete(id)` - Delete job

- **Tasks API**: Query and control task execution

  - `query(task_query)` - Query tasks with filtering and pagination
  - `create(task)` - Create new task
  - `update(id, update)` - Update existing task
  - `delete(id)` - Delete task

- **Schedules API**: Configure task scheduling

  - `query(schedule_query)` - Query schedules with filtering and pagination
  - `create(schedule)` - Create new schedule
  - `update(id, update)` - Update existing schedule
  - `delete(id)` - Delete schedule
  - `get_schedulable()` - Get schedulable schedules

- **Task Instances API**: Monitor task execution instances

  - `query(instance_query)` - Query task instances with filtering and pagination
  - `create(instance)` - Create new task instance
  - `update(id, update)` - Update existing task instance
  - `delete(id)` - Delete task instance

- **Servers API**: Manage server instances

  - `query(server_query)` - Query servers with filtering and pagination
  - `get(id)` - Get specific server
  - `update(id, update)` - Update existing server
  - `delete(id)` - Delete server

- **System API**: Health checks and metrics

  - `health()` - Get system health status
  - `metrics()` - Get system metrics

- **Gateway API**: WebSocket connections and commands

  - `connect()` - Establish WebSocket connection
  - `send_command()` - Send commands via WebSocket

- **Auth API**: Token generation and authentication
  - `generate_token()` - Generate authentication token
  - `validate_token()` - Validate existing token

### Query Pattern

All list operations use a consistent query pattern:

```rust
use hetuflow_core::models::AgentForQuery;
use fusion_common::page::Page;

// Create query with filtering and pagination
let mut query = AgentForQuery::default();
query.filter.status = Some(hetuflow_core::types::AgentStatus::Online);
query.page = Page {
    page: Some(1),
    limit: Some(20),
    offset: Some(0),
    order_bys: None
};

// Execute query
let result = client.agents().query(query).await?;
println!("Found {} agents", result.page.total);
for agent in result.result {
    println!("Agent: {}", agent.id);
}
```

## Configuration

```rust
use hetuflow_sdk::{HetuflowClient, Config};
use std::time::Duration;

let config = Config::new("http://localhost:8080")
    .with_auth_token("your-token-here")
    .with_timeout(Duration::from_secs(30))
    .with_retry_config(3, Duration::from_millis(1000), Duration::from_secs(30))
    .with_user_agent("my-app/1.0")
    .with_header("X-Custom-Header", "value");

let client = HetuflowClient::with_config(config)?;
```

## Examples

See the `examples/` directory for complete examples:

- `examples/usage.rs` - Complete usage example showing all SDK features
- `examples/wasm/web_example.js` - Browser WASM usage

### Quick Example

```rust
use hetuflow_sdk::HetuflowClient;
use hetuflow_core::models::{AgentForQuery, TaskForQuery};
use hetuflow_core::types::TaskStatus;
use fusion_common::page::Page;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = HetuflowClient::new("http://localhost:8080")?;

    // Get system health
    match client.system().health().await {
        Ok(health) => println!("System healthy: {:?}", health),
        Err(e) => println!("Health check failed: {}", e),
    }

    // List agents
    let mut agent_query = AgentForQuery::default();
    agent_query.page = Page { page: Some(1), limit: Some(10), offset: Some(0), order_bys: None };

    match client.agents().query(agent_query).await {
        Ok(result) => println!("Found {} agents", result.page.total),
        Err(e) => println!("Failed to list agents: {}", e),
    }

    // Find failed tasks
    let mut task_query = TaskForQuery::default();
    task_query.page = Page { page: Some(1), limit: Some(50), offset: Some(0), order_bys: None };

    match client.tasks().query(task_query).await {
        Ok(result) => {
            let failed_tasks: Vec<_> = result.result
                .into_iter()
                .filter(|task| task.status == TaskStatus::Failed)
                .collect();
            println!("Found {} failed tasks", failed_tasks.len());
        }
        Err(e) => println!("Failed to query tasks: {}", e),
    }

    Ok(())
}
```

## Building for WASM

```bash
# Build the WASM package
wasm-pack build --target web --out-dir examples/wasm/pkg

# Serve the example
cd examples/wasm
python3 -m http.server 8080
```

Then open `http://localhost:8080/web_example.html` in your browser.

## Error Handling

The SDK provides comprehensive error handling:

```rust
use hetuflow_core::models::AgentForQuery;
use fusion_common::page::Page;

// Query specific agent
let mut query = AgentForQuery::default();
query.filter.id = Some("agent-id".to_string());

match client.agents().query(query).await {
    Ok(result) => {
        if let Some(agent) = result.result.into_iter().next() {
            println!("Agent: {} ({})", agent.id, agent.address);
        } else {
            println!("Agent not found");
        }
    }
    Err(e) => {
        if e.is_client_error() {
            println!("Client error: {}", e);
        } else if e.is_server_error() {
            println!("Server error: {}", e);
        } else if e.is_retryable() {
            println!("Retryable error: {}", e);
        } else {
            println!("Other error: {}", e);
        }
    }
}
```

## Data Models

The SDK uses data models from `hetuflow-core` and pagination types from `fusionsql_core`:

### Models (from `hetuflow_core`):

- `SchedAgent`, `AgentForCreate`, `AgentForUpdate`, `AgentForQuery`
- `SchedJob`, `JobForCreate`, `JobForUpdate`, `JobForQuery`
- `SchedTask`, `TaskForCreate`, `TaskForUpdate`, `TaskForQuery`
- `SchedSchedule`, `ScheduleForCreate`, `ScheduleForUpdate`, `ScheduleForQuery`
- `SchedTaskInstance`, `TaskInstanceForCreate`, `TaskInstanceForUpdate`, `TaskInstanceForQuery`
- And many more...

### Types (from `hetuflow_core`):

- `AgentStatus`, `JobStatus`, `TaskStatus`, `ScheduleStatus`
- `ExecuteCommand`, `TaskConfig`, `AgentCapabilities`
- And many more...

### Pagination (from `fusionsql_core`):

- `PageResult<T>` - Paginated response with `{ page: Paged, result: Vec<T> }`
- `Page` - Pagination query with `{ page, limit, offset, order_bys }`
- `Paged` - Contains `total` count of items

### Usage Example:

```rust
use hetuflow_sdk::HetuflowClient;
use hetuflow_core::models::{AgentForQuery, JobForCreate, TaskConfig};
use hetuflow_core::types::{JobStatus, TaskStatus};
use fusion_common::page::Page;

// Query with pagination
let mut query = AgentForQuery::default();
query.page = Page {
    page: Some(1),
    limit: Some(10),
    offset: Some(0),
    order_bys: None
};

let result = client.agents().query(query).await?;
println!("Found {} agents", result.page.total);
println!("Showing {} agents", result.result.len());

// Create with proper types
let job = JobForCreate {
    name: "my-job".to_string(),
    status: Some(JobStatus::Enabled),
    config: Some(TaskConfig {
        cmd: hetuflow_core::models::ExecuteCommand::Bash,
        args: vec!["echo".to_string(), "Hello".to_string()],
        timeout: 300,
        max_retries: 3,
        retry_interval: 60,
        capture_output: true,
        max_output_size: 1024 * 1024,
        ..Default::default()
    }),
    ..Default::default()
};
```

## Features Flags

- `default`: Native platform support
- `native`: Enable native platform support (reqwest + tokio)
- `wasm`: Enable WebAssembly support (wasm-bindgen + web-sys)
- `full`: Enable all features

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
