//! Should compile. No test functions yet.

use fusionsql::filter::{FilterNode, IntoFilterNodes, OpVal, OpValInt64, OpValString};

pub struct ProjectFilter {
  id: Option<OpValInt64>,
  name: Option<OpValString>,
}

impl IntoFilterNodes for ProjectFilter {
  fn filter_nodes(self, rel: Option<String>) -> Vec<FilterNode> {
    let mut nodes = Vec::new();

    if let Some(id) = self.id {
      let node = FilterNode::new_with_rel(rel.clone(), "id", id);
      nodes.push(node)
    }

    if let Some(name) = self.name {
      let node = FilterNode::new_with_rel(rel, "name", name);
      nodes.push(node)
    }

    nodes
  }
}

#[allow(unused)]
pub struct TaskFilter {
  project: Option<ProjectFilter>,
  title: Option<OpValString>,
  kind: Option<OpValString>,
}

impl IntoFilterNodes for TaskFilter {
  fn filter_nodes(self, rel: Option<String>) -> Vec<FilterNode> {
    let mut nodes = Vec::new();

    if let Some(title) = self.title {
      let node = FilterNode::new_with_rel(rel, "title", title);
      nodes.push(node)
    }

    nodes
  }
}
