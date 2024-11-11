//! Should compile. No test functions yet.

use ultimate_db::modql::filter::{FilterNode, IntoFilterNodes, OpVal, OpValInt64, OpValString, OpValsString};

pub struct ProjectFilter {
  id: Option<Vec<OpValInt64>>,
  name: Option<Vec<OpValString>>,
}

impl IntoFilterNodes for ProjectFilter {
  fn filter_nodes(self, rel: Option<String>) -> Vec<FilterNode> {
    let mut nodes = Vec::new();

    if let Some(id) = self.id {
      let node = FilterNode::new_with_rel(
        rel.clone(),
        "id".to_string(),
        id.into_iter().map(|n| n.into()).collect::<Vec<OpVal>>(),
      );
      nodes.push(node)
    }

    if let Some(name) = self.name {
      let node =
        FilterNode::new_with_rel(rel, "name".to_string(), name.into_iter().map(|n| n.into()).collect::<Vec<OpVal>>());
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
      let node = FilterNode::new_with_rel(
        rel,
        "title".to_string(),
        title.0.into_iter().map(|n| n.into()).collect::<Vec<OpVal>>(),
      );
      nodes.push(node)
    }

    nodes
  }
}
