use std::collections::BTreeMap;

use anyhow::{Context, Result};
use toml_edit::{DocumentMut, Item, Table, value};

use super::constants::{ADDED_ASSOCIATIONS_KEY, COMMAND, DEFAULTS_TABLE, GUARD_TABLE, NOTIFY_KEY};

pub fn initial(defaults: &BTreeMap<String, String>) -> String {
    let mut table = Table::new();
    for (mime, app) in defaults {
        table[mime.as_str()] = value(app.as_str());
    }

    let mut guard = Table::new();
    guard[NOTIFY_KEY] = value(true);
    guard[ADDED_ASSOCIATIONS_KEY] = value(true);

    let mut document = DocumentMut::new();
    document[DEFAULTS_TABLE] = Item::Table(table);
    document[GUARD_TABLE] = Item::Table(guard);
    document.to_string()
}

pub fn set_default(text: &str, mime: &str, app: &str) -> Result<String> {
    let mut document = document(text)?;
    let defaults = document[DEFAULTS_TABLE].or_insert(Item::Table(Table::new()));
    let Some(table) = defaults.as_table_mut() else {
        anyhow::bail!("{COMMAND}: [{DEFAULTS_TABLE}] is not a table");
    };
    table[mime] = value(app);
    Ok(document.to_string())
}

pub fn set_notify(text: &str, on: bool) -> Result<String> {
    let mut document = document(text)?;
    let guard = document[GUARD_TABLE].or_insert(Item::Table(Table::new()));
    let Some(table) = guard.as_table_mut() else {
        anyhow::bail!("{COMMAND}: [{GUARD_TABLE}] is not a table");
    };
    table[NOTIFY_KEY] = value(on);
    Ok(document.to_string())
}

pub fn unset_default(text: &str, mime: &str) -> Result<Option<String>> {
    let mut document = document(text)?;
    let removed = document
        .get_mut(DEFAULTS_TABLE)
        .and_then(Item::as_table_mut)
        .and_then(|table| table.remove(mime));
    Ok(removed.map(|_| document.to_string()))
}

fn document(text: &str) -> Result<DocumentMut> {
    text.parse()
        .with_context(|| format!("{COMMAND}: the configuration is not valid TOML"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse;

    #[test]
    fn the_initial_file_reads_back() {
        let defaults = BTreeMap::from([
            ("text/html".to_string(), "firefox.desktop".to_string()),
            ("image/png".to_string(), "loupe.desktop".to_string()),
        ]);

        let config = parse(&initial(&defaults)).unwrap();

        assert_eq!(config.defaults, defaults);
        assert!(config.guard.notify);
    }

    #[test]
    fn setting_keeps_the_comments() {
        let text = "# mine\n[defaults]\n\"text/html\" = \"chrome.desktop\" # old\n";

        let out = set_default(text, "text/html", "firefox.desktop").unwrap();
        let out = set_default(&out, "image/*", "loupe.desktop").unwrap();

        assert!(out.starts_with("# mine\n"));
        let config = parse(&out).unwrap();
        assert_eq!(config.defaults["text/html"], "firefox.desktop");
        assert_eq!(config.defaults["image/*"], "loupe.desktop");
    }

    #[test]
    fn setting_creates_the_table() {
        let out = set_default("", "text/html", "firefox.desktop").unwrap();

        assert_eq!(
            parse(&out).unwrap().defaults["text/html"],
            "firefox.desktop"
        );
    }

    #[test]
    fn notify_is_switched_keeping_the_comments() {
        let text = "# mine\n[guard]\nnotify = true # keep\ndesktops = [\"kde\"]\n";

        let off = set_notify(text, false).unwrap();

        assert!(off.starts_with("# mine\n"));
        let config = parse(&off).unwrap();
        assert!(!config.guard.notify);
        assert_eq!(config.guard.desktops, ["kde"]);
        assert!(
            parse(&set_notify(&off, true).unwrap())
                .unwrap()
                .guard
                .notify
        );
        assert!(!parse(&set_notify("", false).unwrap()).unwrap().guard.notify);
    }

    #[test]
    fn unsetting_reports_whether_the_type_was_there() {
        let text = "[defaults]\n\"text/html\" = \"firefox.desktop\"\n";

        let out = unset_default(text, "text/html").unwrap().unwrap();

        assert!(parse(&out).unwrap().defaults.is_empty());
        assert!(unset_default(text, "image/png").unwrap().is_none());
    }
}
