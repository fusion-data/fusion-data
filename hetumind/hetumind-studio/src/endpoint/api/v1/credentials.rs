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
use hetumind_core::workflow::WorkflowId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::credential::{
  CredentialEntity, CredentialForInsert, CredentialForQuery, CredentialForUpdate, CredentialSvc,
  CredentialVerifyResult, CredentialWithDecryptedData, VerifyCredentialRequest,
};

// --- API Data Models ---

/// 凭据引用信息
#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialReference {
  /// 引用的工作流ID
  pub workflow_id: WorkflowId,
  /// 工作流名称
  pub workflow_name: String,
  /// 引用的节点名称（可选）
  pub node_name: Option<String>,
  /// 引用类型
  pub reference_type: String,
}

/// 凭据引用响应
#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialReferencesResponse {
  /// 凭据ID
  pub credential_id: Uuid,
  /// 凭据名称
  pub credential_name: String,
  /// 引用列表
  pub references: Vec<CredentialReference>,
  /// 总引用数
  pub total_references: usize,
}

pub fn routes() -> Router<Application> {
  Router::new()
    .route("/query", post(query_credentials))
    .route("/", post(create_credential))
    .route("/verify", post(verify_credential))
    .route("/{id}", get(get_credential).put(update_credential).delete(delete_credential))
    .route("/{id}/verify", post(verify_stored_credential))
    .route("/{id}/references", get(get_credential_references))
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

/// 查询凭据引用
pub async fn get_credential_references(
  credential_svc: CredentialSvc,
  Path(id): Path<Uuid>,
) -> WebResult<CredentialReferencesResponse> {
  // 获取凭据信息
  let credential = credential_svc.get_credential(&id).await?;

  // TODO: 实现实际的引用查询逻辑
  // 这里需要查询工作流定义，找出哪些工作流和节点使用了这个凭据
  // 目前返回一个空的引用列表作为占位符

  let references_response = CredentialReferencesResponse {
    credential_id: id,
    credential_name: credential.credential.name,
    references: vec![], // TODO: 实现实际的引用查询
    total_references: 0,
  };

  ok_json!(references_response)
}
