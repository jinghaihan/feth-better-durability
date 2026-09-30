/// A missing configuration file is equivalent to this default.
pub const DEFAULT_DIAGNOSTIC_LOG: bool = false;

pub fn diagnostic_log_enabled(contents: &str) -> Result<bool, &'static str> {
  let mut configured = None;
  for line in contents.lines() {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
      continue;
    }
    let (key, value) = line.split_once('=').ok_or("expected key=value")?;
    if key.trim() != "diagnostic_log" {
      return Err("unknown configuration key");
    }
    if configured.is_some() {
      return Err("duplicate diagnostic_log setting");
    }
    configured = Some(match value.trim() {
      "true" => true,
      "false" => false,
      _ => return Err("diagnostic_log must be true or false"),
    });
  }
  Ok(configured.unwrap_or(DEFAULT_DIAGNOSTIC_LOG))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn defaults_to_off() {
    assert!(!DEFAULT_DIAGNOSTIC_LOG);
    assert_eq!(diagnostic_log_enabled("# comment\n"), Ok(false));
  }

  #[test]
  fn parses_explicit_setting() {
    assert_eq!(diagnostic_log_enabled("diagnostic_log=true\n"), Ok(true));
    assert_eq!(
      diagnostic_log_enabled(" diagnostic_log = false\r\n"),
      Ok(false)
    );
  }

  #[test]
  fn rejects_ambiguous_settings() {
    assert!(diagnostic_log_enabled("diagnostic_log=yes").is_err());
    assert!(diagnostic_log_enabled("diagnostic_log=true\ndiagnostic_log=false").is_err());
    assert!(diagnostic_log_enabled("other=true").is_err());
  }
}
