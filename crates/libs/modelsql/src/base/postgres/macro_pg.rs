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
						mm: &modelsql::ModelManager,
						entity_c: $for_create,
					) -> modelsql::Result<i64> {
						modelsql::base::create::<Self, _>(mm, entity_c).await
					}

					pub async fn create_many(
						mm: &modelsql::ModelManager,
						entity_c: Vec<$for_create>,
					) -> modelsql::Result<Vec<i64>> {
						modelsql::base::create_many::<Self, _>(mm, entity_c).await
					}
			)?

			$(
					pub async fn insert(
						mm: &modelsql::ModelManager,
						entity_i: $for_insert,
					) -> modelsql::Result<()> {
						modelsql::base::insert::<Self, _>(mm, entity_i).await
					}

					pub async fn insert_many(
						mm: &modelsql::ModelManager,
						entity_i: Vec<$for_insert>,
					) -> modelsql::Result<u64> {
						modelsql::base::insert_many::<Self, _>(mm, entity_i).await
					}
			)?

			pub async fn find_by_id(
				mm: &modelsql::ModelManager,
				id: impl Into<modelsql::id::Id>,
			) -> modelsql::Result<$entity> {
				modelsql::base::pg_find_by_id::<Self, _>(mm, id.into()).await
			}

			pub async fn get_by_id(
				mm: &modelsql::ModelManager,
				id: impl Into<modelsql::id::Id>,
			) -> modelsql::Result<Option<$entity>> {
				modelsql::base::pg_get_by_id::<Self, _>(mm, id.into()).await
			}

			$(
				pub async fn update_by_id(
					mm: &modelsql::ModelManager,
					id: impl Into<modelsql::id::Id>,
					entity_u: $for_update,
				) -> modelsql::Result<()> {
					modelsql::base::update_by_id::<Self, _>(mm, id.into(), entity_u).await
				}
			)?

			pub async fn delete_by_id(
				mm: &modelsql::ModelManager,
				id: impl Into<modelsql::id::Id>,
			) -> modelsql::Result<()> {
				modelsql::base::delete_by_id::<Self>(mm, id.into()).await
			}

			pub async fn delete_by_ids<V, I>(
				mm: &modelsql::ModelManager,
				ids: I,
			) -> modelsql::Result<u64>
			where
					V: Into<modelsql::id::Id>,
					I: IntoIterator<Item = V>,
			{
				let ids = ids.into_iter().map(|v| v.into()).collect();
				modelsql::base::delete_by_ids::<Self>(mm, ids).await
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
				mm: &modelsql::ModelManager,
				filter: Vec<$filter>,
			) -> modelsql::Result<Option<$entity>> {
				modelsql::base::pg_find_unique::<Self, _, _>(mm, filter).await
			}

			pub async fn find_many(
				mm: &modelsql::ModelManager,
				filter: Vec<$filter>,
				page: Option<modelsql::filter::Page>,
			) -> modelsql::Result<Vec<$entity>> {
				modelsql::base::pg_find_many::<Self, _, _>(mm, filter, page).await
			}

			pub async fn count(
				mm: &modelsql::ModelManager,
				filter: Vec<$filter>,
			) -> modelsql::Result<u64> {
				modelsql::base::count::<Self, _>(mm, filter).await
			}

			pub async fn page(
				mm: &modelsql::ModelManager,
				filter: Vec<$filter>,
				page: modelsql::filter::Page,
			) -> modelsql::Result<modelsql::page::PageResult<$entity>> {
				modelsql::base::pg_page::<Self, _, _>(mm, filter, page).await
			}

			pub async fn delete(
				mm: &modelsql::ModelManager,
				filter: Vec<$filter>,
			) -> modelsql::Result<u64> {
				modelsql::base::delete::<Self, _>(mm, filter).await
			}

			$(
				pub async fn update(
					mm: &modelsql::ModelManager,
					filter: Vec<$filter>,
					entity_u: $update,
				) -> modelsql::Result<u64> {
					modelsql::base::update::<Self, _, _>(mm, filter, entity_u).await
				}
			)?
		}
	};
}
