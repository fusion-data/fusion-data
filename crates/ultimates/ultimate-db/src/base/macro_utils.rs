/// Convenience macro rules to generate default CRUD functions for a Bmc/Entity.
/// Note: If custom functionality is required, use the code below as foundational
///       code for the custom implementations.
#[macro_export]
macro_rules! generate_common_bmc_fns {
	(
		Bmc: $struct_name:ident,
		Entity: $entity:ty,
		$(ForCreate: $for_create:ty,)?
		$(ForUpdate: $for_update:ty,)?
	) => {
		impl $struct_name {
			$(
					pub async fn create(
							mm: &ultimate_db::ModelManager,
							entity_c: $for_create,
					) -> ultimate_db::Result<i64> {
							ultimate_db::base::create::<Self, _>(mm, entity_c).await
					}

					pub async fn create_many(
							mm: &ultimate_db::ModelManager,
							entity_c: Vec<$for_create>,
					) -> ultimate_db::Result<Vec<i64>> {
							ultimate_db::base::create_many::<Self, _>(mm, entity_c).await
					}
			)?

			$(
					pub async fn insert(
							mm: &ultimate_db::ModelManager,
							entity_c: $for_create,
					) -> ultimate_db::Result<()> {
							ultimate_db::base::insert::<Self, _>(mm, entity_c).await
					}

					pub async fn insert_many(
							mm: &ultimate_db::ModelManager,
							entity_c: Vec<$for_create>,
					) -> ultimate_db::Result<u64> {
							ultimate_db::base::insert_many::<Self, _>(mm, entity_c).await
					}
			)?

			pub async fn find_by_id(
					mm: &ultimate_db::ModelManager,
					id: impl Into<ultimate_db::Id>,
			) -> ultimate_db::Result<$entity> {
					ultimate_db::base::find_by_id::<Self, _>(mm, id.into()).await
			}

			$(
				pub async fn update_by_id(
						mm: &ultimate_db::ModelManager,
						id: impl Into<ultimate_db::Id>,
						entity_u: $for_update,
				) -> ultimate_db::Result<()> {
						ultimate_db::base::update_by_id::<Self, _>(mm, id.into(), entity_u).await
				}
			)?

			pub async fn delete_by_id(
					mm: &ultimate_db::ModelManager,
					id: impl Into<ultimate_db::Id>,
			) -> ultimate_db::Result<()> {
					ultimate_db::base::delete_by_id::<Self>(mm, id.into()).await
			}

			pub async fn delete_by_ids<V, I>(
					mm: &ultimate_db::ModelManager,
					ids: I,
			) -> ultimate_db::Result<u64>
			where
					V: Into<ultimate_db::Id>,
					I: IntoIterator<Item = V>,
			{
					let ids = ids.into_iter().map(|v| v.into()).collect();
					ultimate_db::base::delete_by_ids::<Self>(mm, ids).await
			}
		}
	};
}

#[macro_export]
macro_rules! generate_filter_bmc_fns {
	(
		Bmc: $struct_name:ident,
		Entity: $entity:ty,
		$(Filter: $filter:ty,)?
	) => {
		impl $struct_name {
			$(
				pub async fn find_unique(
					mm: &ultimate_db::ModelManager,
					filter: Vec<$filter>,
				) -> ultimate_db::Result<Option<$entity>> {
					ultimate_db::base::find_unique::<Self, _, _>(mm, filter).await
				}

				pub async fn find_many(
					mm: &ultimate_db::ModelManager,
					filter: Vec<$filter>,
					pagination: Option<&ultimate_api::v1::Pagination>,
				) -> ultimate_db::Result<Vec<$entity>> {
					ultimate_db::base::find_many::<Self, _, _>(mm, filter, pagination.map(Into::into)).await
				}

				pub async fn count(
					mm: &ultimate_db::ModelManager,
					filter: Vec<$filter>,
				) -> ultimate_db::Result<i64> {
					ultimate_db::base::count::<Self, _>(mm, filter).await
				}

				pub async fn page(
					mm: &ultimate_db::ModelManager,
					filter: Vec<$filter>,
					pagination: ultimate_api::v1::Pagination,
				) -> ultimate_db::Result<ultimate_api::v1::PagePayload<$entity>> {
					ultimate_db::base::page::<Self, _, _>(mm, filter, pagination).await
				}
			)?
		}
	};
}
