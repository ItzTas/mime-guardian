use std::ffi::OsString;

use anyhow::{Result, bail};

use super::constants::{NOTIFY_ENV, SWITCH_EXPECTED, SWITCH_WORDS};

pub fn switch(text: &str) -> Option<bool> {
    let word = text.trim().to_ascii_lowercase();
    SWITCH_WORDS
        .iter()
        .find(|(name, _)| *name == word)
        .map(|(_, on)| *on)
}

pub fn parse_switch(text: &str) -> Result<bool, String> {
    switch(text).ok_or_else(|| format!("expected {SWITCH_EXPECTED}"))
}

pub fn notify_override() -> Result<Option<bool>> {
    override_from(std::env::var_os(NOTIFY_ENV))
}

fn override_from(value: Option<OsString>) -> Result<Option<bool>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let text = value.to_string_lossy();
    if text.trim().is_empty() {
        return Ok(None);
    }
    let Some(on) = switch(&text) else {
        bail!("{NOTIFY_ENV}={text} is not valid (expected {SWITCH_EXPECTED})");
    };
    Ok(Some(on))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_word_pair_is_read_in_any_case() {
        for word in ["on", "true", "1", "yes", "ON", " Yes "] {
            assert_eq!(switch(word), Some(true), "{word}");
        }
        for word in ["off", "false", "0", "no", "OFF", " No "] {
            assert_eq!(switch(word), Some(false), "{word}");
        }
        assert_eq!(switch("maybe"), None);
    }

    #[test]
    fn the_variable_overrides_only_when_set_to_a_word() {
        assert_eq!(override_from(None).unwrap(), None);
        assert_eq!(override_from(Some("".into())).unwrap(), None);
        assert_eq!(override_from(Some("0".into())).unwrap(), Some(false));
        assert_eq!(override_from(Some("yes".into())).unwrap(), Some(true));

        let err = override_from(Some("maybe".into())).unwrap_err().to_string();
        assert!(err.contains("MMG_NOTIFY=maybe"));
    }
}
