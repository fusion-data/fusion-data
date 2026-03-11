pub fn get_trace_id() -> Option<String> {
  tracing_opentelemetry_instrumentation_sdk::find_current_trace_id()
}
