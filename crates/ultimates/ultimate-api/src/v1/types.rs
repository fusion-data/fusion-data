use super::{ArrayBool, ArrayDouble, ArrayInt32, ArrayInt64, ArrayString, Null};

impl Null {
  pub fn is_null(&self) -> bool {
    self == &Null::IsNull
  }

  pub fn not_null(&self) -> bool {
    self == &Null::NotNull
  }
}

macro_rules! impl_array_from {
  ($Value:ty, $Array:ty) => {
    impl From<Vec<$Value>> for $Array {
      fn from(value: Vec<$Value>) -> Self {
        Self { value }
      }
    }

    impl From<$Value> for $Array {
      fn from(value: $Value) -> Self {
        Self { value: vec![value] }
      }
    }
  };
}

impl_array_from!(bool, ArrayBool);
impl_array_from!(i32, ArrayInt32);
impl_array_from!(i64, ArrayInt64);
impl_array_from!(f64, ArrayDouble);
impl_array_from!(String, ArrayString);

impl From<uuid::Uuid> for ArrayString {
  fn from(value: uuid::Uuid) -> Self {
    Self { value: vec![value.to_string()] }
  }
}

impl From<Vec<uuid::Uuid>> for ArrayString {
  fn from(value: Vec<uuid::Uuid>) -> Self {
    Self { value: value.into_iter().map(|v| v.to_string()).collect() }
  }
}

impl TryFrom<ArrayString> for Vec<uuid::Uuid> {
  type Error = uuid::Error;

  fn try_from(value: ArrayString) -> Result<Self, Self::Error> {
    let mut v = Vec::with_capacity(value.value.len());
    for item in value.value {
      v.push(item.parse()?);
    }
    Ok(v)
  }
}
