use log::{debug, info, warn};

use hetumind_core::workflow::ExecutionData;

use super::LimitConfig;

/// 应用限制操作到数据项
///
/// 这个函数实现了 Limit 节点的核心逻辑，基于 Rust 的切片操作
/// 提供高性能的数据量控制功能。
///
/// # 算法复杂度
/// - 时间复杂度: O(k)，其中 k 是要保留的数据项数量
/// - 空间复杂度: O(k)，创建新的数组包含保留的数据项
///
/// # 参数
/// - `input_items`: 输入的数据项列表
/// - `config`: 限制配置
///
/// # 返回值
/// 返回应用限制后的数据项列表
#[allow(dead_code)]
pub fn apply_limit_operation(input_items: &[ExecutionData], config: &LimitConfig) -> Vec<ExecutionData> {
  debug!(
    "应用限制操作 - 输入项目数: {}, 限制数: {}, 策略: {:?}",
    input_items.len(),
    config.max_items,
    config.keep_strategy
  );

  // 早期退出优化：如果不需要限制，直接返回原数据
  if input_items.len() <= config.max_items {
    debug!("无需限制，返回所有输入项目");
    return input_items.to_vec();
  }

  // 根据策略应用不同的限制操作
  let result = match config.keep_strategy {
    super::KeepStrategy::FirstItems => {
      // 保留前 N 个项目
      debug!("保留前 {} 个项目", config.max_items);
      input_items[..config.max_items].to_vec()
    }
    super::KeepStrategy::LastItems => {
      // 保留后 N 个项目
      let start_index = input_items.len().saturating_sub(config.max_items);
      debug!("保留后 {} 个项目 (从索引 {} 开始)", config.max_items, start_index);
      input_items[start_index..].to_vec()
    }
  };

  info!("限制操作完成 - 输入: {} 项, 输出: {} 项, 策略: {:?}", input_items.len(), result.len(), config.keep_strategy);

  result
}

/// 计算限制统计信息
#[allow(dead_code)]
pub struct LimitStats {
  /// 输入项目数量
  pub input_count: usize,
  /// 输出项目数量
  pub output_count: usize,
  /// 被移除的项目数量
  pub removed_count: usize,
  /// 保留策略
  pub keep_strategy: super::KeepStrategy,
  /// 最大限制数量
  pub max_items: usize,
}

impl LimitStats {
  /// 计算限制统计信息
  #[allow(dead_code)]
  pub fn new(input_items: &[ExecutionData], output_items: &[ExecutionData], config: &LimitConfig) -> Self {
    Self {
      input_count: input_items.len(),
      output_count: output_items.len(),
      removed_count: input_items.len().saturating_sub(output_items.len()),
      keep_strategy: config.keep_strategy.clone(),
      max_items: config.max_items,
    }
  }

  /// 获取限制比例 (0.0 - 1.0)
  #[allow(dead_code)]
  pub fn limit_ratio(&self) -> f64 {
    if self.input_count == 0 { 0.0 } else { self.output_count as f64 / self.input_count as f64 }
  }

  /// 是否应用了限制
  #[allow(dead_code)]
  pub fn is_limited(&self) -> bool {
    self.removed_count > 0
  }

  /// 记录统计信息到日志
  #[allow(dead_code)]
  pub fn log_stats(&self) {
    if self.is_limited() {
      warn!(
        "Limit 统计 - 输入: {} 项, 输出: {} 项, 移除: {} 项 ({:.1}% 保留), 策略: {:?}",
        self.input_count,
        self.output_count,
        self.removed_count,
        self.limit_ratio() * 100.0,
        self.keep_strategy
      );
    } else {
      info!(
        "Limit 统计 - 输入: {} 项, 输出: {} 项 (无限制), 策略: {:?}",
        self.input_count, self.output_count, self.keep_strategy
      );
    }
  }
}

/// 验证限制参数的有效性
#[allow(dead_code)]
pub fn validate_limit_parameters(max_items: usize) -> Result<(), String> {
  if max_items == 0 {
    return Err("max_items must be greater than 0".to_string());
  }

  if max_items > 100_000 {
    return Err("max_items is too large (maximum: 100000)".to_string());
  }

  Ok(())
}

/// 计算最优的批处理大小
///
/// 在大数据集场景下，这个函数可以帮助确定合适的批处理大小
/// 以平衡内存使用和处理效率。
#[allow(dead_code)]
pub fn calculate_optimal_batch_size(total_items: usize, max_items: usize) -> usize {
  // 如果总项目数小于等于最大限制，直接返回总项目数
  if total_items <= max_items {
    return total_items;
  }

  // 计算批处理大小，确保不会超过内存限制
  // 这里使用启发式算法：每批最多处理 1000 项，但不超过总项目的 10%
  let batch_size = (total_items / 10).clamp(1, 1000);

  // 确保批处理大小不会超过最大限制
  std::cmp::min(batch_size, max_items)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::core::limit_node::KeepStrategy;

  fn create_test_data(count: usize) -> Vec<ExecutionData> {
    (0..count)
      .map(|i| {
        hetumind_core::workflow::ExecutionData::new_json(
          serde_json::json!({"id": i, "value": format!("item_{}", i)}),
          None,
        )
      })
      .collect()
  }

  #[test]
  fn test_apply_limit_first_items() {
    let input = create_test_data(10);
    let config = LimitConfig { max_items: 3, keep_strategy: KeepStrategy::FirstItems, warn_on_limit: false };

    let result = apply_limit_operation(&input, &config);

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].json()["id"], 0);
    assert_eq!(result[2].json()["id"], 2);
  }

  #[test]
  fn test_apply_limit_last_items() {
    let input = create_test_data(10);
    let config = LimitConfig { max_items: 3, keep_strategy: KeepStrategy::LastItems, warn_on_limit: false };

    let result = apply_limit_operation(&input, &config);

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].json()["id"], 7);
    assert_eq!(result[2].json()["id"], 9);
  }

  #[test]
  fn test_apply_limit_no_limit_needed() {
    let input = create_test_data(3);
    let config = LimitConfig { max_items: 5, keep_strategy: KeepStrategy::FirstItems, warn_on_limit: false };

    let result = apply_limit_operation(&input, &config);

    assert_eq!(result.len(), 3);
    assert_eq!(result.len(), input.len());
  }

  #[test]
  fn test_limit_stats() {
    let input = create_test_data(10);
    let config = LimitConfig { max_items: 3, keep_strategy: KeepStrategy::FirstItems, warn_on_limit: false };

    let result = apply_limit_operation(&input, &config);
    let stats = LimitStats::new(&input, &result, &config);

    assert_eq!(stats.input_count, 10);
    assert_eq!(stats.output_count, 3);
    assert_eq!(stats.removed_count, 7);
    assert!(stats.is_limited());
    assert!((stats.limit_ratio() - 0.3).abs() < f64::EPSILON);
  }

  #[test]
  fn test_validate_limit_parameters() {
    // 有效参数
    assert!(validate_limit_parameters(1).is_ok());
    assert!(validate_limit_parameters(1000).is_ok());

    // 无效参数
    assert!(validate_limit_parameters(0).is_err());
    assert!(validate_limit_parameters(200_000).is_err());
  }

  #[test]
  fn test_calculate_optimal_batch_size() {
    // 小数据集 - 超过限制，使用启发式算法：min(1000, 100/10) = 10，然后 min(10, 50) = 10
    assert_eq!(calculate_optimal_batch_size(100, 50), 10);

    // 中等数据集 - 超过限制，使用启发式算法：min(1000, 1000/10) = 100，然后 min(100, 100) = 100
    assert_eq!(calculate_optimal_batch_size(1000, 100), 100);

    // 大数据集 - 超过限制，使用启发式算法：min(1000, 10000/10) = 1000，然后 min(1000, 1000) = 1000
    assert_eq!(calculate_optimal_batch_size(10_000, 1000), 1000);

    // 测试不超过限制的情况
    assert_eq!(calculate_optimal_batch_size(30, 50), 30);
  }
}
