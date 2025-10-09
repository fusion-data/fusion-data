//! Usage example for the Hetuflow SDK using full APIs

use fusion_common::page::Page;
use hetuflow_core::models::{AgentForQuery, JobForCreate, JobForQuery, TaskForQuery};
use hetuflow_core::types::JobStatus;
use hetuflow_sdk::HetuflowClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Initialize the client
  let client = HetuflowClient::new("http://localhost:9500".to_string())?;

  // Example 1: Get system health
  println!("=== System Health ===");
  match client.system().health().await {
    Ok(health) => println!("Health: {}", serde_json::to_string_pretty(&health)?),
    Err(e) => println!("Failed to get health: {}", e),
  }

  // Example 2: Get system metrics
  println!("\n=== System Metrics ===");
  match client.system().metrics().await {
    Ok(metrics) => println!("Metrics: {}", serde_json::to_string_pretty(&metrics)?),
    Err(e) => println!("Failed to get metrics: {}", e),
  }

  // Example 3: List all agents
  println!("\n=== Listing Agents ===");
  let query = AgentForQuery { page: Page { limit: Some(10), ..Default::default() }, ..Default::default() };

  match client.agents().query(query).await {
    Ok(result) => {
      if result.result.is_empty() {
        println!("No agents found");
      } else {
        println!(
          "Found {} agents (showing page {}):",
          result.page.total,
          1 // current page - we can simplify this since we're not tracking total pages
        );
        for agent in result.result {
          println!("  - {}: {} ({:?})", agent.id, agent.address, agent.status);
        }
      }
    }
    Err(e) => println!("Failed to list agents: {}", e),
  }

  // Example 4: Query jobs with pagination
  println!("\n=== Querying Jobs ===");
  let job_query = JobForQuery {
    page: Page { page: Some(1), limit: Some(10), offset: Some(0), order_bys: None },
    ..Default::default()
  };
  let current_page = job_query.page.page; // Store page info before moving

  match client.jobs().query(job_query).await {
    Ok(result) => println!("Found {} jobs (page {:?})", result.page.total, current_page),
    Err(e) => println!("Failed to query jobs: {}", e),
  }

  // Example 5: Create a new job
  println!("\n=== Creating Job ===");
  let create_job = JobForCreate {
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
    status: Some(JobStatus::Enabled),
  };

  match client.jobs().create(create_job).await {
    Ok(result) => println!("Created job with ID: {}", result.id),
    Err(e) => println!("Failed to create job: {}", e),
  }

  // Example 6: Find tasks by status
  println!("\n=== Finding Failed Tasks ===");
  let task_query = TaskForQuery {
    page: Page { page: Some(1), limit: Some(50), offset: Some(0), order_bys: None },
    ..Default::default()
  };

  match client.tasks().query(task_query).await {
    Ok(result) => {
      let failed_tasks: Vec<_> = result
        .result
        .into_iter()
        .filter(|task| task.status == hetuflow_core::types::TaskStatus::Failed)
        .collect();

      if failed_tasks.is_empty() {
        println!("No failed tasks found");
      } else {
        println!("Found {} failed tasks:", failed_tasks.len());
        for task in &failed_tasks {
          println!("  - {}: {:?} (retry count: {})", task.id, task.status, task.retry_count);
        }
      }
    }
    Err(e) => println!("Failed to query tasks: {}", e),
  }

  // Example 7: Get schedulable schedules
  println!("\n=== Getting Schedulable Schedules ===");
  match client.schedules().get_schedulable().await {
    Ok(schedules) => {
      if schedules.is_empty() {
        println!("No schedulable schedules found");
      } else {
        println!("Found {} schedulable schedules:", schedules.len());
        for schedule in schedules {
          println!("  - {}: {} ({:?})", schedule.id, schedule.name.as_deref().unwrap_or("unnamed"), schedule.status);
        }
      }
    }
    Err(e) => println!("Failed to get schedulable schedules: {}", e),
  }

  Ok(())
}
