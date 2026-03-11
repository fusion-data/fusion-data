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
						mm: &ultimatesql::ModelManager,
						entity_c: $for_create,
					) -> ultimatesql::Result<i64> {
						ultimatesql::base::create::<Self, _>(mm, entity_c).await
					}

					pub async fn create_many(
						mm: &ultimatesql::ModelManager,
						entity_c: Vec<$for_create>,
					) -> ultimatesql::Result<Vec<i64>> {
						ultimatesql::base::create_many::<Self, _>(mm, entity_c).await
					}
			)?

			$(
					pub async fn insert(
						mm: &ultimatesql::ModelManager,
						entity_i: $for_insert,
					) -> ultimatesql::Result<()> {
						ultimatesql::base::insert::<Self, _>(mm, entity_i).await
					}

					pub async fn insert_many(
						mm: &ultimatesql::ModelManager,
						entity_i: Vec<$for_insert>,
					) -> ultimatesql::Result<u64> {
						ultimatesql::base::insert_many::<Self, _>(mm, entity_i).await
					}
			)?

			pub async fn find_by_id(
				mm: &ultimatesql::ModelManager,
				id: impl Into<ultimatesql::id::Id>,
			) -> ultimatesql::Result<$entity> {
				ultimatesql::base::pg_find_by_id::<Self, _>(mm, id.into()).await
			}

			pub async fn get_by_id(
				mm: &ultimatesql::ModelManager,
				id: impl Into<ultimatesql::id::Id>,
			) -> ultimatesql::Result<Option<$entity>> {
				ultimatesql::base::pg_get_by_id::<Self, _>(mm, id.into()).await
			}

			$(
				pub async fn update_by_id(
					mm: &ultimatesql::ModelManager,
					id: impl Into<ultimatesql::id::Id>,
					entity_u: $for_update,
				) -> ultimatesql::Result<()> {
					ultimatesql::base::update_by_id::<Self, _>(mm, id.into(), entity_u).await
				}
			)?

			pub async fn delete_by_id(
				mm: &ultimatesql::ModelManager,
				id: impl Into<ultimatesql::id::Id>,
			) -> ultimatesql::Result<()> {
				ultimatesql::base::delete_by_id::<Self>(mm, id.into()).await
			}

			pub async fn delete_by_ids<V, I>(
				mm: &ultimatesql::ModelManager,
				ids: I,
			) -> ultimatesql::Result<u64>
			where
					V: Into<ultimatesql::id::Id>,
					I: IntoIterator<Item = V>,
			{
				let ids = ids.into_iter().map(|v| v.into()).collect();
				ultimatesql::base::delete_by_ids::<Self>(mm, ids).await
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
				mm: &ultimatesql::ModelManager,
				filter: Vec<$filter>,
			) -> ultimatesql::Result<Option<$entity>> {
				ultimatesql::base::pg_find_unique::<Self, _, _>(mm, filter).await
			}

			pub async fn find_many(
				mm: &ultimatesql::ModelManager,
				filter: Vec<$filter>,
				page: Option<ultimatesql::page::Page>,
			) -> ultimatesql::Result<Vec<$entity>> {
				ultimatesql::base::pg_find_many::<Self, _, _>(mm, filter, page).await
			}

			pub async fn count(
				mm: &ultimatesql::ModelManager,
				filter: Vec<$filter>,
			) -> ultimatesql::Result<u64> {
				ultimatesql::base::count::<Self, _>(mm, filter).await
			}

			pub async fn page(
				mm: &ultimatesql::ModelManager,
				filter: Vec<$filter>,
				page: ultimatesql::page::Page,
			) -> ultimatesql::Result<ultimatesql::page::PageResult<$entity>> {
				ultimatesql::base::pg_page::<Self, _, _>(mm, filter, page).await
			}

			pub async fn delete(
				mm: &ultimatesql::ModelManager,
				filter: Vec<$filter>,
			) -> ultimatesql::Result<u64> {
				ultimatesql::base::delete::<Self, _>(mm, filter).await
			}

			$(
				pub async fn update(
					mm: &ultimatesql::ModelManager,
					filter: Vec<$filter>,
					entity_u: $update,
				) -> ultimatesql::Result<u64> {
					ultimatesql::base::update::<Self, _, _>(mm, filter, entity_u).await
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
      pub async fn get_filter(
        mm: &ultimatesql::ModelManager,
        filter: Vec<$filter>,
      ) -> ultimatesql::Result<Option<$filter>> {
        ultimatesql::base::pg_get_filter::<Self, _, _>(mm, filter).await
      }
    }
  };
}
