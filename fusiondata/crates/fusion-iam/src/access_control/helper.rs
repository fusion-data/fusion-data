use ultimate::DataError;

use crate::pb::fusion_iam::v1::PolicyStatementDto;

use super::Policy;

impl TryFrom<Policy> for PolicyStatementDto {
  type Error = DataError;

  fn try_from(value: Policy) -> Result<Self, Self::Error> {
    Ok(Self {
      id: value.id.to_string(),
      description: value.description,
      policy: serde_json::to_string(&value.policy)?,
      status: value.status,
      cid: value.cid,
      ctime: value.ctime.timestamp_millis(),
      mid: value.mid,
      mtime: value.mtime.map(|t| t.timestamp_millis()),
    })
  }
}
