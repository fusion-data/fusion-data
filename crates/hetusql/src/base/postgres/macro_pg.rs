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
						mm: &hetusql::ModelManager,
						entity_c: $for_create,
					) -> hetusql::Result<i64> {
						hetusql::base::create::<Self, _>(mm, entity_c).await
					}

					pub async fn create_many(
						mm: &hetusql::ModelManager,
						entity_c: Vec<$for_create>,
					) -> hetusql::Result<Vec<i64>> {
						hetusql::base::create_many::<Self, _>(mm, entity_c).await
					}
			)?

			$(
					pub async fn insert(
						mm: &hetusql::ModelManager,
						entity_i: $for_insert,
					) -> hetusql::Result<()> {
						hetusql::base::insert::<Self, _>(mm, entity_i).await
					}

					pub async fn insert_many(
						mm: &hetusql::ModelManager,
						entity_i: Vec<$for_insert>,
					) -> hetusql::Result<u64> {
						hetusql::base::insert_many::<Self, _>(mm, entity_i).await
					}
			)?

			pub async fn find_by_id(
				mm: &hetusql::ModelManager,
				id: impl Into<hetusql::id::Id>,
			) -> hetusql::Result<$entity> {
				hetusql::base::pg_find_by_id::<Self, _>(mm, id.into()).await
			}

			pub async fn get_by_id(
				mm: &hetusql::ModelManager,
				id: impl Into<hetusql::id::Id>,
			) -> hetusql::Result<Option<$entity>> {
				hetusql::base::pg_get_by_id::<Self, _>(mm, id.into()).await
			}

			$(
				pub async fn update_by_id(
					mm: &hetusql::ModelManager,
					id: impl Into<hetusql::id::Id>,
					entity_u: $for_update,
				) -> hetusql::Result<()> {
					hetusql::base::update_by_id::<Self, _>(mm, id.into(), entity_u).await
				}
			)?

			pub async fn delete_by_id(
				mm: &hetusql::ModelManager,
				id: impl Into<hetusql::id::Id>,
			) -> hetusql::Result<()> {
				hetusql::base::delete_by_id::<Self>(mm, id.into()).await
			}

			pub async fn delete_by_ids<V, I>(
				mm: &hetusql::ModelManager,
				ids: I,
			) -> hetusql::Result<u64>
			where
					V: Into<hetusql::id::Id>,
					I: IntoIterator<Item = V>,
			{
				let ids = ids.into_iter().map(|v| v.into()).collect();
				hetusql::base::delete_by_ids::<Self>(mm, ids).await
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
				mm: &hetusql::ModelManager,
				filter: Vec<$filter>,
			) -> hetusql::Result<Option<$entity>> {
				hetusql::base::pg_find_unique::<Self, _, _>(mm, filter).await
			}

			pub async fn find_many(
				mm: &hetusql::ModelManager,
				filter: Vec<$filter>,
				page: Option<hetusql::page::Page>,
			) -> hetusql::Result<Vec<$entity>> {
				hetusql::base::pg_find_many::<Self, _, _>(mm, filter, page).await
			}

			pub async fn count(
				mm: &hetusql::ModelManager,
				filter: Vec<$filter>,
			) -> hetusql::Result<u64> {
				hetusql::base::count::<Self, _>(mm, filter).await
			}

			pub async fn page(
				mm: &hetusql::ModelManager,
				filter: Vec<$filter>,
				page: hetusql::page::Page,
			) -> hetusql::Result<hetusql::page::PageResult<$entity>> {
				hetusql::base::pg_page::<Self, _, _>(mm, filter, page).await
			}

			pub async fn delete(
				mm: &hetusql::ModelManager,
				filter: Vec<$filter>,
			) -> hetusql::Result<u64> {
				hetusql::base::delete::<Self, _>(mm, filter).await
			}

			$(
				pub async fn update(
					mm: &hetusql::ModelManager,
					filter: Vec<$filter>,
					entity_u: $update,
				) -> hetusql::Result<u64> {
					hetusql::base::update::<Self, _, _>(mm, filter, entity_u).await
				}
			)?
		}
	};
}

#[macro_export]
macro_rules! generate_pg_bmc_filter_x {
  (
		Bmc: $struct_name:ident,
		Entity: $entity:ty,
		Filter: $filter:ty,
		$(ForUpdate: $update:ty,)?
	) => {
    impl $struct_name {
      pub async fn get_filter(mm: &hetusql::ModelManager, filter: Vec<$filter>) -> hetusql::Result<Option<$filter>> {
        hetusql::base::pg_get_filter::<Self, _, _>(mm, filter).await
      }
    }
  };
}
