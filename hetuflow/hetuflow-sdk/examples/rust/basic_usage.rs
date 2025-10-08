//! Basic usage example for the Hetuflow SDK

use hetuflow_sdk::{HetuflowClient, Config};
use fusionsql::page::Page;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the client
    let config = Config::new("http://localhost:8080")
        .with_timeout(Duration::from_secs(10))
        .with_retry_config(3, Duration::from_millis(1000), Duration::from_secs(30));

    let client = HetuflowClient::with_config(config)?;

    // Example 1: List all agents
    println!("=== Listing Agents ===");
    let agents = client.agents().list().await?;
    for agent in agents {
        println!("Agent: {} - {} ({})", agent.id, agent.address, agent.status);
    }

    // Example 2: Query jobs with pagination
    println!("\n=== Querying Jobs ===");
    let mut query = hetuflow_sdk::JobForQuery::default();
    query.page = Page {
        page: 1,
        size: 10,
        ..Default::default()
    };

    let jobs = client.jobs().query(query).await?;
    println!("Found {} jobs (page {} of {})", jobs.total, jobs.page, jobs.total_pages);

    // Example 3: Create a new job
    println!("\n=== Creating Job ===");
    let create_job = hetuflow_sdk::JobForCreate {
        id: None,
        namespace_id: Some("default".to_string()),
        name: "example-job".to_string(),
        description: Some("Example job created via SDK".to_string()),
        environment: Some(serde_json::json!({
            "ENV": "development"
        })),
        config: Some(hetuflow_sdk::TaskConfig {
            timeout: 300,
            max_retries: 3,
            retry_interval: 60,
            cmd: hetuflow_sdk::ExecuteCommand::Bash,
            args: vec!["echo".to_string(), "Hello, Hetuflow!".to_string()],
            capture_output: true,
            max_output_size: 1024 * 1024, // 1MB
            labels: None,
            resource_limits: None,
        }),
        status: Some(hetuflow_sdk::JobStatus::Enabled),
    };

    match client.jobs().create(create_job).await {
        Ok(result) => println!("Created job with ID: {}", result.id),
        Err(e) => println!("Failed to create job: {}", e),
    }

    // Example 4: Get system health
    println!("\n=== System Health ===");
    match client.system().health().await {
        Ok(health) => println!("System health: {}", serde_json::to_string_pretty(&health)?),
        Err(e) => println!("Failed to get health: {}", e),
    }

    // Example 5: Find tasks by status
    println!("\n=== Finding Failed Tasks ===");
    match client.tasks().find_by_status(hetuflow_sdk::TaskStatus::Failed).await {
        Ok(failed_tasks) => {
            if failed_tasks.is_empty() {
                println!("No failed tasks found");
            } else {
                println!("Found {} failed tasks:", failed_tasks.len());
                for task in &failed_tasks {
                    println!("  - {}: {} (retry count: {})", task.id, task.status, task.retry_count);
                }
            }
        }
        Err(e) => println!("Failed to query tasks: {}", e),
    }

    Ok(())
}
