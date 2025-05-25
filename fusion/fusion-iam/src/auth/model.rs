use modelsql::filter::OpValString;

use crate::{pb::fusion_iam::v1::SigninRequest, user::UserFilter};

// #[derive(FilterNodes)]
// pub struct LoginFilter {
//   pub email: Option<OpValsString>,
//   pub phone: Option<OpValsString>,
// }

// impl From<&SigninReq> for LoginFilter {
//   fn from(req: &SigninReq) -> Self {
//     Self {
//       email: req.email.as_deref().map(|s| OpValString::Eq(s.to_string()).into()),
//       phone: req.phone.as_deref().map(|s| OpValString::Eq(s.to_string()).into()),
//     }
//   }
// }

// #[derive(Deserialize)]
// pub struct SigninReq {
//   pub email: Option<String>,
//   pub phone: Option<String>,
//   #[serde(skip_serializing)]
//   pub password: String,
// }

// impl From<&SigninReq> for UserFilter {
//   fn from(value: &SigninReq) -> Self {
//     UserFilter {
//       email: value.email.as_deref().map(|s| OpValString::Eq(s.to_string()).into()),
//       phone: value.phone.as_deref().map(|s| OpValString::Eq(s.to_string()).into()),
//       ..Default::default()
//     }
//   }
// }

impl From<&SigninRequest> for UserFilter {
  fn from(value: &SigninRequest) -> Self {
    UserFilter {
      email: value.email.as_deref().map(|s| OpValString::Eq(s.to_string()).into()),
      phone: value.phone.as_deref().map(|s| OpValString::Eq(s.to_string()).into()),
      ..Default::default()
    }
  }
}
