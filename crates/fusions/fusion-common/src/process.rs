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

/// 检查是否为僵尸进程
pub fn is_zombie_process(pid: u32) -> bool {
  #[cfg(unix)]
  {
    use std::fs;

    let stat_path = format!("/proc/{}/stat", pid);
    if let Ok(stat_content) = fs::read_to_string(stat_path) {
      let fields: Vec<&str> = stat_content.split_whitespace().collect();
      if fields.len() > 2 {
        return fields[2] == "Z";
      }
    }
  }

  // Windows 没有僵尸进程的概念

  false
}
