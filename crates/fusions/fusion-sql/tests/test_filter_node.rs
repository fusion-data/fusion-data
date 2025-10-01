#![allow(unused)] // For early development.

use fusion_sql::filter::{
  FilterNode, FilterNodeOptions, IntoSeaError, OpValsInt32, OpValsValue, SeaResult, ToSeaConditionFnHolder,
};
use sea_query::{ColumnRef, ConditionExpression};
use std::sync::Arc;

#[test]
fn test_filter_node_with_sea_condition() {
  let special_to_sea_cond = ToSeaConditionFnHolder::new(special_to_sea_condition); // This should implement IntoSeaCondition

  let node = FilterNode {
    rel: None,
    name: "some_name".to_string(),
    opvals: OpValsInt32::eq(123).into(),
    options: FilterNodeOptions::default(),
    for_sea_condition: Some(special_to_sea_cond.into()),
  };
}

pub fn special_to_sea_condition(col: &ColumnRef, op_val: OpValsValue) -> SeaResult<Vec<ConditionExpression>> {
  todo!()
}
