pub fn get_trace_id() -> Option<String> {
  #[cfg(feature = "opentelemetry")]
  return tracing_opentelemetry_instrumentation_sdk::find_current_trace_id();
  #[cfg(not(feature = "opentelemetry"))]
  None
}
