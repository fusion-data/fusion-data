use fusion_common::ahash::HashMap;

use super::{ConnectionKind, ExecutionDataItems, ExecutionDataMap};

pub fn make_execution_data_map<T>(iter: T) -> ExecutionDataMap
where
  T: IntoIterator<Item = (ConnectionKind, Vec<ExecutionDataItems>)>,
{
  let mut map = HashMap::default();
  for (conn_kind, items) in iter {
    map.insert(conn_kind, items);
  }
  map
}
