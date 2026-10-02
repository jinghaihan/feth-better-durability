use std::sync::{
  atomic::{AtomicBool, AtomicUsize, Ordering},
  Mutex,
};

use skyline::nn::fs;

use crate::{config, rolling_log};

const MOUNT: &[u8] = b"fethdiag\0";
const CONFIG_PATH: &[u8] = b"fethdiag:/feth-better-durability.cfg\0";
const LOG_PATH: &[u8] = b"fethdiag:/feth-better-durability.log\0";
const MAX_CONFIG_BYTES: i64 = 4096;
static MAX_LOG_BYTES: AtomicUsize = AtomicUsize::new(2 * 1024 * 1024);
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
  let settings = match config::parse(&contents) {
    Ok(value) => value,
    Err(error) => {
      println!("[feth-diag] invalid configuration: {error}");
      return false;
    }
  };
  MAX_LOG_BYTES.store(settings.log_max_kib * 1024, Ordering::Release);
  if !settings.diagnostic_log {
    return false;
  }

  // An existing file is intentionally preserved; the next game launch adds
  // another clearly marked session to the same log.
  unsafe {
    fs::CreateFile(LOG_PATH.as_ptr(), 0);
  }
  ENABLED.store(true, Ordering::Release);
  append("=== feth durability diagnostic; new game launch ===");
  append(&format!("log_max_kib={}", settings.log_max_kib));
  enabled()
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
  let result = (|| {
    let mut file = rolling_log::switch::File::open(LOG_PATH)?;
    let line = format!("{message}\n");
    rolling_log::append(
      &mut file,
      line.as_bytes(),
      MAX_LOG_BYTES.load(Ordering::Acquire) as u64,
    )?;
    file.flush()
  })();
  if let Err(error) = result {
    ENABLED.store(false, Ordering::Release);
    println!("[feth-diag] log failed: {error}");
  }
}
