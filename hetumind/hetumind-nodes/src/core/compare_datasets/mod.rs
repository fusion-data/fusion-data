use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{Node, NodeDefinition, NodeExecutor, NodeGroupKind, NodeKind, RegistrationError},
};
use serde::{Deserialize, Serialize};

pub mod compare_v1;
pub mod utils;

use compare_v1::CompareDatasetsV1;

/// Comparison result type for datasets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComparisonResultType {
  /// Records only in Input Dataset A
  InAOnly,
  /// Records only in Input Dataset B
  InBOnly,
  /// Records that match exactly in both datasets
  Same,
  /// Records that exist in both but have different values
  Different,
}

/// Comparison mode for field matching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComparisonMode {
  /// Exact match comparison
  Exact,
  /// Fuzzy comparison with tolerance
  Fuzzy,
}

/// Conflict resolution strategy for handling differences
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConflictResolution {
  /// Prefer values from Input Dataset A
  PreferInputA,
  /// Prefer values from Input Dataset B
  PreferInputB,
  /// Mix values from both datasets
  Mix,
  /// Include both values when differences exist
  IncludeBoth,
}

/// Field matching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMatchConfig {
  /// Field name for matching (supports dot notation)
  pub field_name: String,
  /// Whether this field is a key field for matching
  pub is_key_field: bool,
  /// Comparison mode for this field
  pub comparison_mode: ComparisonMode,
  /// Fuzzy matching threshold (0.0 to 1.0, only used for fuzzy comparison)
  pub fuzzy_threshold: Option<f64>,
  /// Case sensitivity flag
  pub case_sensitive: bool,
  /// Whether to trim whitespace before comparison
  pub trim_whitespace: bool,
}

impl Default for FieldMatchConfig {
  fn default() -> Self {
    Self {
      field_name: String::new(),
      is_key_field: false,
      comparison_mode: ComparisonMode::Exact,
      fuzzy_threshold: None,
      case_sensitive: true,
      trim_whitespace: true,
    }
  }
}

/// Compare Datasets operation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareDatasetsOperation {
  /// List of fields to match on
  pub match_fields: Vec<FieldMatchConfig>,
  /// Conflict resolution strategy
  pub conflict_resolution: ConflictResolution,
  /// Whether to include all fields in output or only matched fields
  pub include_all_fields: bool,
  /// Whether to enable fuzzy matching for all fields
  pub enable_fuzzy_matching: bool,
  /// Global fuzzy matching threshold
  pub fuzzy_threshold: f64,
  /// Maximum number of differences to include in output
  pub max_differences: Option<usize>,
  /// Whether to sort output results
  pub sort_results: bool,
  /// Field to sort results by
  pub sort_field: Option<String>,
}

impl Default for CompareDatasetsOperation {
  fn default() -> Self {
    Self {
      match_fields: Vec::new(),
      conflict_resolution: ConflictResolution::PreferInputA,
      include_all_fields: true,
      enable_fuzzy_matching: false,
      fuzzy_threshold: 0.8,
      max_differences: None,
      sort_results: false,
      sort_field: None,
    }
  }
}

impl CompareDatasetsOperation {
  /// Validate the operation configuration
  pub fn validate(&self) -> Result<(), String> {
    // Check if we have at least one match field
    if self.match_fields.is_empty() {
      return Err("At least one match field must be specified".to_string());
    }

    // Check if we have at least one key field
    if !self.match_fields.iter().any(|f| f.is_key_field) {
      return Err("At least one key field must be specified".to_string());
    }

    // Validate fuzzy threshold
    if self.enable_fuzzy_matching && (self.fuzzy_threshold < 0.0 || self.fuzzy_threshold > 1.0) {
      return Err("Fuzzy threshold must be between 0.0 and 1.0".to_string());
    }

    // Validate each match field
    for field in &self.match_fields {
      if field.field_name.is_empty() {
        return Err("Field name cannot be empty".to_string());
      }

      // Validate fuzzy threshold for individual fields
      if let Some(threshold) = field.fuzzy_threshold {
        if threshold < 0.0 || threshold > 1.0 {
          return Err(format!("Fuzzy threshold for field '{}' must be between 0.0 and 1.0", field.field_name));
        }
      }
    }

    // Validate sort field if sorting is enabled
    if self.sort_results && self.sort_field.is_none() {
      return Err("Sort field must be specified when sort_results is true".to_string());
    }

    Ok(())
  }
}

/// Comparison result for a single record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordComparisonResult {
  /// Result type (InAOnly, InBOnly, Same, Different)
  pub result_type: ComparisonResultType,
  /// Record from Input Dataset A (if available)
  pub record_a: Option<serde_json::Value>,
  /// Record from Input Dataset B (if available)
  pub record_b: Option<serde_json::Value>,
  /// Merged record (for Different results with conflict resolution)
  pub merged_record: Option<serde_json::Value>,
  /// List of fields that differ (for Different results)
  pub differing_fields: Option<Vec<String>>,
  /// Match score (for fuzzy matching)
  pub match_score: Option<f64>,
  /// Match reason or description
  pub match_reason: Option<String>,
}

/// Overall comparison results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResults {
  /// Records only in Input Dataset A
  pub in_a_only: Vec<RecordComparisonResult>,
  /// Records only in Input Dataset B
  pub in_b_only: Vec<RecordComparisonResult>,
  /// Records that match exactly
  pub same: Vec<RecordComparisonResult>,
  /// Records that exist in both but differ
  pub different: Vec<RecordComparisonResult>,
  /// Summary statistics
  pub summary: ComparisonSummary,
}

/// Summary statistics for the comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonSummary {
  /// Total records in Input Dataset A
  pub total_a: usize,
  /// Total records in Input Dataset B
  pub total_b: usize,
  /// Count of records only in A
  pub count_in_a_only: usize,
  /// Count of records only in B
  pub count_in_b_only: usize,
  /// Count of matching records
  pub count_same: usize,
  /// Count of different records
  pub count_different: usize,
  /// Overall match percentage
  pub match_percentage: f64,
}

/// Compare Datasets Node implementation
pub struct CompareDatasetsNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl CompareDatasetsNode {
  /// Create a new CompareDatasetsNode
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(CompareDatasetsV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  fn base() -> NodeDefinition {
    NodeDefinition::new("hetumind_nodes::CompareDatasets", Version::new(1, 0, 0), "Compare Datasets")
      .add_group(NodeGroupKind::Transform)
      .with_description("Compare two datasets and categorize results into matched, unmatched, and different records")
      .with_icon("compare")
  }
}

impl Node for CompareDatasetsNode {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[NodeExecutor] {
    &self.executors
  }

  fn kind(&self) -> NodeKind {
    self.executors[0].definition().kind.clone()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_field_match_config_default() {
    let config = FieldMatchConfig::default();
    assert_eq!(config.field_name, "");
    assert!(!config.is_key_field);
    assert_eq!(config.comparison_mode, ComparisonMode::Exact);
    assert_eq!(config.fuzzy_threshold, None);
    assert!(config.case_sensitive);
    assert!(config.trim_whitespace);
  }

  #[test]
  fn test_compare_operation_default() {
    let operation = CompareDatasetsOperation::default();
    assert!(operation.match_fields.is_empty());
    assert_eq!(operation.conflict_resolution, ConflictResolution::PreferInputA);
    assert!(operation.include_all_fields);
    assert!(!operation.enable_fuzzy_matching);
    assert_eq!(operation.fuzzy_threshold, 0.8);
    assert_eq!(operation.max_differences, None);
    assert!(!operation.sort_results);
    assert_eq!(operation.sort_field, None);
  }

  #[test]
  fn test_compare_operation_validation_empty_fields() {
    let operation = CompareDatasetsOperation::default();
    let result = operation.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "At least one match field must be specified");
  }

  #[test]
  fn test_compare_operation_validation_no_key_field() {
    let mut operation = CompareDatasetsOperation::default();
    operation.match_fields.push(FieldMatchConfig {
      field_name: "name".to_string(),
      is_key_field: false,
      ..Default::default()
    });

    let result = operation.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "At least one key field must be specified");
  }

  #[test]
  fn test_compare_operation_validation_invalid_fuzzy_threshold() {
    let mut operation = CompareDatasetsOperation::default();
    operation.match_fields.push(FieldMatchConfig {
      field_name: "id".to_string(),
      is_key_field: true,
      ..Default::default()
    });
    operation.enable_fuzzy_matching = true;
    operation.fuzzy_threshold = 1.5; // Invalid threshold

    let result = operation.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Fuzzy threshold must be between 0.0 and 1.0"));
  }

  #[test]
  fn test_compare_operation_validation_sort_without_field() {
    let mut operation = CompareDatasetsOperation::default();
    operation.match_fields.push(FieldMatchConfig {
      field_name: "id".to_string(),
      is_key_field: true,
      ..Default::default()
    });
    operation.sort_results = true;
    operation.sort_field = None;

    let result = operation.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Sort field must be specified when sort_results is true");
  }

  #[test]
  fn test_compare_operation_validation_valid() {
    let mut operation = CompareDatasetsOperation::default();
    operation.match_fields.push(FieldMatchConfig {
      field_name: "id".to_string(),
      is_key_field: true,
      ..Default::default()
    });

    let result = operation.validate();
    assert!(result.is_ok());
  }

  #[test]
  fn test_comparison_result_type_serialization() {
    let result_type = ComparisonResultType::InAOnly;
    let serialized = serde_json::to_string(&result_type).unwrap();
    assert_eq!(serialized, "\"IN_A_ONLY\"");

    let deserialized: ComparisonResultType = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, ComparisonResultType::InAOnly);
  }

  #[test]
  fn test_conflict_resolution_serialization() {
    let resolution = ConflictResolution::PreferInputA;
    let serialized = serde_json::to_string(&resolution).unwrap();
    assert_eq!(serialized, "\"PREFER_INPUT_A\"");

    let deserialized: ConflictResolution = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, ConflictResolution::PreferInputA);
  }

  #[test]
  fn test_node_creation() {
    let node = CompareDatasetsNode::new();
    assert!(node.is_ok());

    let node = node.unwrap();
    assert_eq!(node.default_version().major, 1);
    assert_eq!(node.node_executors().len(), 1);
  }
}
