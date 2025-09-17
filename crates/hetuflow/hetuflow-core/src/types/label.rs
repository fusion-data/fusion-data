use std::ops::{Deref, DerefMut};

use fusion_common::ahash::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
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

impl<I, S> From<I> for Labels
where
  I: IntoIterator<Item = (S, S)>,
  S: Into<String>,
{
  fn from(value: I) -> Self {
    let labels = value.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
    Self(labels)
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

  pub fn append_label(&mut self, label: &str, value: &str) -> Option<String> {
    self.0.insert(label.to_string(), value.to_string())
  }

  pub fn append_labels<T, S>(&mut self, labels: T) -> Vec<Option<String>>
  where
    T: IntoIterator<Item = (&'static str, S)>,
    S: AsRef<str>,
  {
    labels.into_iter().map(|(k, v)| self.append_label(k, v.as_ref())).collect()
  }

  pub fn into_inner(self) -> HashMap<String, String> {
    self.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_labels() {
    let labels = Labels::from([("os", "linux"), ("arch", "x86_64")]);
    let json_str = serde_json::to_string(&labels).unwrap();
    println!("json_str: {}", json_str);
    assert!(labels.match_label("os", "linux"));
    assert!(labels.match_label("arch", "x86_64"));
    assert!(!labels.match_label("gpu", "enabled"));
  }
}
