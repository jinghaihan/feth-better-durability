use std::sync::{
  atomic::{AtomicBool, Ordering},
  Mutex,
};

use skyline::nn::fs;

use crate::config;

const MOUNT: &[u8] = b"fethdiag\0";
const CONFIG_PATH: &[u8] = b"fethdiag:/feth-better-durability.cfg\0";
const LOG_PATH: &[u8] = b"fethdiag:/feth-better-durability.log\0";
const MAX_CONFIG_BYTES: i64 = 4096;
static ENABLED: AtomicBool = AtomicBool::new(false);
static WRITE_LOCK: Mutex<()> = Mutex::new(());

pub fn enabled() -> bool {
  ENABLED.load(Ordering::Acquire)
}

/// Read the opt-in setting once at startup. A missing/invalid file or an SD
/// error leaves normal gameplay hooks active but never installs the tracer.
pub fn init() -> bool {
  // SAFETY: MOUNT is NUL-terminated and nn::fs retains only the mounted SD
  // filesystem, not this string pointer.
  let mount_result = unsafe { fs::MountSdCard(MOUNT.as_ptr()) };
  if mount_result != 0 {
    println!("[feth-diag] SD mount failed: {mount_result:#x}");
    return false;
  }

  let Some(contents) = read_config() else {
    return false;
  };
  let setting = match config::diagnostic_log_enabled(&contents) {
    Ok(value) => value,
    Err(error) => {
      println!("[feth-diag] invalid configuration: {error}");
      return false;
    }
  };
  if !setting {
    return false;
  }

  // An existing file is intentionally preserved; the next game launch adds
  // another clearly marked session to the same log.
  unsafe {
    fs::CreateFile(LOG_PATH.as_ptr(), 0);
  }
  let mut handle = fs::FileHandle { handle: 0 };
  let open_result = unsafe {
    fs::OpenFile(
      &mut handle,
      LOG_PATH.as_ptr(),
      (fs::OpenMode_OpenMode_Write | fs::OpenMode_OpenMode_Append) as i32,
    )
  };
  if open_result != 0 {
    println!("[feth-diag] log open failed: {open_result:#x}");
    return false;
  }
  unsafe { fs::CloseFile(handle) };

  ENABLED.store(true, Ordering::Release);
  append("=== feth durability diagnostic; new game launch ===");
  true
}

fn read_config() -> Option<String> {
  let mut handle = fs::FileHandle { handle: 0 };
  // SAFETY: CONFIG_PATH is NUL-terminated. A missing file is the off default.
  if unsafe {
    fs::OpenFile(
      &mut handle,
      CONFIG_PATH.as_ptr(),
      fs::OpenMode_OpenMode_Read as i32,
    )
  } != 0
  {
    return None;
  }

  let mut size = 0i64;
  let size_result = unsafe { fs::GetFileSize(&mut size, handle) };
  if size_result != 0 || !(0..=MAX_CONFIG_BYTES).contains(&size) {
    println!("[feth-diag] invalid configuration size or read error");
    unsafe { fs::CloseFile(handle) };
    return None;
  }
  if size == 0 {
    unsafe { fs::CloseFile(handle) };
    return Some(String::new());
  }
  let mut bytes = vec![0u8; size as usize];
  let mut read = 0u64;
  let read_result =
    unsafe { fs::ReadFile1(&mut read, handle, 0, bytes.as_mut_ptr(), bytes.len() as u64) };
  unsafe { fs::CloseFile(handle) };
  if read_result != 0 || read != bytes.len() as u64 {
    println!("[feth-diag] could not read configuration");
    return None;
  }
  match String::from_utf8(bytes) {
    Ok(value) => Some(value),
    Err(_) => {
      println!("[feth-diag] configuration is not UTF-8");
      None
    }
  }
}

pub fn append(message: &str) {
  if !enabled() {
    return;
  }
  let Ok(_lock) = WRITE_LOCK.lock() else {
    return;
  };
  let mut handle = fs::FileHandle { handle: 0 };
  // SAFETY: LOG_PATH is NUL-terminated. The handle is used only after a
  // successful open, and the line buffer outlives WriteFile.
  unsafe {
    let open_result = fs::OpenFile(
      &mut handle,
      LOG_PATH.as_ptr(),
      (fs::OpenMode_OpenMode_Write | fs::OpenMode_OpenMode_Append) as i32,
    );
    if open_result != 0 {
      println!("[feth-diag] log open failed: {open_result:#x}; {message}");
      return;
    }
    let mut offset = 0i64;
    let size_result = fs::GetFileSize(&mut offset, handle);
    if size_result != 0 {
      println!("[feth-diag] log size failed: {size_result:#x}");
      fs::CloseFile(handle);
      return;
    }
    let line = format!("{message}\n");
    let option = fs::WriteOption {
      flags: fs::WriteOptionFlag_WriteOptionFlag_Flush as i32,
    };
    let write_result = fs::WriteFile(handle, offset, line.as_ptr(), line.len() as u64, &option);
    fs::CloseFile(handle);
    if write_result != 0 {
      println!("[feth-diag] log write failed: {write_result:#x}; {message}");
    }
  }
}
