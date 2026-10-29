use std::collections::{BTreeMap, BTreeSet};

use super::constants::{MEDIA_SEPARATOR, WILDCARD};

pub fn expand(
    defaults: &BTreeMap<String, String>,
    known: &BTreeSet<String>,
    canonical: impl Fn(&str) -> String,
) -> BTreeMap<String, String> {
    let mut wanted = BTreeMap::new();

    for (pattern, app) in defaults {
        let Some(media) = wildcard_media(pattern) else {
            continue;
        };
        for mime in known.iter().filter(|mime| in_media(mime, media)) {
            wanted.insert(mime.clone(), app.clone());
        }
    }
    for (mime, app) in defaults {
        if wildcard_media(mime).is_some() {
            continue;
        }
        wanted.insert(canonical(mime), app.clone());
    }

    wanted
}

fn wildcard_media(pattern: &str) -> Option<&str> {
    let (media, subtype) = pattern.split_once(MEDIA_SEPARATOR)?;
    (subtype == WILDCARD).then_some(media)
}

fn in_media(mime: &str, media: &str) -> bool {
    mime.split_once(MEDIA_SEPARATOR)
        .is_some_and(|(found, _)| found == media)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    #[test]
    fn an_exact_type_beats_its_wildcard() {
        let known: BTreeSet<String> = ["image/png", "image/jpeg", "text/plain"]
            .map(String::from)
            .into();

        let wanted = expand(
            &map(&[("image/*", "loupe.desktop"), ("image/png", "gimp.desktop")]),
            &known,
            str::to_string,
        );

        assert_eq!(
            wanted,
            map(&[
                ("image/jpeg", "loupe.desktop"),
                ("image/png", "gimp.desktop")
            ])
        );
    }

    #[test]
    fn exact_types_are_kept_even_when_unknown_and_canonicalized() {
        let wanted = expand(
            &map(&[("application/x-pdf", "okular.desktop")]),
            &BTreeSet::new(),
            |mime| match mime {
                "application/x-pdf" => "application/pdf".to_string(),
                other => other.to_string(),
            },
        );

        assert_eq!(wanted, map(&[("application/pdf", "okular.desktop")]));
    }
}
