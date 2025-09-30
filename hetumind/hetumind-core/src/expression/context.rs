use std::sync::Arc;

use fusion_common::ahash::HashMap;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::workflow::{Execution, Workflow};

use super::Value;

/// 执行上下文，包含工作流、执行和环境信息
#[derive(Debug, Clone, TypedBuilder)]
pub struct ExpressionExecutionContext {
  /// 工作流元数据
  pub workflow: Arc<Workflow>,
  /// 执行元数据
  pub execution: Arc<Execution>,
  /// 环境变量
  #[builder(default)]
  pub env: HashMap<String, String>,
  /// HTTP 分页信息（如果有）
  #[builder(default, setter(strip_option))]
  pub http_pagination: Option<HttpPagination>,
  /// 自定义变量
  #[builder(default)]
  pub vars: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpPagination {
  pub page: i32,
  pub total: i32,
  pub per_page: i32,
  pub has_next: bool,
}

impl ExpressionExecutionContext {
  pub fn set_var(&mut self, key: impl Into<String>, value: Value) {
    self.vars.insert(key.into(), value);
  }

  pub fn get_var(&self, key: impl AsRef<str>) -> Option<&Value> {
    self.vars.get(key.as_ref())
  }

  pub fn set_http_pagination(&mut self, pagination: HttpPagination) {
    self.http_pagination = Some(pagination);
  }
}
