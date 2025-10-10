use fusionsql::generate_enum_i32_to_sea_query_value;

use crate::workflow::CredentialKind;

generate_enum_i32_to_sea_query_value!(Enum: CredentialKind);
