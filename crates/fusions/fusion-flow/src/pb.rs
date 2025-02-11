pub mod fusion_flow {
  pub mod v1 {
    tonic::include_proto!("fusion_flow.v1");

    #[cfg(feature = "tonic-reflection")]
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("fusion_flow_descriptor");
  }
}
