//! Should compile. No test functions yet.

use fusion_sql::filter::{FilterNode, IntoFilterNodes, OpVal, OpValsInt64, OpValsString};

pub struct ProjectFilter {
  id: Option<OpValsInt64>,
  name: Option<OpValsString>,
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
  title: Option<OpValsString>,
  kind: Option<OpValsString>,
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
