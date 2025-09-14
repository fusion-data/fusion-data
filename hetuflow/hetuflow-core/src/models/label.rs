use std::ops::{Deref, DerefMut};

use fusion_common::ahash::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Labels(HashMap<String, String>);

impl Deref for Labels {
  type Target = HashMap<String, String>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Labels {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl Labels {
  pub fn new(labels: HashMap<String, String>) -> Self {
    Self(labels)
  }

  /// 检查任务是否匹配指定的标签
  pub fn match_label(&self, label: &str, value: &str) -> bool {
    self.0.get(label).is_some_and(|v| v == value)
  }
}
