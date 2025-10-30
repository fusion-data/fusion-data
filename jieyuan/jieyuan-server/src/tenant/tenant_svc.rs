use axum::extract::FromRequestParts;
use fusions::core::DataError;
use fusionsql::page::PageResult;

use jieyuan_core::model::{Tenant, TenantForCreate, TenantForPage, TenantForUpdate, TenantStatus};

use super::TenantBmc;

#[derive(Clone)]
pub struct TenantSvc {
  mm: fusionsql::ModelManager,
}

impl TenantSvc {
  /// Create new tenant service instance
  pub fn new(mm: fusionsql::ModelManager) -> Self {
    Self { mm }
  }

  /// Create a new tenant
  pub async fn create(&self, input: TenantForCreate) -> Result<i64, DataError> {
    // Validate input
    self.validate_create_input(&input).await?;

    // Create tenant
    let id = TenantBmc::create(&self.mm, input).await?;

    Ok(id)
  }

  /// Get tenant by ID
  pub async fn get(&self, id: i64) -> Result<Option<Tenant>, DataError> {
    let entity = TenantBmc::get_by_id(&self.mm, id).await?;
    Ok(entity)
  }

  /// Get active tenant by ID
  pub async fn get_active(&self, id: i64) -> Result<Option<Tenant>, DataError> {
    let entity = TenantBmc::get_active(&self.mm, id).await?;
    Ok(entity)
  }

  /// Get tenant by name (only active tenants)
  pub async fn get_by_name(&self, name: &str) -> Result<Option<Tenant>, DataError> {
    let entity = TenantBmc::get_by_name(&self.mm, name).await?;
    Ok(entity)
  }

  /// Update tenant
  pub async fn update(&self, id: i64, input: TenantForUpdate) -> Result<(), DataError> {
    // Get existing tenant
    let existing = TenantBmc::get_by_id(&self.mm, id)
      .await?
      .ok_or(DataError::not_found(format!("Tenant with id {} not found", id)))?;

    // If updating name, check uniqueness
    if let Some(ref name) = input.name
      && name != &existing.name
    {
      let exists = TenantBmc::name_exists_excluding_id(&self.mm, name, id).await?;
      if exists {
        return Err(DataError::conflicted(format!("Tenant with name '{}' already exists", name)));
      }
    }

    // Update tenant
    TenantBmc::update_by_id(&self.mm, id, input).await?;

    Ok(())
  }

  /// Enable tenant (set status to Active)
  pub async fn enable(&self, id: i64) -> Result<(), DataError> {
    let update_input = TenantForUpdate { status: Some(TenantStatus::Active), ..Default::default() };
    TenantBmc::update_by_id(&self.mm, id, update_input).await?;
    Ok(())
  }

  /// Disable tenant (set status to Inactive)
  pub async fn disable(&self, id: i64) -> Result<(), DataError> {
    let update_input = TenantForUpdate { status: Some(TenantStatus::Inactive), ..Default::default() };
    TenantBmc::update_by_id(&self.mm, id, update_input).await?;
    Ok(())
  }

  /// Get paginated list of tenants
  pub async fn page(&self, req: TenantForPage) -> Result<PageResult<Tenant>, DataError> {
    let page_result = TenantBmc::page(&self.mm, req.filters, req.page).await?;
    Ok(page_result)
  }

  /// Get all active tenants
  pub async fn list_active(&self) -> Result<Vec<Tenant>, DataError> {
    let entities = TenantBmc::list_active(&self.mm).await?;
    Ok(entities)
  }

  /// Get all tenants including inactive
  pub async fn list_all(&self) -> Result<Vec<Tenant>, DataError> {
    let entities = TenantBmc::list_all(&self.mm).await?;
    Ok(entities)
  }

  /// Count active tenants
  pub async fn count_active(&self) -> Result<u64, DataError> {
    let count = TenantBmc::count_active(&self.mm).await?;
    Ok(count)
  }

  // Private validation methods
  async fn validate_create_input(&self, input: &TenantForCreate) -> Result<(), DataError> {
    // Validate name format
    if !self.is_valid_name(&input.name) {
      return Err(DataError::bad_request(format!("Invalid tenant name: {}", input.name)));
    }

    // Validate name uniqueness
    let exists = TenantBmc::exists_by_name(&self.mm, &input.name).await?;
    if exists {
      return Err(DataError::conflicted(format!("Tenant with name '{}' already exists", input.name)));
    }

    // Validate description length if provided
    if let Some(ref description) = input.description
      && description.len() > 1000
    {
      return Err(DataError::bad_request("Description too long, maximum 1000 characters".to_string()));
    }

    Ok(())
  }

  /// Validate tenant name format
  fn is_valid_name(&self, name: &str) -> bool {
    !name.trim().is_empty()
      && name.len() <= 255
      && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c.is_whitespace())
  }
}

// FromRequestParts implementation for Axum integration
impl FromRequestParts<fusions::core::application::Application> for TenantSvc {
  type Rejection = fusions::web::WebError;

  async fn from_request_parts(
    parts: &mut axum::http::request::Parts,
    state: &fusions::core::application::Application,
  ) -> core::result::Result<Self, Self::Rejection> {
    // Extract context and create model manager
    let ctx = fusions::web::extract_ctx(parts, state.fusion_setting().security())?;
    let mm = state.component::<fusionsql::ModelManager>().with_ctx(ctx);

    Ok(Self::new(mm))
  }
}
