use fusion_common::ahash::HashMap;

use super::{NodeConnectionKind, ExecutionDataItems, ExecutionDataMap};

pub fn make_execution_data_map<T>(iter: T) -> ExecutionDataMap
where
  T: IntoIterator<Item = (NodeConnectionKind, Vec<ExecutionDataItems>)>,
{
  let mut map = HashMap::default();
  for (conn_kind, items) in iter {
    map.insert(conn_kind, items);
  }
  map
}
