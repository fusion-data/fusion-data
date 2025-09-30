# Hetuflow SDK WASM Guide

This guide explains how to generate and use WebAssembly (WASM) bindings for the Hetuflow SDK.

## Prerequisites

1. **Install Rust WASM target:**
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. **Install wasm-pack:**
   ```bash
   cargo install wasm-pack
   ```

3. **Install a local HTTP server (for testing):**
   ```bash
   # Python 3
   python3 -m http.server 8080

   # Or Node.js
   npx serve
   ```

## Building WASM Package

### Option 1: Using the Build Script

```bash
cd hetuflow/hetuflow-sdk
./examples/wasm/build-wasm.sh
```

### Option 2: Manual Build

```bash
cd hetuflow/hetuflow-sdk
wasm-pack build --target web --out-dir examples/wasm/pkg --features wasm
```

This will create a `pkg/` directory containing:
- `hetuflow_sdk.js` - JavaScript bindings
- `hetuflow_sdk_bg.wasm` - WebAssembly binary
- `hetuflow_sdk.d.ts` - TypeScript definitions
- Other support files

## Running the Example

1. **Build the WASM package** (see above)

2. **Start a local server:**
   ```bash
   cd examples/wasm
   python3 -m http.server 8080
   ```

3. **Open in browser:**
   ```
   http://localhost:8080
   ```

## Usage in JavaScript/TypeScript

### Basic Setup

```javascript
import init, {
    WasmHetuflowClient,
    WasmAgentQuery,
    WasmAgentCreate,
    WasmJobQuery,
    WasmJobCreate
} from "./pkg/hetuflow_sdk.js";

// Initialize WASM module
await init();

// Create client
const client = new WasmHetuflowClient("http://localhost:9500");

// Add authentication (optional)
const authenticatedClient = client.with_auth_token("your-token-here");
```

### Querying Data

```javascript
// Query agents
const agentQuery = new WasmAgentQuery()
    .with_page(1)
    .with_limit(10)
    .with_status("online");

const agents = await client.query_agents(agentQuery);
console.log("Agents:", agents);

// Query jobs
const jobQuery = new WasmJobQuery()
    .with_page(1)
    .with_limit(20)
    .with_name("example-job");

const jobs = await client.query_jobs(jobQuery);
console.log("Jobs:", jobs);
```

### Creating Resources

```javascript
// Create an agent
const agent = new WasmAgentCreate("agent-1", "Test Agent", "localhost", 8081)
    .with_description("A test agent for demonstration");

const agentResult = await client.create_agent(agent);
console.log("Created agent:", agentResult);

// Create a job
const job = new WasmJobCreate("example-job", "echo")
    .with_namespace("default")
    .with_args(["Hello", "Hetuflow!"])
    .with_timeout(300);

const jobResult = await client.create_job(job);
console.log("Created job:", jobResult);
```

### System Operations

```javascript
// Check system health
const health = await client.health();
console.log("System health:", health);
```

## API Reference

### WasmHetuflowClient

The main client for interacting with the Hetuflow API.

#### Constructor
- `new(base_url: string)` - Create a new client

#### Methods
- `with_auth_token(token: string)` - Add authentication token
- `health()` - Get system health status
- `query_agents(query: WasmAgentQuery)` - Query agents
- `create_agent(agent: WasmAgentCreate)` - Create a new agent
- `query_jobs(query: WasmJobQuery)` - Query jobs
- `create_job(job: WasmJobCreate)` - Create a new job

### Query Types

#### WasmAgentQuery
- `new()` - Create new query
- `with_page(page: number)` - Set page number
- `with_limit(limit: number)` - Set page limit
- `with_status(status: string)` - Filter by status

#### WasmJobQuery
- `new()` - Create new query
- `with_page(page: number)` - Set page number
- `with_limit(limit: number)` - Set page limit
- `with_name(name: string)` - Filter by name

### Create Types

#### WasmAgentCreate
- `new(id: string, name: string, host: string, port: number)` - Create new agent
- `with_description(description: string)` - Set description

#### WasmJobCreate
- `new(name: string, command: string)` - Create new job
- `with_namespace(namespace: string)` - Set namespace
- `with_description(description: string)` - Set description
- `with_args(args: string[])` - Set command arguments
- `with_timeout(timeout: number)` - Set timeout in seconds

## Error Handling

All WASM functions return promises that resolve to the result or reject with an error:

```javascript
try {
    const agents = await client.query_agents(query);
    console.log("Success:", agents);
} catch (error) {
    console.error("Error:", error);
}
```

## Integration Examples

### React Integration

```jsx
import React, { useState, useEffect } from 'react';
import init, { WasmHetuflowClient } from './pkg/hetuflow_sdk.js';

function HetuflowComponent() {
    const [client, setClient] = useState(null);
    const [agents, setAgents] = useState([]);

    useEffect(() => {
        async function initialize() {
            await init();
            const wasmClient = new WasmHetuflowClient("http://localhost:9500");
            setClient(wasmClient);
        }
        initialize();
    }, []);

    const loadAgents = async () => {
        if (!client) return;

        const query = new WasmAgentQuery().with_page(1).with_limit(10);
        const result = await client.query_agents(query);
        setAgents(result.result || []);
    };

    return (
        <div>
            <button onClick={loadAgents} disabled={!client}>
                Load Agents
            </button>
            <ul>
                {agents.map(agent => (
                    <li key={agent.id}>{agent.name}</li>
                ))}
            </ul>
        </div>
    );
}
```

### Vue.js Integration

```javascript
import { ref, onMounted } from 'vue';
import init, { WasmHetuflowClient } from './pkg/hetuflow_sdk.js';

export default {
    setup() {
        const client = ref(null);
        const agents = ref([]);

        onMounted(async () => {
            await init();
            client.value = new WasmHetuflowClient("http://localhost:9500");
        });

        const loadAgents = async () => {
            if (!client.value) return;

            const query = new WasmAgentQuery().with_page(1).with_limit(10);
            const result = await client.value.query_agents(query);
            agents.value = result.result || [];
        };

        return { client, agents, loadAgents };
    }
};
```

## Limitations

1. **Browser-only**: WASM bindings are designed for browser environments
2. **HTTP only**: Currently supports only HTTP/HTTPS requests
3. **JSON serialization**: All data is serialized as JSON
4. **Limited API**: Only a subset of the full SDK is exposed to WASM

## Troubleshooting

### Common Issues

1. **"No global window object"**: Make sure you're running in a browser environment
2. **"Failed to fetch"**: Check CORS settings on your Hetuflow server
3. **"WASM initialization failed"**: Ensure all files are served from the same directory

### CORS Configuration

If you encounter CORS errors, make sure your Hetuflow server is configured to allow requests from your web application:

```rust
// Example CORS configuration
use tower_http::cors::{Any, CorsLayer};

let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

let app = Router::new()
    .layer(cors)
    // ... your routes
```

## Development

### Adding New API Methods

1. Add the method to `WasmHetuflowClient` in `src/wasm/bindings.rs`
2. Implement the HTTP request logic
3. Add corresponding query/create types if needed
4. Rebuild with `wasm-pack build`

### Testing

```bash
# Run WASM tests
wasm-pack test --headless --firefox

# Or build and test manually
wasm-pack build --target web --out-dir examples/wasm/pkg --features wasm
cd examples/wasm
python3 -m http.server 8080
```

## Support

For issues and questions:
- Check the [GitHub Issues](https://github.com/your-org/hetuflow/issues)
- Review the [API Documentation](https://docs.rs/hetuflow-sdk)
- Join our [Discord Community](https://discord.gg/hetuflow)