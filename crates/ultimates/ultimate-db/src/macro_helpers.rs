/// 生成 sea_query::Value 实现。要求 Enum 设置 `#[repr(i32)]`
#[macro_export]
macro_rules! generate_enum_i32_to_sea_query_value {
  (
    $(Enum: $Enum:ty,)*
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
