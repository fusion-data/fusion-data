use regex::Regex;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::{
  CompareDatasetsOperation, ComparisonMode, ComparisonResultType, ComparisonResults, ComparisonSummary,
  ConflictResolution, FieldMatchConfig, RecordComparisonResult,
};

/// Compare two datasets and return categorized results
pub async fn compare_datasets(
  data_a: &[Value],
  data_b: &[Value],
  operation: &CompareDatasetsOperation,
) -> Result<ComparisonResults, String> {
  let mut in_a_only = Vec::new();
  let mut in_b_only = Vec::new();
  let mut same = Vec::new();
  let mut different = Vec::new();

  // Create lookup maps for efficient matching
  let (lookup_a, lookup_b) = create_lookup_maps(data_a, data_b, operation)?;

  // Process records from dataset A
  for record_a in data_a {
    let key = extract_key_value(record_a, operation)?;

    match lookup_b.get(&key) {
      Some(record_b) => {
        // Found matching record in B, compare details
        let comparison_result = compare_records(record_a, record_b, operation).await?;

        match comparison_result.result_type {
          ComparisonResultType::Same => same.push(comparison_result),
          ComparisonResultType::Different => different.push(comparison_result),
          _ => {} // Should not happen for matched records
        }
      }
      None => {
        // No matching record found in B
        in_a_only.push(RecordComparisonResult {
          result_type: ComparisonResultType::InAOnly,
          record_a: Some(record_a.clone()),
          record_b: None,
          merged_record: None,
          differing_fields: None,
          match_score: None,
          match_reason: Some("No match found in dataset B".to_string()),
        });
      }
    }
  }

  // Process records from dataset B that weren't matched
  for record_b in data_b {
    let key = extract_key_value(record_b, operation)?;

    if !lookup_a.contains_key(&key) {
      in_b_only.push(RecordComparisonResult {
        result_type: ComparisonResultType::InBOnly,
        record_a: None,
        record_b: Some(record_b.clone()),
        merged_record: None,
        differing_fields: None,
        match_score: None,
        match_reason: Some("No match found in dataset A".to_string()),
      });
    }
  }

  // Apply max differences limit if specified
  if let Some(max_diff) = operation.max_differences {
    different.truncate(max_diff);
  }

  // Sort results if requested
  if operation.sort_results {
    let sort_field = operation.sort_field.as_deref().unwrap_or("");
    sort_results(&mut in_a_only, sort_field);
    sort_results(&mut in_b_only, sort_field);
    sort_results(&mut same, sort_field);
    sort_results(&mut different, sort_field);
  }

  // Create summary
  let total_a = data_a.len();
  let total_b = data_b.len();
  let count_in_a_only = in_a_only.len();
  let count_in_b_only = in_b_only.len();
  let count_same = same.len();
  let count_different = different.len();

  let match_percentage =
    if total_a + total_b > 0 { (count_same * 2) as f64 / (total_a + total_b) as f64 * 100.0 } else { 0.0 };

  let summary = ComparisonSummary {
    total_a,
    total_b,
    count_in_a_only,
    count_in_b_only,
    count_same,
    count_different,
    match_percentage,
  };

  Ok(ComparisonResults { in_a_only, in_b_only, same, different, summary })
}

pub type TupleMap = (HashMap<String, Value>, HashMap<String, Value>);

/// Create lookup maps for efficient record matching
fn create_lookup_maps(
  data_a: &[Value],
  data_b: &[Value],
  operation: &CompareDatasetsOperation,
) -> Result<TupleMap, String> {
  let mut lookup_a = HashMap::new();
  let mut lookup_b = HashMap::new();

  // Build lookup for dataset A
  for record in data_a {
    let key = extract_key_value(record, operation)?;
    lookup_a.insert(key, record.clone());
  }

  // Build lookup for dataset B
  for record in data_b {
    let key = extract_key_value(record, operation)?;
    lookup_b.insert(key, record.clone());
  }

  Ok((lookup_a, lookup_b))
}

/// Extract key value from a record for matching
fn extract_key_value(record: &Value, operation: &CompareDatasetsOperation) -> Result<String, String> {
  let key_fields: Vec<&FieldMatchConfig> = operation.match_fields.iter().filter(|f| f.is_key_field).collect();

  if key_fields.is_empty() {
    return Err("No key fields defined for matching".to_string());
  }

  let mut key_parts = Vec::new();

  for field_config in key_fields {
    let value = extract_field_value(record, &field_config.field_name)?;
    let normalized = normalize_value_for_comparison(&value, field_config);
    key_parts.push(normalized);
  }

  // Combine key parts with a separator
  Ok(key_parts.join("||"))
}

/// Extract field value from a JSON record using dot notation
pub fn extract_field_value(record: &Value, field_path: &str) -> Result<String, String> {
  let parts: Vec<&str> = field_path.split('.').collect();
  let mut current = record;

  for part in parts {
    match current {
      Value::Object(map) => {
        current = map.get(part).ok_or_else(|| format!("Field '{}' not found in record", part))?;
      }
      _ => return Err(format!("Cannot access field '{}' on non-object value", part)),
    }
  }

  match current {
    Value::String(s) => Ok(s.clone()),
    Value::Number(n) => Ok(n.to_string()),
    Value::Bool(b) => Ok(b.to_string()),
    Value::Null => Ok("".to_string()),
    _ => Ok(current.to_string()),
  }
}

/// Normalize a value for comparison based on field configuration
pub fn normalize_value_for_comparison(value: &str, field_config: &FieldMatchConfig) -> String {
  let mut normalized = value.to_string();

  if field_config.trim_whitespace {
    normalized = normalized.trim().to_string();
  }

  if !field_config.case_sensitive {
    normalized = normalized.to_lowercase();
  }

  normalized
}

/// Compare two records in detail
async fn compare_records(
  record_a: &Value,
  record_b: &Value,
  operation: &CompareDatasetsOperation,
) -> Result<RecordComparisonResult, String> {
  let mut differing_fields = Vec::new();
  let mut match_scores = Vec::new();
  let mut field_differences = HashMap::new();

  // Compare all specified match fields
  for field_config in &operation.match_fields {
    let value_a = extract_field_value(record_a, &field_config.field_name).unwrap_or_default();
    let value_b = extract_field_value(record_b, &field_config.field_name).unwrap_or_default();

    let normalized_a = normalize_value_for_comparison(&value_a, field_config);
    let normalized_b = normalize_value_for_comparison(&value_b, field_config);

    let (is_match, match_score) = compare_field_values(&normalized_a, &normalized_b, field_config);

    match_scores.push(match_score);

    if !is_match {
      differing_fields.push(field_config.field_name.clone());
      field_differences.insert(
        field_config.field_name.clone(),
        json!({
            "value_a": value_a,
            "value_b": value_b,
            "normalized_a": normalized_a,
            "normalized_b": normalized_b,
            "match_score": match_score
        }),
      );
    }
  }

  // Determine overall match result
  let overall_match_score =
    if match_scores.is_empty() { 1.0 } else { match_scores.iter().sum::<f64>() / match_scores.len() as f64 };

  let (result_type, merged_record, match_reason) = if differing_fields.is_empty() {
    (
      ComparisonResultType::Same,
      Some(record_a.clone()), // Records are identical
      Some("All fields match exactly".to_string()),
    )
  } else {
    (
      ComparisonResultType::Different,
      Some(merge_records(record_a, record_b, &field_differences, operation)?),
      Some(format!("{} fields differ", differing_fields.len())),
    )
  };

  Ok(RecordComparisonResult {
    result_type,
    record_a: Some(record_a.clone()),
    record_b: Some(record_b.clone()),
    merged_record,
    differing_fields: if differing_fields.is_empty() { None } else { Some(differing_fields) },
    match_score: Some(overall_match_score),
    match_reason,
  })
}

/// Compare two field values based on configuration
fn compare_field_values(value_a: &str, value_b: &str, field_config: &FieldMatchConfig) -> (bool, f64) {
  match field_config.comparison_mode {
    ComparisonMode::Exact => {
      let is_match = value_a == value_b;
      let score = if is_match { 1.0 } else { 0.0 };
      (is_match, score)
    }
    ComparisonMode::Fuzzy => {
      let threshold = field_config.fuzzy_threshold.unwrap_or(0.8);
      let similarity = calculate_similarity(value_a, value_b);
      let is_match = similarity >= threshold;
      (is_match, similarity)
    }
  }
}

/// Calculate similarity between two strings using character-based similarity
fn calculate_similarity(value_a: &str, value_b: &str) -> f64 {
  if value_a.is_empty() && value_b.is_empty() {
    return 1.0;
  }
  if value_a.is_empty() || value_b.is_empty() {
    return 0.0;
  }

  if value_a == value_b {
    return 1.0;
  }

  let len_a = value_a.chars().count();
  let len_b = value_b.chars().count();

  // Calculate character-level similarity (Jaccard index)
  let a_chars: std::collections::HashSet<char> = value_a.chars().collect();
  let b_chars: std::collections::HashSet<char> = value_b.chars().collect();

  let intersection = a_chars.intersection(&b_chars).count();
  let union = a_chars.union(&b_chars).count();

  let jaccard_similarity = if union == 0 { 0.0 } else { intersection as f64 / union as f64 };

  // Calculate position-based similarity for strings with similar length
  let position_similarity = if len_a == len_b {
    let matches = value_a.chars().zip(value_b.chars()).filter(|(a, b)| a == b).count() as f64;
    matches / len_a as f64
  } else {
    0.0
  };

  // Calculate length similarity penalty
  let length_similarity = 1.0 - (len_a.abs_diff(len_b) as f64 / len_a.max(len_b) as f64);

  // Combine similarities with weights that favor character overlap and position
  (jaccard_similarity * 0.5) + (position_similarity * 0.3) + (length_similarity * 0.2)
}

/// Merge two records based on conflict resolution strategy
fn merge_records(
  record_a: &Value,
  record_b: &Value,
  field_differences: &HashMap<String, Value>,
  operation: &CompareDatasetsOperation,
) -> Result<Value, String> {
  let mut merged = json!({});

  // Start with all fields from record_a
  if let Value::Object(map_a) = record_a {
    for (key, value) in map_a {
      merged[key] = value.clone();
    }
  }

  // Process fields from record_b
  if let Value::Object(map_b) = record_b {
    for (key, value_b) in map_b {
      if field_differences.contains_key(key) {
        // This field differs between records, apply conflict resolution
        let value_a = record_a.get(key);
        let merged_value = apply_conflict_resolution(value_a, Some(value_b), key, &operation.conflict_resolution)?;
        merged[key] = merged_value;
      } else if !merged.as_object().unwrap().contains_key(key) {
        // Field only exists in record_b
        if operation.include_all_fields {
          merged[key] = value_b.clone();
        }
      }
    }
  }

  Ok(merged)
}

/// Apply conflict resolution strategy to a field with different values
fn apply_conflict_resolution(
  value_a: Option<&Value>,
  value_b: Option<&Value>,
  field_name: &str,
  strategy: &ConflictResolution,
) -> Result<Value, String> {
  match strategy {
    ConflictResolution::PreferInputA => Ok(value_a.cloned().unwrap_or_else(|| json!(null))),
    ConflictResolution::PreferInputB => Ok(value_b.cloned().unwrap_or_else(|| json!(null))),
    ConflictResolution::Mix => {
      // For mix strategy, try to intelligently combine values
      match (value_a, value_b) {
        (Some(Value::Array(arr_a)), Some(Value::Array(arr_b))) => {
          // Merge arrays, avoiding duplicates
          let mut merged_arr = arr_a.clone();
          for item in arr_b {
            if !merged_arr.contains(item) {
              merged_arr.push(item.clone());
            }
          }
          Ok(Value::Array(merged_arr))
        }
        (Some(val_a), Some(val_b)) if val_a != val_b => {
          // Different values, create array with both
          Ok(json!([val_a, val_b]))
        }
        _ => {
          // Values are the same or one is null, prefer non-null value
          Ok(value_a.or(value_b).cloned().unwrap_or_else(|| json!(null)))
        }
      }
    }
    ConflictResolution::IncludeBoth => match (value_a, value_b) {
      (Some(val_a), Some(val_b)) => Ok(json!({
          "value_a": val_a,
          "value_b": val_b,
          "field": field_name
      })),
      _ => Ok(value_a.or(value_b).cloned().unwrap_or_else(|| json!(null))),
    },
  }
}

/// Sort results by a specified field
fn sort_results(results: &mut [RecordComparisonResult], sort_field: &str) {
  if sort_field.is_empty() {
    return;
  }

  results.sort_by(|a, b| {
    let value_a = get_sort_value(a, sort_field);
    let value_b = get_sort_value(b, sort_field);
    value_a.cmp(&value_b)
  });
}

/// Get sort value from a record comparison result
fn get_sort_value(result: &RecordComparisonResult, sort_field: &str) -> String {
  // Try to get value from merged record first, then from record_a, then record_b
  let record = result.merged_record.as_ref().or(result.record_a.as_ref()).or(result.record_b.as_ref());

  if let Some(rec) = record
    && let Ok(value) = extract_field_value(rec, sort_field)
  {
    return value;
  }

  // Fallback to empty string for sorting
  String::new()
}

/// Validate that field path follows valid JSON dot notation
#[allow(dead_code)]
pub fn validate_field_path(field_path: &str) -> Result<(), String> {
  if field_path.is_empty() {
    return Err("Field path cannot be empty".to_string());
  }

  // Check for invalid characters
  let invalid_chars_regex = Regex::new(r"[^\w\.\-\[\]]").unwrap();
  if invalid_chars_regex.is_match(field_path) {
    return Err("Field path contains invalid characters".to_string());
  }

  // Check for consecutive dots
  if field_path.contains("..") {
    return Err("Field path cannot contain consecutive dots".to_string());
  }

  // Check if it starts or ends with a dot
  if field_path.starts_with('.') || field_path.ends_with('.') {
    return Err("Field path cannot start or end with a dot".to_string());
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::super::{CompareDatasetsOperation, ComparisonMode, ConflictResolution, FieldMatchConfig};
  use super::*;

  #[test]
  fn test_extract_field_value_simple() {
    let record = json!({
        "id": 123,
        "name": "Alice"
    });

    assert_eq!(extract_field_value(&record, "id").unwrap(), "123");
    assert_eq!(extract_field_value(&record, "name").unwrap(), "Alice");
  }

  #[test]
  fn test_extract_field_value_nested() {
    let record = json!({
        "user": {
            "profile": {
                "email": "alice@example.com"
            }
        }
    });

    assert_eq!(extract_field_value(&record, "user.profile.email").unwrap(), "alice@example.com");
  }

  #[test]
  fn test_extract_field_value_missing() {
    let record = json!({"id": 123});

    assert!(extract_field_value(&record, "name").is_err());
  }

  #[test]
  fn test_normalize_value_for_comparison() {
    let config = FieldMatchConfig {
      field_name: "name".to_string(),
      is_key_field: false,
      comparison_mode: ComparisonMode::Exact,
      fuzzy_threshold: None,
      case_sensitive: false,
      trim_whitespace: true,
    };

    assert_eq!(normalize_value_for_comparison("  Alice  ", &config), "alice");
    assert_eq!(normalize_value_for_comparison("ALICE", &config), "alice");
  }

  #[test]
  fn test_calculate_similarity() {
    assert_eq!(calculate_similarity("hello", "hello"), 1.0);
    assert_eq!(calculate_similarity("hello", ""), 0.0);
    assert_eq!(calculate_similarity("", ""), 1.0);

    // Test similarity between different strings
    let similarity = calculate_similarity("hello", "hallo");
    assert!(similarity > 0.5); // Should be somewhat similar
  }

  #[test]
  fn test_compare_field_values_exact() {
    let config = FieldMatchConfig {
      field_name: "name".to_string(),
      is_key_field: false,
      comparison_mode: ComparisonMode::Exact,
      fuzzy_threshold: None,
      case_sensitive: true,
      trim_whitespace: false,
    };

    let (is_match, score) = compare_field_values("Alice", "Alice", &config);
    assert!(is_match);
    assert_eq!(score, 1.0);

    let (is_match, score) = compare_field_values("Alice", "alice", &config);
    assert!(!is_match);
    assert_eq!(score, 0.0);
  }

  #[test]
  fn test_compare_field_values_fuzzy() {
    let config = FieldMatchConfig {
      field_name: "name".to_string(),
      is_key_field: false,
      comparison_mode: ComparisonMode::Fuzzy,
      fuzzy_threshold: Some(0.7), // Lower threshold to be more realistic
      case_sensitive: true,
      trim_whitespace: false,
    };

    let (is_match, score) = compare_field_values("hello", "hallo", &config);
    assert!(is_match, "Expected hello and hallo to be similar with score {}", score);
    assert!(score >= 0.7, "Expected score >= 0.7, got {}", score);

    let (is_match, score) = compare_field_values("hello", "world", &config);
    assert!(!is_match);
    assert!(score < 0.7);
  }

  #[test]
  fn test_validate_field_path() {
    assert!(validate_field_path("name").is_ok());
    assert!(validate_field_path("user.profile.email").is_ok());
    assert!(validate_field_path("").is_err());
    assert!(validate_field_path(".name").is_err());
    assert!(validate_field_path("name.").is_err());
    assert!(validate_field_path("name..profile").is_err());
    assert!(validate_field_path("name@profile").is_err());
  }

  #[test]
  fn test_apply_conflict_resolution_prefer_a() {
    let value_a = Some(&json!("Alice"));
    let value_b = Some(&json!("Bob"));

    let result = apply_conflict_resolution(value_a, value_b, "name", &ConflictResolution::PreferInputA).unwrap();
    assert_eq!(result, "Alice");
  }

  #[test]
  fn test_apply_conflict_resolution_prefer_b() {
    let value_a = Some(&json!("Alice"));
    let value_b = Some(&json!("Bob"));

    let result = apply_conflict_resolution(value_a, value_b, "name", &ConflictResolution::PreferInputB).unwrap();
    assert_eq!(result, "Bob");
  }

  #[test]
  fn test_apply_conflict_resolution_include_both() {
    let value_a = Some(&json!("Alice"));
    let value_b = Some(&json!("Bob"));

    let result = apply_conflict_resolution(value_a, value_b, "name", &ConflictResolution::IncludeBoth).unwrap();

    assert_eq!(result["value_a"], "Alice");
    assert_eq!(result["value_b"], "Bob");
    assert_eq!(result["field"], "name");
  }

  #[tokio::test]
  async fn test_compare_datasets_basic() {
    let data_a = vec![json!({"id": 1, "name": "Alice"}), json!({"id": 2, "name": "Bob"})];

    let data_b = vec![json!({"id": 1, "name": "Alice"}), json!({"id": 3, "name": "Charlie"})];

    let operation = CompareDatasetsOperation {
      match_fields: vec![FieldMatchConfig {
        field_name: "id".to_string(),
        is_key_field: true,
        comparison_mode: ComparisonMode::Exact,
        fuzzy_threshold: None,
        case_sensitive: true,
        trim_whitespace: true,
      }],
      ..Default::default()
    };

    let results = compare_datasets(&data_a, &data_b, &operation).await.unwrap();

    assert_eq!(results.in_a_only.len(), 1); // Bob
    assert_eq!(results.in_b_only.len(), 1); // Charlie
    assert_eq!(results.same.len(), 1); // Alice
    assert_eq!(results.different.len(), 0);

    assert_eq!(results.summary.total_a, 2);
    assert_eq!(results.summary.total_b, 2);
    assert_eq!(results.summary.count_same, 1);
  }
}
