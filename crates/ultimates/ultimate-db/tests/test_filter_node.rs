#![allow(unused)] // For early development.
#![cfg(feature = "with-sea-query")]

use sea_query::{ColumnRef, ConditionExpression};
use std::sync::Arc;
use ultimate_db::modql::filter::{
  FilterNode, FilterNodeOptions, IntoSeaError, OpValInt32, OpValValue, SeaResult, ToSeaConditionFnHolder,
};

#[test]
fn test_filter_node_with_sea_condition() {
  let special_to_sea_cond = ToSeaConditionFnHolder::new(special_to_sea_condition); // This should implement IntoSeaCondition

  let node = FilterNode {
    rel: None,
    name: "some_name".to_string(),
    opvals: vec![123.into()],
    options: FilterNodeOptions::default(),
    for_sea_condition: Some(special_to_sea_cond.into()),
  };
}

pub fn special_to_sea_condition(col: &ColumnRef, op_val: OpValValue) -> SeaResult<ConditionExpression> {
  todo!()
}
