use std::sync::Arc;

use fusion_common::ahash::HashMap;
use serde::{Deserialize, Serialize};

use crate::workflow::{Execution, Workflow};

use super::Value;

/// 执行上下文，包含工作流、执行和环境信息
#[derive(Debug, Clone)]
pub struct ExpressionExecutionContext {
  /// 工作流元数据
  pub workflow: Arc<Workflow>,
  /// 执行元数据
  pub execution: Arc<Execution>,
  /// 环境变量
  pub env: HashMap<String, String>,
  /// HTTP 分页信息（如果有）
  pub http_pagination: Option<HttpPagination>,
  /// 自定义变量
  pub vars: HashMap<String, Value>,
}

impl ExpressionExecutionContext {
  pub fn new(workflow: Arc<Workflow>, execution: Arc<Execution>) -> Self {
    Self { workflow, execution, env: HashMap::default(), http_pagination: None, vars: HashMap::default() }
  }

  pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
    self.env = env;
    self
  }

  pub fn with_http_pagination(mut self, http_pagination: HttpPagination) -> Self {
    self.http_pagination = Some(http_pagination);
    self
  }

  pub fn with_vars(mut self, vars: HashMap<String, Value>) -> Self {
    self.vars = vars;
    self
  }

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpPagination {
  pub page: i32,
  pub total: i32,
  pub per_page: i32,
  pub has_next: bool,
}
