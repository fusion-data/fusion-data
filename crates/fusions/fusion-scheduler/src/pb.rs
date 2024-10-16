pub mod fusion_scheduler {
  pub mod v1 {
    tonic::include_proto!("fusion_scheduler.v1");

    #[cfg(feature = "tonic-reflection")]
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("fusion_scheduler_descriptor");
  }
}
