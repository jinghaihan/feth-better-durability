#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
  pub diagnostic_log: bool,
  pub log_max_kib: usize,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      diagnostic_log: false,
      log_max_kib: 2048,
    }
  }
}

pub fn parse(contents: &str) -> Result<Settings, &'static str> {
  let mut settings = Settings::default();
  let mut seen = [false; 2];
  for line in contents.lines() {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
      continue;
    }
    let (key, value) = line.split_once('=').ok_or("expected key=value")?;
    let index = match key.trim() {
      "diagnostic_log" => 0,
      "log_max_kib" => 1,
      _ => return Err("unknown configuration key"),
    };
    if seen[index] {
      return Err("duplicate configuration key");
    }
    seen[index] = true;
    match index {
      0 => {
        settings.diagnostic_log = match value.trim() {
          "true" => true,
          "false" => false,
          _ => return Err("diagnostic_log must be true or false"),
        };
      }
      1 => {
        settings.log_max_kib = value.trim().parse().map_err(|_| "invalid log_max_kib")?;
        if !(64..=65536).contains(&settings.log_max_kib) {
          return Err("log_max_kib must be between 64 and 65536");
        }
      }
      _ => unreachable!(),
    }
  }
  Ok(settings)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn defaults_to_off() {
    assert_eq!(parse("# comment\n"), Ok(Settings::default()));
  }

  #[test]
  fn parses_explicit_setting() {
    assert!(parse("diagnostic_log=true\n").unwrap().diagnostic_log);
    assert!(!parse(" diagnostic_log = false\r\n").unwrap().diagnostic_log);
    assert_eq!(parse("diagnostic_log=true").unwrap().log_max_kib, 2048);
    for size in [64, 1024, 65536] {
      assert_eq!(
        parse(&format!("log_max_kib={size}")).unwrap().log_max_kib,
        size
      );
    }
  }

  #[test]
  fn rejects_ambiguous_settings() {
    for contents in [
      "diagnostic_log=yes",
      "diagnostic_log=true\ndiagnostic_log=false",
      "other=true",
      "log_max_kib=64\nlog_max_kib=128",
      "log_max_kib=0",
      "log_max_kib=63",
      "log_max_kib=65537",
      "log_max_kib=-1",
      "log_max_kib=huge",
    ] {
      assert!(parse(contents).is_err(), "{contents}");
    }
  }
}
