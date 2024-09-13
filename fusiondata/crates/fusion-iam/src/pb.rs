pub mod fusion_iam {
  pub mod v1 {
    tonic::include_proto!("fusion_iam.v1");

    #[cfg(feature = "tonic-reflection")]
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("fusion_iam_descriptor");
  }
}
