use fusion_core::Builder;

#[test]
fn should_generate_builder_for_struct_with_no_properties() {
  #[derive(Debug, Builder)]
  struct Gleipnir {}

  let _gleipnir: Gleipnir = Gleipnir::builder().build();
  // assert_eq!(gleipnir, Gleipnir {});
}

#[test]
fn should_generate_builder_for_struct_one_property() {
  #[derive(Builder)]
  struct Gleipnir {
    roots_of: String,
  }

  let gleipnir = Gleipnir::builder().roots_of("mountains".to_string()).build();
  assert_eq!(&gleipnir.roots_of, "mountains");
}
