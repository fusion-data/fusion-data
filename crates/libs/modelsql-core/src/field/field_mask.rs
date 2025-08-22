use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldMask {
  pub paths: Vec<String>,
}

impl FieldMask {
  pub fn new<I, S>(paths: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self { paths: paths.into_iter().map(|s| s.into()).collect() }
  }

  pub fn is_empty(&self) -> bool {
    self.paths.is_empty()
  }

  #[inline]
  pub fn non_empty(&self) -> bool {
    !self.is_empty()
  }

  /// Returns true if not set `self.paths` or the path is in the `self.paths`
  pub fn hit(&self, path: &str) -> bool {
    self.is_empty() || self.paths.iter().any(|p| p == path)
  }
}
