pub mod fusion_metadata {
  pub mod v1 {
    tonic::include_proto!("fusion_metadata.v1");

    #[allow(unused)]
    #[cfg(feature = "tonic-reflection")]
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("fusion_metadata_descriptor");
  }
}
