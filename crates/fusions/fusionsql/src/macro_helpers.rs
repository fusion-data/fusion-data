#[macro_export]
macro_rules! generate_enum_string_to_sea_query_value {
  (
    $(Enum: $Enum:ty),* $(,)?
  ) => {
    $(
      impl From<$Enum> for sea_query::Value {
        fn from(value: $Enum) -> Self {
          sea_query::Value::String(Some(Box::new(value.to_string())))
        }
      }

      impl sea_query::Nullable for $Enum {
        fn null() -> sea_query::Value {
          sea_query::Value::String(None)
        }
      }
    )*
  };
}

/// 生成 sea_query::Value 实现。要求 Enum 设置 `#[repr(i32)]`
#[macro_export]
macro_rules! generate_enum_i32_to_sea_query_value {
  (
    $(Enum: $Enum:ty),* $(,)?
  ) => {
    $(
      impl From<$Enum> for sea_query::Value {
        fn from(value: $Enum) -> Self {
          sea_query::Value::Int(Some(value as i32))
        }
      }

      impl sea_query::Nullable for $Enum {
        fn null() -> sea_query::Value {
          sea_query::Value::Int(None)
        }
      }
    )*
  };
}

#[macro_export]
macro_rules! generate_uuid_newtype_to_sea_query_value {
  (
    $(Struct: $Struct:ty),* $(,)?
  ) => {
    $(
      impl From<$Struct> for sea_query::Value {
        fn from(value: $Struct) -> Self {
          sea_query::Value::Uuid(Some(Box::new(value.0)))
        }
      }
      impl sea_query::Nullable for $Struct {
        fn null() -> sea_query::Value {
          sea_query::Value::Uuid(None)
        }
      }
      impl From<$Struct> for fusionsql::id::Id {
        fn from(value: $Struct) -> Self {
          fusionsql::id::Id::Uuid(value.0)
        }
      }
    )*
  };
}

#[macro_export]
macro_rules! generate_i64_newtype_to_sea_query_value {
  (
    $(Struct: $Struct:ty),* $(,)?
  ) => {
    $(
      impl From<$Struct> for sea_query::Value {
        fn from(value: $Struct) -> Self {
          sea_query::Value::BigInt(Some(value.0))
        }
      }
      impl sea_query::Nullable for $Struct {
        fn null() -> sea_query::Value {
          sea_query::Value::BigInt(None)
        }
      }
      impl From<$Struct> for fusionsql::id::Id {
        fn from(value: $Struct) -> Self {
          fusionsql::id::Id::I64(value.0)
        }
      }
    )*
  };
}

#[macro_export]
macro_rules! generate_i32_newtype_to_sea_query_value {
  (
    $(Struct: $Struct:ty),* $(,)?
  ) => {
    $(
      impl From<$Struct> for sea_query::Value {
        fn from(value: $Struct) -> Self {
          sea_query::Value::Int(Some(value.0))
        }
      }
      impl sea_query::Nullable for $Struct {
        fn null() -> sea_query::Value {
          sea_query::Value::Int(None)
        }
      }
      impl From<$Struct> for fusionsql::id::Id {
        fn from(value: $Struct) -> Self {
          fusionsql::id::Id::I32(value.0)
        }
      }
    )*
  };
}

#[macro_export]
macro_rules! generate_string_newtype_to_sea_query_value {
  (
    $(Struct: $Struct:ty),* $(,)?
  ) => {
    $(
      impl From<$Struct> for sea_query::Value {
        fn from(value: $Struct) -> Self {
          sea_query::Value::String(Some(Box::new(value.0)))
        }
      }
      impl sea_query::Nullable for $Struct {
        fn null() -> sea_query::Value {
          sea_query::Value::String(None)
        }
      }
      impl From<$Struct> for fusionsql::id::Id {
        fn from(value: $Struct) -> Self {
          fusionsql::id::Id::String(value.0)
        }
      }
    )*
  };
}
