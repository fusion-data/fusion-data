use axum::{
  Router,
  extract::Path,
  response::Json,
  routing::{get, post},
};
use fusion_common::model::IdUuidResult;
use fusion_core::application::Application;
use fusion_web::{WebResult, ok_json};
use fusionsql::page::PageResult;
use uuid::Uuid;

use crate::domain::credential::{
  CredentialEntity, CredentialForInsert, CredentialForQuery, CredentialForUpdate, CredentialSvc,
  CredentialVerifyResult, CredentialWithDecryptedData, VerifyCredentialRequest,
};

pub fn routes() -> Router<Application> {
  Router::new()
    .route("/query", post(query_credentials))
    .route("/", post(create_credential))
    .route("/verify", post(verify_credential))
    .route("/{id}", get(get_credential).put(update_credential).delete(delete_credential))
    .route("/{id}/verify", post(verify_stored_credential))
}

/// 创建凭证
pub async fn create_credential(
  credential_svc: CredentialSvc,
  Json(input): Json<CredentialForInsert>,
) -> WebResult<IdUuidResult> {
  let id = credential_svc.create_credential(input).await?;

  ok_json!(IdUuidResult::new(id))
}

/// 查询凭证列表
pub async fn query_credentials(
  credential_svc: CredentialSvc,
  Json(input): Json<CredentialForQuery>,
) -> WebResult<PageResult<CredentialEntity>> {
  let result = credential_svc.query_credentials(input).await?;
  ok_json!(result)
}

/// 获取凭证详情
pub async fn get_credential(
  credential_svc: CredentialSvc,
  Path(id): Path<Uuid>,
) -> WebResult<CredentialWithDecryptedData> {
  let credential = credential_svc.get_credential(&id).await?;
  ok_json!(credential)
}

/// 更新凭证
pub async fn update_credential(
  credential_svc: CredentialSvc,
  Path(id): Path<Uuid>,
  Json(input): Json<CredentialForUpdate>,
) -> WebResult<()> {
  credential_svc.update_credential(&id, input).await?;
  ok_json!()
}

/// 删除凭证
pub async fn delete_credential(credential_svc: CredentialSvc, Path(id): Path<Uuid>) -> WebResult<()> {
  credential_svc.delete_credential(&id).await?;
  ok_json!()
}

/// 验证未保存的凭证
pub async fn verify_credential(
  credential_svc: CredentialSvc,
  Json(input): Json<VerifyCredentialRequest>,
) -> WebResult<CredentialVerifyResult> {
  let result = credential_svc.test_credential(&input.data, &input.kind).await?;
  ok_json!(result)
}

/// 验证已保存的凭证
pub async fn verify_stored_credential(
  credential_svc: CredentialSvc,
  Path(id): Path<Uuid>,
) -> WebResult<CredentialVerifyResult> {
  let credential = credential_svc.get_credential(&id).await?;
  let result = credential_svc.test_credential(&credential.decrypted_data, &credential.credential.kind).await?;
  ok_json!(result)
}
