/// Convenience macro rules to generate default CRUD functions for a Bmc/Entity.
/// Note: If custom functionality is required, use the code below as foundational
///       code for the custom implementations.
#[macro_export]
macro_rules! generate_pg_bmc_common {
	(
		Bmc: $struct_name:ident,
		Entity: $entity:ty,
		$(ForCreate: $for_create:ty,)?
		$(ForUpdate: $for_update:ty,)?
		$(ForInsert: $for_insert:ty,)?
	) => {
		impl $struct_name {
			$(
					pub async fn create(
						mm: &fusionsql::ModelManager,
						entity_c: $for_create,
					) -> fusionsql::Result<i64> {
						fusionsql::base::create::<Self, _>(mm, entity_c).await
					}

					pub async fn create_many(
						mm: &fusionsql::ModelManager,
						entity_c: Vec<$for_create>,
					) -> fusionsql::Result<Vec<i64>> {
						fusionsql::base::create_many::<Self, _>(mm, entity_c).await
					}
			)?

			$(
					pub async fn insert(
						mm: &fusionsql::ModelManager,
						entity_i: $for_insert,
					) -> fusionsql::Result<()> {
						fusionsql::base::insert::<Self, _>(mm, entity_i).await
					}

					pub async fn insert_many(
						mm: &fusionsql::ModelManager,
						entity_i: Vec<$for_insert>,
					) -> fusionsql::Result<u64> {
						fusionsql::base::insert_many::<Self, _>(mm, entity_i).await
					}
			)?

			pub async fn find_by_id(
				mm: &fusionsql::ModelManager,
				id: impl Into<fusionsql::id::Id>,
			) -> fusionsql::Result<$entity> {
				fusionsql::base::pg_find_by_id::<Self, _>(mm, id.into()).await
			}

			pub async fn get_by_id(
				mm: &fusionsql::ModelManager,
				id: impl Into<fusionsql::id::Id>,
			) -> fusionsql::Result<Option<$entity>> {
				fusionsql::base::pg_get_by_id::<Self, _>(mm, id.into()).await
			}

			$(
				pub async fn update_by_id(
					mm: &fusionsql::ModelManager,
					id: impl Into<fusionsql::id::Id>,
					entity_u: $for_update,
				) -> fusionsql::Result<()> {
					fusionsql::base::update_by_id::<Self, _>(mm, id.into(), entity_u).await
				}
			)?

			pub async fn delete_by_id(
				mm: &fusionsql::ModelManager,
				id: impl Into<fusionsql::id::Id>,
			) -> fusionsql::Result<()> {
				fusionsql::base::delete_by_id::<Self>(mm, id.into()).await
			}

			pub async fn delete_by_ids<V, I>(
				mm: &fusionsql::ModelManager,
				ids: I,
			) -> fusionsql::Result<u64>
			where
					V: Into<fusionsql::id::Id>,
					I: IntoIterator<Item = V>,
			{
				let ids = ids.into_iter().map(|v| v.into()).collect();
				fusionsql::base::delete_by_ids::<Self>(mm, ids).await
			}
		}
	};
}

#[macro_export]
macro_rules! generate_pg_bmc_filter {
	(
		Bmc: $struct_name:ident,
		Entity: $entity:ty,
		Filter: $filter:ty,
		$(ForUpdate: $update:ty,)?
	) => {
		impl $struct_name {
			pub async fn find_unique(
				mm: &fusionsql::ModelManager,
				filter: Vec<$filter>,
			) -> fusionsql::Result<Option<$entity>> {
				fusionsql::base::pg_find_unique::<Self, _, _>(mm, filter).await
			}

			pub async fn find_many(
				mm: &fusionsql::ModelManager,
				filter: Vec<$filter>,
				page: Option<fusionsql::page::Page>,
			) -> fusionsql::Result<Vec<$entity>> {
				fusionsql::base::pg_find_many::<Self, _, _>(mm, filter, page).await
			}

			pub async fn count(
				mm: &fusionsql::ModelManager,
				filter: Vec<$filter>,
			) -> fusionsql::Result<u64> {
				fusionsql::base::count::<Self, _>(mm, filter).await
			}

			pub async fn page(
				mm: &fusionsql::ModelManager,
				filter: Vec<$filter>,
				page: fusionsql::page::Page,
			) -> fusionsql::Result<fusionsql::page::PageResult<$entity>> {
				fusionsql::base::pg_page::<Self, _, _>(mm, filter, page).await
			}

			pub async fn delete(
				mm: &fusionsql::ModelManager,
				filter: Vec<$filter>,
			) -> fusionsql::Result<u64> {
				fusionsql::base::delete::<Self, _>(mm, filter).await
			}

			$(
				pub async fn update(
					mm: &fusionsql::ModelManager,
					filter: Vec<$filter>,
					entity_u: $update,
				) -> fusionsql::Result<u64> {
					fusionsql::base::update::<Self, _, _>(mm, filter, entity_u).await
				}
			)?
		}
	};
}
