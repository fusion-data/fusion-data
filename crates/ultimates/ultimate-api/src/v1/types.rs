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
  ($Primitive:ty, $Array:ty) => {
    impl<I> From<I> for $Array
    where
      I: IntoIterator<Item = $Primitive>,
    {
      fn from(value: I) -> Self {
        Self { value: value.into_iter().collect() }
      }
    }
  };
}

impl_array_from!(bool, ArrayBool);
impl_array_from!(i32, ArrayInt32);
impl_array_from!(i64, ArrayInt64);
impl_array_from!(f64, ArrayDouble);
impl_array_from!(String, ArrayString);
