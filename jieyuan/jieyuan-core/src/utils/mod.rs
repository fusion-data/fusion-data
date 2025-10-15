#[cfg(feature = "with-openapi")]
pub fn semver_version_schema() -> utoipa::openapi::schema::Schema {
  utoipa::openapi::schema::ObjectBuilder::new()
    .schema_type(utoipa::openapi::schema::Type::String)
    .format(Some(utoipa::openapi::schema::SchemaFormat::Custom("semver".to_string())))
    .description(Some("Semantic version string"))
    .examples([serde_json::json!("1.2.3")])
    .into()
}
