use modelsql::{field::Fields, postgres::PgRowType};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Webhook实体表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "webhook_entity")]
pub struct WebhookEntity {
  pub webhook_path: String,
  pub method: String,
  pub node: String,
  pub webhook_id: Option<String>,
  pub path_length: Option<i32>,
  pub workflow_id: String,
}
impl PgRowType for WebhookEntity {}
