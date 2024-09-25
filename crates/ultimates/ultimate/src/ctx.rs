use std::{ops::Deref, sync::Arc};

use josekit::jwt::JwtPayload;
use ultimate_common::time::{self, DateTime, Duration, Utc, UtcDateTime};

use crate::DataError;

#[derive(Debug, Default)]
pub struct InnerCtx {
  /// 会话用户 ID
  uid: i64,

  /// 请求时时间
  req_time: DateTime<Utc>,

  /// 会话过期时间
  expires_at: DateTime<Utc>,

  /// 组织ID列表。用于需要通过不同组织身份进行细粒度权限控制
  ext_orgs: Vec<i64>,

  /// 角色ID列表。用于需要通过不同角色身份进行细粒度权限控制
  ext_roles: Vec<i64>,

  /// 权限ID列表。用于需要通过不同权限ID进行细粒度权限控制
  ext_privileges: Vec<i64>,
}

/// 会话上下文。
/// 此处 clone 的成本很低，若后续数据多的话可以使用 Arc 加 Wrapper 模式来降低数据复制的成本
#[derive(Clone, Debug, Default)]
pub struct Ctx(Arc<InnerCtx>);

impl Ctx {
  pub fn new(uid: i64, req_time: UtcDateTime, expires_at: UtcDateTime) -> Self {
    Self(Arc::new(InnerCtx { uid, req_time, expires_at, ..Default::default() }))
  }

  pub fn new_root() -> Self {
    let req_time = time::now_utc();
    let expires_at = req_time + Duration::minutes(30);
    Self::new(0, req_time, expires_at)
  }

  pub fn new_super_admin() -> Self {
    let req_time = time::now_utc();
    let expires_at = req_time + Duration::minutes(30);
    Self::new(1, req_time, expires_at)
  }

  pub fn uid(&self) -> i64 {
    self.uid
  }

  pub fn req_time(&self) -> &UtcDateTime {
    &self.req_time
  }

  pub fn expires_at(&self) -> &UtcDateTime {
    &self.expires_at
  }

  // pub fn with_expires_at(mut self, expires_at: UtcDateTime) -> Self {
  //   self.expires_at = expires_at;
  //   self
  // }

  pub fn ext_orgs(&self) -> &[i64] {
    &self.ext_orgs
  }

  // pub fn with_ext_orgs(mut self, ext_orgs: Vec<i64>) -> Self {
  //   self.ext_orgs = ext_orgs;
  //   self
  // }

  pub fn ext_roles(&self) -> &[i64] {
    &self.ext_roles
  }

  // pub fn with_ext_roles(mut self, ext_roles: Vec<i64>) -> Self {
  //   self.0.ext_roles = ext_roles;
  //   self
  // }

  pub fn ext_privileges(&self) -> &[i64] {
    &self.ext_privileges
  }

  // pub fn with_ext_privileges(mut self, ext_privileges: Vec<i64>) -> Self {
  //   self.ext_privileges = ext_privileges;
  //   self
  // }

  pub fn try_from_jwt_payload(payload: &JwtPayload, req_time: Option<UtcDateTime>) -> Result<Self, DataError> {
    let req_time = req_time.unwrap_or_else(time::now_utc);

    let sub = payload.subject().ok_or_else(|| DataError::unauthorized("'sub' of jwt missing"))?;

    let uid: i64 = sub.parse().map_err(|_| DataError::unauthorized(format!("<sub:{sub}> invalid")))?;

    let expires_at: UtcDateTime = if let Some(st) = payload.expires_at() {
      let expires_at = st.into();
      if expires_at < time::now_utc() {
        return Err(DataError::unauthorized("The token expired"));
      }
      expires_at
    } else {
      UtcDateTime::MAX_UTC
    };

    Ok(Ctx::new(uid, req_time, expires_at))
  }
}

impl Deref for Ctx {
  type Target = InnerCtx;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl TryFrom<JwtPayload> for Ctx {
  type Error = DataError;

  fn try_from(payload: JwtPayload) -> std::result::Result<Self, Self::Error> {
    Ctx::try_from_jwt_payload(&payload, Some(time::now_utc()))
  }
}
