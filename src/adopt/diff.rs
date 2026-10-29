use anyhow::{Context, Result};

use super::constants::COMMAND;
use super::types::{Adoption, Change};
use crate::config;
use crate::mimeapps::MimeApps;

pub fn changes(before: &[MimeApps], after: &[MimeApps]) -> Vec<Change> {
    let mut mimes: Vec<String> = Vec::new();
    for mime in before.iter().chain(after).flat_map(MimeApps::default_types) {
        if !mimes.contains(&mime) {
            mimes.push(mime);
        }
    }

    mimes
        .into_iter()
        .filter_map(|mime| {
            let before = first(before, &mime);
            let after = first(after, &mime);
            (before != after).then_some(Change {
                mime,
                before,
                after,
            })
        })
        .collect()
}

pub fn adopt(text: &str, changes: Vec<Change>) -> Result<Adoption> {
    let mut adoption = Adoption {
        text: text.to_string(),
        ..Adoption::default()
    };

    for change in changes {
        if let Err(err) = config::validate(&change.mime) {
            adoption.refused.push((change, err.to_string()));
            continue;
        }
        let edited = match &change.after {
            Some(app) => Some(config::set_default(&adoption.text, &change.mime, app)?),
            None => config::unset_default(&adoption.text, &change.mime)?,
        };
        let Some(edited) = edited else {
            continue;
        };
        adoption.text = edited;
        adoption.adopted.push(change);
    }

    config::parse(&adoption.text)
        .with_context(|| format!("{COMMAND}: the result is not a valid configuration"))?;
    Ok(adoption)
}

fn first(files: &[MimeApps], mime: &str) -> Option<String> {
    files
        .iter()
        .find_map(|file| file.defaults(mime).into_iter().next())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mimeapps::parse;

    fn change(mime: &str, before: Option<&str>, after: Option<&str>) -> Change {
        Change {
            mime: mime.to_string(),
            before: before.map(str::to_string),
            after: after.map(str::to_string),
        }
    }

    #[test]
    fn only_the_first_default_of_each_type_is_compared() {
        let before = [parse(
            "[Default Applications]\ntext/html=chrome.desktop;\nimage/png=loupe.desktop;\ntext/plain=nvim.desktop;\n",
        )];
        let after = [parse(
            "[Added Associations]\ntext/html=chrome.desktop;\n[Default Applications]\nimage/png=loupe.desktop;gimp.desktop;\ntext/html=firefox.desktop;\nimage/*=feh.desktop;\n",
        )];

        assert_eq!(
            changes(&before, &after),
            [
                change("text/html", Some("chrome.desktop"), Some("firefox.desktop")),
                change("text/plain", Some("nvim.desktop"), None),
                change("image/*", None, Some("feh.desktop")),
            ]
        );
    }

    #[test]
    fn an_earlier_file_decides_the_default() {
        let desktop = parse("[Default Applications]\ntext/html=chrome.desktop;\n");
        let generic = parse("[Default Applications]\nimage/png=loupe.desktop;\n");
        let edited_generic =
            parse("[Default Applications]\ntext/html=firefox.desktop;\nimage/png=gimp.desktop;\n");

        assert_eq!(
            changes(
                &[desktop.clone(), generic],
                &[desktop, edited_generic.clone()]
            ),
            [change(
                "image/png",
                Some("loupe.desktop"),
                Some("gimp.desktop")
            )]
        );
        assert_eq!(
            changes(
                &[
                    parse("[Default Applications]\ntext/html=chrome.desktop;\n"),
                    edited_generic.clone()
                ],
                &[parse(""), edited_generic]
            ),
            [change(
                "text/html",
                Some("chrome.desktop"),
                Some("firefox.desktop")
            )]
        );
    }

    #[test]
    fn changes_land_in_the_configuration_with_its_comments() {
        let text = "# mine\n[defaults]\n\"text/html\" = \"chrome.desktop\"\n\"text/plain\" = \"nvim.desktop\"\n";

        let adoption = adopt(
            text,
            vec![
                change("text/html", Some("chrome.desktop"), Some("firefox.desktop")),
                change("text/plain", Some("nvim.desktop"), None),
                change("image/*", None, Some("feh.desktop")),
            ],
        )
        .unwrap();

        assert!(adoption.text.starts_with("# mine\n"));
        let config = config::parse(&adoption.text).unwrap();
        assert_eq!(config.defaults["text/html"], "firefox.desktop");
        assert_eq!(config.defaults["image/*"], "feh.desktop");
        assert!(!config.defaults.contains_key("text/plain"));
        assert_eq!(adoption.adopted.len(), 3);
        assert!(adoption.refused.is_empty());
    }

    #[test]
    fn unsetting_an_unprotected_type_adopts_nothing() {
        let adoption = adopt("", vec![change("text/plain", Some("nvim.desktop"), None)]).unwrap();

        assert!(adoption.is_empty());
        assert_eq!(adoption.text, "");
    }

    #[test]
    fn patterns_the_configuration_cannot_hold_are_refused() {
        let adoption = adopt(
            "",
            vec![
                change("image/p*", None, Some("feh.desktop")),
                change("text/html", None, Some("firefox.desktop")),
            ],
        )
        .unwrap();

        assert_eq!(adoption.refused.len(), 1);
        assert_eq!(adoption.refused[0].0.mime, "image/p*");
        assert_eq!(
            config::parse(&adoption.text).unwrap().defaults["text/html"],
            "firefox.desktop"
        );
    }
}
