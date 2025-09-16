use fusion_core::Result;
use modelsql::{ModelManager, page::PageResult};

use hetuiam_core::types::{Policy, PolicyForCreate, PolicyForPage, PolicyForUpdate};

use super::PolicyBmc;

#[derive(Debug, Clone)]
pub struct PolicySvc {
  mm: ModelManager,
}

impl PolicySvc {
  pub fn new(mm: ModelManager) -> Self {
    Self { mm }
  }

  pub async fn create(&self, policy_for_create: PolicyForCreate) -> Result<i64> {
    let id = PolicyBmc::create(&self.mm, policy_for_create).await?;
    Ok(id)
  }

  pub async fn find_option_by_id(&self, id: i64) -> Result<Option<Policy>> {
    let policy = PolicyBmc::find_by_id(&self.mm, id).await.ok();
    Ok(policy)
  }

  pub async fn find_by_id(&self, id: i64) -> Result<Policy> {
    let policy = PolicyBmc::find_by_id(&self.mm, id).await?;
    Ok(policy)
  }

  pub async fn update_by_id(&self, id: i64, policy_for_update: PolicyForUpdate) -> Result<()> {
    PolicyBmc::update_by_id(&self.mm, id, policy_for_update).await?;
    Ok(())
  }

  pub async fn delete_by_id(&self, id: i64) -> Result<()> {
    PolicyBmc::delete_by_id(&self.mm, id).await?;
    Ok(())
  }

  pub async fn page(&self, req: PolicyForPage) -> Result<PageResult<Policy>> {
    let page = PolicyBmc::page(&self.mm, req.filter, req.page).await?;
    Ok(page)
  }
}
