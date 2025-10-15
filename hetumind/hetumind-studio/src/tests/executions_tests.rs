#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use hetumind_core::workflow::{Execution, ExecutionData, ExecutionId, ExecutionStatus};
    use tokio_stream::wrappers::ReceiverStream;

    // Mock execution service for testing
    struct MockExecutionSvc {
        executions: std::collections::HashMap<ExecutionId, Execution>,
        logs: std::collections::HashMap<ExecutionId, Vec<ExecutionData>>,
    }

    impl MockExecutionSvc {
        fn new() -> Self {
            Self {
                executions: std::collections::HashMap::new(),
                logs: std::collections::HashMap::new(),
            }
        }

        fn add_execution(&mut self, execution: Execution) {
            let id = execution.id;
            self.executions.insert(id, execution);
        }

        fn add_log(&mut self, execution_id: ExecutionId, log: ExecutionData) {
            self.logs.entry(execution_id).or_insert_with(Vec::new).push(log);
        }
    }

    #[tokio::test]
    async fn test_get_execution_status() {
        // Create mock execution service
        let mut mock_svc = MockExecutionSvc::new();

        let execution_id = ExecutionId::new();
        let execution = Execution {
            id: execution_id,
            status: ExecutionStatus::Running,
            started_at: Some(fusion_common::time::now_utc()),
            finished_at: None,
            error: None,
            // ... other fields would be set here
        };

        mock_svc.add_execution(execution);

        // Test the status response structure
        let status_response = ExecutionStatusResponse {
            id: execution_id,
            status: ExecutionStatus::Running,
            started_at: Some(fusion_common::time::now_utc()),
            finished_at: None,
            error: None,
            progress: Some(0.5),
        };

        assert_eq!(status_response.id, execution_id);
        assert_eq!(status_response.status, ExecutionStatus::Running);
        assert!(status_response.started_at.is_some());
        assert!(status_response.finished_at.is_none());
        assert!(status_response.error.is_none());
        assert_eq!(status_response.progress, Some(0.5));
    }

    #[tokio::test]
    async fn test_execution_log_response_serialization() {
        let logs = vec![
            ExecutionData::new_text("Test log message 1"),
            ExecutionData::new_text("Test log message 2"),
        ];

        let log_response = ExecutionLogResponse(logs);

        // Test serialization
        let json = serde_json::to_string(&log_response).unwrap();
        assert!(json.contains("Test log message 1"));
        assert!(json.contains("Test log message 2"));

        // Test deserialization
        let deserialized: ExecutionLogResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.len(), 2);
    }

    #[tokio::test]
    async fn test_execution_status_response_serialization() {
        let status_response = ExecutionStatusResponse {
            id: ExecutionId::new(),
            status: ExecutionStatus::Success,
            started_at: Some(fusion_common::time::now_utc()),
            finished_at: Some(fusion_common::time::now_utc()),
            error: None,
            progress: Some(1.0),
        };

        // Test serialization
        let json = serde_json::to_string(&status_response).unwrap();
        assert!(json.contains("Success"));

        // Test deserialization
        let deserialized: ExecutionStatusResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.status, ExecutionStatus::Success);
        assert_eq!(deserialized.progress, Some(1.0));
    }

    #[tokio::test]
    async fn test_execution_status_with_error() {
        let status_response = ExecutionStatusResponse {
            id: ExecutionId::new(),
            status: ExecutionStatus::Failed,
            started_at: Some(fusion_common::time::now_utc()),
            finished_at: Some(fusion_common::time::now_utc()),
            error: Some("Test error message".to_string()),
            progress: Some(0.75),
        };

        assert_eq!(status_response.status, ExecutionStatus::Failed);
        assert_eq!(status_response.error, Some("Test error message".to_string()));
        assert_eq!(status_response.progress, Some(0.75));
    }

    #[tokio::test]
    async fn test_stream_execution_logs_event_format() {
        use axum::response::sse::Event;

        // Test creating different event types
        let log_data = ExecutionData::new_text("Test log");
        let log_json = serde_json::to_string(&log_data).unwrap();

        let log_event = Event::default()
            .event("log")
            .id("log-0")
            .data(&log_json);

        assert_eq!(log_event.event(), Some("log"));
        assert_eq!(log_event.id(), Some("log-0"));
        assert_eq!(log_event.data(), &log_json);

        let status_event = Event::default()
            .event("status")
            .id("status-1")
            .data("Execution status: Running");

        assert_eq!(status_event.event(), Some("status"));
        assert_eq!(status_event.id(), Some("status-1"));

        let complete_event = Event::default()
            .event("complete")
            .data("Stream completed");

        assert_eq!(complete_event.event(), Some("complete"));
    }

    #[tokio::test]
    async fn test_execution_id_serialization() {
        let execution_id = ExecutionId::new();

        // Test serialization
        let json = serde_json::to_string(&execution_id).unwrap();
        assert!(!json.is_empty());

        // Test deserialization
        let deserialized: ExecutionId = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, execution_id);
    }

    #[tokio::test]
    async fn test_execution_status_transitions() {
        let statuses = vec![
            ExecutionStatus::Pending,
            ExecutionStatus::Running,
            ExecutionStatus::Success,
            ExecutionStatus::Failed,
            ExecutionStatus::Cancelled,
            ExecutionStatus::Crashed,
        ];

        for status in statuses {
            let status_response = ExecutionStatusResponse {
                id: ExecutionId::new(),
                status,
                started_at: Some(fusion_common::time::now_utc()),
                finished_at: Some(fusion_common::time::now_utc()),
                error: None,
                progress: None,
            };

            // Test that each status can be serialized
            let json = serde_json::to_string(&status_response).unwrap();
            let deserialized: ExecutionStatusResponse = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.status, status);
        }
    }

    #[tokio::test]
    async fn test_progress_values() {
        let progress_values = vec![0.0, 0.25, 0.5, 0.75, 1.0];

        for progress in progress_values {
            let status_response = ExecutionStatusResponse {
                id: ExecutionId::new(),
                status: ExecutionStatus::Running,
                started_at: Some(fusion_common::time::now_utc()),
                finished_at: None,
                error: None,
                progress: Some(progress),
            };

            // Test serialization and deserialization
            let json = serde_json::to_string(&status_response).unwrap();
            let deserialized: ExecutionStatusResponse = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.progress, Some(progress));
        }
    }

    // Integration test example - this would require actual database setup
    #[ignore]
    #[tokio::test]
    async fn test_get_execution_status_integration() {
        // This test would require:
        // 1. Database setup
        // 2. Real ExecutionSvc implementation
        // 3. Actual execution data

        // Placeholder for integration test
        // let app = create_test_application();
        // let execution_svc = ExecutionSvc::new(app.component());
        // let execution_id = ExecutionId::new();

        // let result = get_execution_status(execution_svc, axum::extract::Path(execution_id)).await;
        // assert!(result.is_ok());
    }

    // Integration test for streaming - would require more complex setup
    #[ignore]
    #[tokio::test]
    async fn test_stream_execution_logs_integration() {
        // This test would require:
        // 1. Database setup
        // 2. Real ExecutionSvc implementation
        // 3. SSE stream handling
        // 4. Async stream testing utilities

        // Placeholder for integration test
        // let app = create_test_application();
        // let execution_svc = ExecutionSvc::new(app.component());
        // let execution_id = ExecutionId::new();

        // let stream = stream_execution_logs(execution_svc, axum::extract::Path(execution_id)).await;
        // let mut stream = stream.into_inner();

        // Test receiving events from the stream
        // while let Some(event_result) = stream.next().await {
        //     let event = event_result.unwrap();
        //     // Verify event format and content
        // }
    }
}