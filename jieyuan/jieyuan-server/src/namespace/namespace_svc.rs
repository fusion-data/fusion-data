use axum::extract::FromRequestParts;
use fusion_core::DataError;
use fusionsql::page::PageResult;

use jieyuan_core::model::{NamespaceEntity, NamespaceForCreate, NamespaceForPage, NamespaceForUpdate, NamespaceStatus};

use super::NamespaceBmc;

#[derive(Clone)]
pub struct NamespaceSvc {
  mm: fusionsql::ModelManager,
}

impl NamespaceSvc {
  /// Create new namespace service instance
  pub fn new(mm: fusionsql::ModelManager) -> Self {
    Self { mm }
  }

  /// Create a new namespace
  pub async fn create(&self, input: NamespaceForCreate) -> Result<i64, DataError> {
    // Validate input
    self.validate_create_input(&input).await?;

    // Create namespace
    let id = NamespaceBmc::create(&self.mm, input).await?;

    Ok(id)
  }

  /// Get namespace by ID
  pub async fn get(&self, id: i64) -> Result<Option<NamespaceEntity>, DataError> {
    let entity = NamespaceBmc::get_by_id(&self.mm, id).await?;

    // Validate tenant access
    if let Some(ref ns) = entity {
      self.validate_tenant_access(ns)?;
    }

    Ok(entity)
  }

  /// Get namespace by name (within current tenant)
  pub async fn get_by_name(&self, name: &str) -> Result<Option<NamespaceEntity>, DataError> {
    let tenant_id = self.mm.ctx_ref()?.tenant_id();
    let entity = NamespaceBmc::get_by_name(&self.mm, name, tenant_id).await?;

    Ok(entity)
  }

  /// Update namespace
  pub async fn update(&self, id: i64, input: NamespaceForUpdate) -> Result<(), DataError> {
    // Get existing namespace
    let existing = NamespaceBmc::find_by_id(&self.mm, id).await?;

    // Validate tenant access
    self.validate_tenant_access(&existing)?;

    // If updating name, check uniqueness
    if let Some(ref name) = input.name {
      if name != &existing.name {
        let exists = NamespaceBmc::exists_by_name(&self.mm, name, existing.tenant_id).await?;
        if exists {
          return Err(DataError::conflicted(format!("Namespace with name '{}' already exists", name)));
        }
      }
    }

    NamespaceBmc::update_by_id(&self.mm, id, input).await.map_err(DataError::from)
  }

  /// Enable namespace (set status to Active)
  pub async fn enable(&self, id: i64) -> Result<(), DataError> {
    // Get existing namespace
    let existing = NamespaceBmc::find_by_id(&self.mm, id).await?;

    // Validate tenant access
    self.validate_tenant_access(&existing)?;

    let update_input = NamespaceForUpdate { status: Some(NamespaceStatus::Active), ..Default::default() };

    NamespaceBmc::update_by_id(&self.mm, id, update_input).await.map_err(DataError::from)
  }

  /// Disable namespace (set status to Disabled)
  pub async fn disable(&self, id: i64) -> Result<(), DataError> {
    // Get existing namespace
    let existing = NamespaceBmc::find_by_id(&self.mm, id).await?;

    // Validate tenant access
    self.validate_tenant_access(&existing)?;

    let update_input = NamespaceForUpdate { status: Some(NamespaceStatus::Disabled), ..Default::default() };

    NamespaceBmc::update_by_id(&self.mm, id, update_input).await.map_err(DataError::from)
  }

  /// Get paginated list of namespaces (tenant-scoped)
  pub async fn page(&self, req: NamespaceForPage) -> Result<PageResult<NamespaceEntity>, DataError> {
    NamespaceBmc::page(&self.mm, req.filters, req.page).await.map_err(DataError::from)
  }

  /// Get all namespaces for current tenant
  pub async fn list_by_tenant(&self) -> Result<Vec<NamespaceEntity>, DataError> {
    let tenant_id = self.mm.ctx_ref()?.tenant_id();
    let entities = NamespaceBmc::list_by_tenant(&self.mm, tenant_id).await?;
    Ok(entities)
  }

  /// Count namespaces for current tenant
  pub async fn count_by_tenant(&self) -> Result<u64, DataError> {
    let tenant_id = self.mm.ctx_ref()?.tenant_id();
    NamespaceBmc::count_by_tenant(&self.mm, tenant_id).await.map_err(DataError::from)
  }

  // Private validation methods
  async fn validate_create_input(&self, input: &NamespaceForCreate) -> Result<(), DataError> {
    // Validate name format
    if !self.is_valid_name(&input.name) {
      return Err(DataError::bad_request(format!("Invalid namespace name: {}", input.name)));
    }

    // Validate name uniqueness
    let tenant_id = self.mm.ctx_ref()?.tenant_id();
    let exists = NamespaceBmc::exists_by_name(&self.mm, &input.name, tenant_id).await?;
    if exists {
      return Err(DataError::conflicted(format!("Namespace with name '{}' already exists", input.name)));
    }

    Ok(())
  }

  fn validate_tenant_access(&self, namespace: &NamespaceEntity) -> Result<(), DataError> {
    let ctx = self.mm.ctx_ref()?;
    if namespace.tenant_id != ctx.tenant_id() {
      return Err(DataError::forbidden(format!("Access denied to namespace {}", namespace.id)));
    }
    Ok(())
  }

  /// Validate namespace name format
  fn is_valid_name(&self, name: &str) -> bool {
    !name.trim().is_empty()
      && name.len() <= 255
      && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c.is_whitespace())
  }
}

// FromRequestParts implementation for Axum integration
impl FromRequestParts<fusion_core::application::Application> for NamespaceSvc {
  type Rejection = fusion_web::WebError;

  async fn from_request_parts(
    parts: &mut axum::http::request::Parts,
    state: &fusion_core::application::Application,
  ) -> core::result::Result<Self, Self::Rejection> {
    // Extract context and create model manager
    let ctx = fusion_web::extract_ctx(parts, state.fusion_setting().security())?;
    let mm = state.component::<fusionsql::ModelManager>().with_ctx(ctx);

    Ok(Self::new(mm))
  }
}
