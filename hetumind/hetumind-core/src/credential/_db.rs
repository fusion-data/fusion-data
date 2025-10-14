use fusionsql::generate_uuid_newtype_to_sea_query_value;

use crate::credential::CredentialId;

generate_uuid_newtype_to_sea_query_value!(Struct: CredentialId);
