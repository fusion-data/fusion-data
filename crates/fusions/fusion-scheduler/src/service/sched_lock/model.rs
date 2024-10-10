#[derive(Clone, Copy, sqlx::Type)]
#[repr(i32)]
pub enum LockKind {
  RegisterScheduler = 1,
  ScanNamespaces = 2,
  ComputeProcessTasks = 3,
}
