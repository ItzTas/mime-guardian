use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::constants::{COMMAND, DIR, FILE_NAME, MEDIA_SEPARATOR, WILDCARD};
use super::types::Config;
use crate::utils::Xdg;

pub fn path(xdg: &Xdg) -> PathBuf {
    xdg.config_home.join(DIR).join(FILE_NAME)
}

pub fn read(path: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => {
            Err(err).with_context(|| format!("{COMMAND}: failed to read {}", path.display()))
        }
    }
}

pub fn load(path: &Path) -> Result<Config> {
    let Some(text) = read(path)? else {
        bail!(
            "{COMMAND}: {} does not exist (use \"mmg init\" to create it)",
            path.display()
        );
    };
    parse(&text).with_context(|| format!("{COMMAND}: {} is not valid", path.display()))
}

pub fn parse(text: &str) -> Result<Config> {
    let config: Config = toml_edit::de::from_str(text)?;
    for mime in config.defaults.keys() {
        validate(mime)?;
    }
    Ok(config)
}

pub fn validate(mime: &str) -> Result<()> {
    let Some((media, subtype)) = mime.split_once(MEDIA_SEPARATOR) else {
        bail!("\"{mime}\" is not a MIME type (expected media/subtype)");
    };
    if media.is_empty() || media == WILDCARD || subtype.is_empty() {
        bail!("\"{mime}\" is not a MIME type (expected media/subtype)");
    }
    if subtype != WILDCARD && subtype.contains(WILDCARD) {
        bail!("\"{mime}\" can only use * as the whole subtype, as in image/*");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_config_is_read() {
        let config = parse(
            "[defaults]\n\"text/html\" = \"firefox.desktop\"\n\"image/*\" = \"loupe.desktop\"\n\n[guard]\nnotify = false\ndesktops = [\"kde\"]\n",
        )
        .unwrap();

        assert_eq!(config.defaults["text/html"], "firefox.desktop");
        assert_eq!(config.defaults["image/*"], "loupe.desktop");
        assert!(!config.guard.notify);
        assert!(config.guard.added_associations);
        assert_eq!(config.guard.desktops, ["kde"]);
    }

    #[test]
    fn an_empty_file_is_an_empty_config() {
        assert_eq!(parse("").unwrap(), Config::default());
    }

    #[test]
    fn bad_keys_and_unknown_fields_are_refused() {
        assert!(parse("[defaults]\n\"html\" = \"a.desktop\"\n").is_err());
        assert!(parse("[defaults]\n\"*/*\" = \"a.desktop\"\n").is_err());
        assert!(parse("[defaults]\n\"image/p*\" = \"a.desktop\"\n").is_err());
        assert!(parse("[guard]\nnotfy = true\n").is_err());
    }

    #[test]
    fn a_missing_file_points_at_init() {
        let dir = tempfile::tempdir().unwrap();

        let err = load(&dir.path().join("config.toml"))
            .unwrap_err()
            .to_string();

        assert!(err.contains("mmg init"));
    }
}
