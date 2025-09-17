#[cfg(unix)]
pub fn send_sigterm_to_self() {
  use libc::{SIGTERM, kill};
  let pid = std::process::id() as i32;
  println!("Unix: sending SIGTERM to self (pid={})", pid);
  unsafe {
    kill(pid, SIGTERM);
  }
}

#[cfg(windows)]
pub fn send_sigterm_to_self() {
  use windows_sys::Win32::System::Threading::PROCESS_TERMINATE;
  use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess};

  let pid = std::process::id();
  println!("Windows: terminating self (pid={})", pid);

  unsafe {
    let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
    if handle != 0 {
      TerminateProcess(handle, 1);
    }
  }
}
