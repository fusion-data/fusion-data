use modelsql::{
  ModelManager, Result,
  base::{self, DbBmc, compute_list_options},
  filter::{FilterGroups, ListOptions},
  generate_pg_bmc_common,
};
use sea_query::{Condition, Expr, Query, SelectStatement};
use fusion_api::v1::{Page, PagePayload, Pagination};

use crate::{pb::fusion_iam::v1::CreateRoleDto, role::RoleIden};

use super::{
  Role, RoleFilters, RoleForUpdate,
  role_permission::{RolePermissionBmc, RolePermissionIden},
};

pub struct RoleBmc;
impl DbBmc for RoleBmc {
  const TABLE: &'static str = "role";
}

generate_pg_bmc_common!(
  Bmc: RoleBmc,
  Entity: Role,
  ForCreate: CreateRoleDto,
  ForUpdate: RoleForUpdate,
);

impl RoleBmc {
  pub async fn page(mm: &ModelManager, filters: RoleFilters, pagination: Pagination) -> Result<PagePayload<Role>> {
    let total_size = Self::count(mm, filters.clone()).await?;
    let items = Self::find_many(mm, filters, Some((&pagination).into())).await?;
    Ok(PagePayload::new(Page::new(total_size), items))
  }

  async fn count(mm: &ModelManager, filters: RoleFilters) -> Result<i64> {
    let count = base::count_on::<Self, _>(mm, move |query| Self::select_statement(query, filters, None)).await?;
    Ok(count)
  }

  async fn find_many(mm: &ModelManager, filters: RoleFilters, list_options: Option<ListOptions>) -> Result<Vec<Role>> {
    let items =
      base::pg_find_many_on::<Self, Role, _>(mm, |query| Self::select_statement(query, filters, list_options)).await?;
    Ok(items)
  }

  fn select_statement(
    query: &mut SelectStatement,
    filters: RoleFilters,
    list_options: Option<ListOptions>,
  ) -> Result<()> {
    // condition from filter
    {
      let group: FilterGroups = filters.filter.into();
      let cond: Condition = group.try_into()?;
      query.cond_where(cond);
    }

    {
      let sub_cond: Condition = filters.role_perm_filter.try_into()?;
      if !sub_cond.is_empty() {
        query.and_where(Expr::col(RoleIden::Id).in_subquery({
          let mut q = Query::select();
          q.from(RolePermissionBmc::table_ref()).column(RolePermissionIden::RoleId);
          q.cond_where(sub_cond);
          q
        }));
      }
    }

    let list_options = compute_list_options::<RoleBmc>(list_options)?;
    list_options.apply_to_sea_query(query);

    Ok(())
  }
}
