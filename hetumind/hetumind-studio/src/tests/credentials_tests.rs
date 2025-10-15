#[cfg(test)]
mod tests {
    use super::*;
    use hetumind_core::credential::{CredentialId, CredentialReference};
    use serde_json::json;

    // Mock credential service for testing
    struct MockCredentialSvc {
        references: std::collections::HashMap<CredentialId, Vec<CredentialReference>>,
    }

    impl MockCredentialSvc {
        fn new() -> Self {
            Self {
                references: std::collections::HashMap::new(),
            }
        }

        fn add_reference(&mut self, credential_id: CredentialId, reference: CredentialReference) {
            self.references.entry(credential_id).or_insert_with(Vec::new).push(reference);
        }
    }

    #[tokio::test]
    async fn test_credential_reference_response_serialization() {
        let credential_id = CredentialId::new();
        let references = vec![
            CredentialReference {
                id: "workflow-1".to_string(),
                name: "Main Workflow".to_string(),
                node_type: "trigger".to_string(),
                node_name: "webhook_trigger".to_string(),
                parameter_path: "auth.api_key".to_string(),
                created_at: fusion_common::time::now_utc(),
                updated_at: fusion_common::time::now_utc(),
            },
            CredentialReference {
                id: "workflow-2".to_string(),
                name: "Data Processing".to_string(),
                node_type: "action".to_string(),
                node_name: "api_call".to_string(),
                parameter_path: "headers.Authorization".to_string(),
                created_at: fusion_common::time::now_utc(),
                updated_at: fusion_common::time::now_utc(),
            },
        ];

        let response = CredentialReferenceResponse {
            credential_id,
            references,
            total_count: 2,
        };

        // Test serialization
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Main Workflow"));
        assert!(json.contains("webhook_trigger"));
        assert!(json.contains("2"));

        // Test deserialization
        let deserialized: CredentialReferenceResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_count, 2);
        assert_eq!(deserialized.references.len(), 2);
        assert_eq!(deserialized.references[0].name, "Main Workflow");
    }

    #[tokio::test]
    async fn test_credential_reference_serialization() {
        let reference = CredentialReference {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            node_type: "action".to_string(),
            node_name: "test_node".to_string(),
            parameter_path: "config.api_key".to_string(),
            created_at: fusion_common::time::now_utc(),
            updated_at: fusion_common::time::now_utc(),
        };

        // Test serialization
        let json = serde_json::to_string(&reference).unwrap();
        assert!(json.contains("test-workflow"));
        assert!(json.contains("Test Workflow"));
        assert!(json.contains("config.api_key"));

        // Test deserialization
        let deserialized: CredentialReference = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test-workflow");
        assert_eq!(deserialized.name, "Test Workflow");
        assert_eq!(deserialized.node_type, "action");
        assert_eq!(deserialized.node_name, "test_node");
        assert_eq!(deserialized.parameter_path, "config.api_key");
    }

    #[tokio::test]
    async fn test_empty_credential_references() {
        let credential_id = CredentialId::new();
        let response = CredentialReferenceResponse {
            credential_id,
            references: vec![],
            total_count: 0,
        };

        // Test serialization
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("0"));

        // Test deserialization
        let deserialized: CredentialReferenceResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_count, 0);
        assert!(deserialized.references.is_empty());
    }

    #[tokio::test]
    async fn test_credential_reference_types() {
        let credential_id = CredentialId::new();

        // Test different node types
        let node_types = vec!["trigger", "action", "condition", "transform"];

        for node_type in node_types {
            let reference = CredentialReference {
                id: format!("workflow-{}", node_type),
                name: format!("{} Workflow", node_type.to_uppercase()),
                node_type: node_type.to_string(),
                node_name: format!("{}_node", node_type),
                parameter_path: "auth.token".to_string(),
                created_at: fusion_common::time::now_utc(),
                updated_at: fusion_common::time::now_utc(),
            };

            let response = CredentialReferenceResponse {
                credential_id,
                references: vec![reference.clone()],
                total_count: 1,
            };

            // Test serialization and deserialization
            let json = serde_json::to_string(&response).unwrap();
            let deserialized: CredentialReferenceResponse = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.references[0].node_type, node_type);
        }
    }

    #[tokio::test]
    async fn test_parameter_path_formats() {
        let credential_id = CredentialId::new();
        let parameter_paths = vec![
            "auth.api_key",
            "config.token",
            "headers.Authorization",
            "connection.password",
            "credentials.client_secret",
        ];

        for path in parameter_paths {
            let reference = CredentialReference {
                id: "test-workflow".to_string(),
                name: "Test Workflow".to_string(),
                node_type: "action".to_string(),
                node_name: "test_node".to_string(),
                parameter_path: path.to_string(),
                created_at: fusion_common::time::now_utc(),
                updated_at: fusion_common::time::now_utc(),
            };

            let response = CredentialReferenceResponse {
                credential_id,
                references: vec![reference.clone()],
                total_count: 1,
            };

            // Test serialization and deserialization
            let json = serde_json::to_string(&response).unwrap();
            let deserialized: CredentialReferenceResponse = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.references[0].parameter_path, path);
        }
    }

    #[tokio::test]
    async fn test_credential_id_serialization() {
        let credential_id = CredentialId::new();

        // Test serialization
        let json = serde_json::to_string(&credential_id).unwrap();
        assert!(!json.is_empty());

        // Test deserialization
        let deserialized: CredentialId = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, credential_id);
    }

    #[tokio::test]
    async fn test_timestamp_serialization() {
        let now = fusion_common::time::now_utc();
        let later = now + fusion_common::time::Duration::hours(1);

        let reference = CredentialReference {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            node_type: "action".to_string(),
            node_name: "test_node".to_string(),
            parameter_path: "auth.token".to_string(),
            created_at: now,
            updated_at: later,
        };

        // Test serialization
        let json = serde_json::to_string(&reference).unwrap();

        // Test deserialization
        let deserialized: CredentialReference = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.created_at, now);
        assert_eq!(deserialized.updated_at, later);
    }

    #[tokio::test]
    async fn test_multiple_credential_references() {
        let credential_id = CredentialId::new();
        let mut references = Vec::new();

        // Create multiple references
        for i in 1..=5 {
            references.push(CredentialReference {
                id: format!("workflow-{}", i),
                name: format!("Workflow {}", i),
                node_type: "action".to_string(),
                node_name: format!("node_{}", i),
                parameter_path: format!("config.param_{}", i),
                created_at: fusion_common::time::now_utc(),
                updated_at: fusion_common::time::now_utc(),
            });
        }

        let response = CredentialReferenceResponse {
            credential_id,
            references,
            total_count: 5,
        };

        // Test serialization
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("5"));

        // Test deserialization
        let deserialized: CredentialReferenceResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_count, 5);
        assert_eq!(deserialized.references.len(), 5);

        // Verify all references are present
        for i in 1..=5 {
            let found = deserialized.references.iter().any(|ref_| {
                ref_.id == format!("workflow-{}", i) &&
                ref_.name == format!("Workflow {}", i) &&
                ref_.parameter_path == format!("config.param_{}", i)
            });
            assert!(found, "Reference for workflow-{} not found", i);
        }
    }

    #[tokio::test]
    async fn test_reference_sorting() {
        let credential_id = CredentialId::new();
        let base_time = fusion_common::time::now_utc();

        // Create references with different update times
        let mut references = vec![
            CredentialReference {
                id: "workflow-3".to_string(),
                name: "Workflow 3".to_string(),
                node_type: "action".to_string(),
                node_name: "node_3".to_string(),
                parameter_path: "config.param_3".to_string(),
                created_at: base_time,
                updated_at: base_time + fusion_common::time::Duration::minutes(30),
            },
            CredentialReference {
                id: "workflow-1".to_string(),
                name: "Workflow 1".to_string(),
                node_type: "action".to_string(),
                node_name: "node_1".to_string(),
                parameter_path: "config.param_1".to_string(),
                created_at: base_time,
                updated_at: base_time + fusion_common::time::Duration::minutes(10),
            },
            CredentialReference {
                id: "workflow-2".to_string(),
                name: "Workflow 2".to_string(),
                node_type: "action".to_string(),
                node_name: "node_2".to_string(),
                parameter_path: "config.param_2".to_string(),
                created_at: base_time,
                updated_at: base_time + fusion_common::time::Duration::minutes(20),
            },
        ];

        // Sort by updated_at descending
        references.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        let response = CredentialReferenceResponse {
            credential_id,
            references,
            total_count: 3,
        };

        // Verify sorting
        assert_eq!(response.references[0].id, "workflow-3");
        assert_eq!(response.references[1].id, "workflow-2");
        assert_eq!(response.references[2].id, "workflow-1");
    }

    // Integration test example - this would require actual database setup
    #[ignore]
    #[tokio::test]
    async fn test_get_credential_references_integration() {
        // This test would require:
        // 1. Database setup with credential and workflow tables
        // 2. Real CredentialSvc implementation
        // 3. Actual credential references data

        // Placeholder for integration test
        // let app = create_test_application();
        // let credential_svc = CredentialSvc::new(app.component());
        // let credential_id = CredentialId::new();

        // let result = get_credential_references(credential_svc, axum::extract::Path(credential_id)).await;
        // assert!(result.is_ok());

        // let response = result.unwrap();
        // assert!(response.total_count >= 0);
        // assert_eq!(response.credential_id, credential_id);
    }
}