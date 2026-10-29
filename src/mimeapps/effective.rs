use super::types::{Effective, Layer, Origin};

pub fn effective(
    layers: &[Layer],
    keys: &[String],
    installed: impl Fn(&str) -> bool,
) -> Option<Effective> {
    from_defaults(layers, keys, &installed).or_else(|| from_added(layers, keys, &installed))
}

fn each<'a>(
    keys: &'a [String],
    read: impl Fn(&str) -> Vec<String> + 'a,
) -> impl Iterator<Item = String> + 'a {
    keys.iter().flat_map(move |key| read(key))
}

fn from_defaults(
    layers: &[Layer],
    keys: &[String],
    installed: &impl Fn(&str) -> bool,
) -> Option<Effective> {
    layers.iter().find_map(|layer| {
        let app = each(keys, |key| layer.file.defaults(key)).find(|app| installed(app))?;
        Some(Effective {
            app,
            path: layer.source.path.clone(),
            origin: Origin::Default,
        })
    })
}

fn from_added(
    layers: &[Layer],
    keys: &[String],
    installed: &impl Fn(&str) -> bool,
) -> Option<Effective> {
    let mut removed: Vec<String> = Vec::new();
    for layer in layers {
        removed.extend(each(keys, |key| layer.file.removed(key)));
        let found = each(keys, |key| layer.file.added(key))
            .find(|app| installed(app) && !removed.contains(app));
        let Some(app) = found else {
            continue;
        };
        return Some(Effective {
            app,
            path: layer.source.path.clone(),
            origin: Origin::Added,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::mimeapps::{Source, parse};

    fn layer(path: &str, text: &str) -> Layer {
        Layer {
            source: Source {
                path: PathBuf::from(path),
                writable: true,
            },
            file: parse(text),
        }
    }

    fn everything(_: &str) -> bool {
        true
    }

    fn keys(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    #[test]
    fn an_alias_entry_answers_for_its_canonical_type() {
        let layers = [layer(
            "/user",
            "[Default Applications]\nvideo/x-matroska=mpv.desktop\n",
        )];

        let found = effective(
            &layers,
            &keys(&["video/matroska", "video/x-matroska"]),
            everything,
        );

        assert_eq!(found.unwrap().app, "mpv.desktop");
    }

    #[test]
    fn the_first_layer_with_a_default_wins() {
        let layers = [
            layer(
                "/kde",
                "[Default Applications]\ntext/html=chrome.desktop;\n",
            ),
            layer(
                "/generic",
                "[Default Applications]\ntext/html=firefox.desktop;\n",
            ),
        ];

        let found = effective(&layers, &keys(&["text/html"]), everything).unwrap();

        assert_eq!(found.app, "chrome.desktop");
        assert_eq!(found.path, PathBuf::from("/kde"));
        assert_eq!(found.origin, Origin::Default);
    }

    #[test]
    fn an_uninstalled_default_falls_through() {
        let layers = [
            layer("/kde", "[Default Applications]\ntext/html=gone.desktop;\n"),
            layer(
                "/generic",
                "[Default Applications]\ntext/html=firefox.desktop;\n",
            ),
        ];

        let found = effective(&layers, &keys(&["text/html"]), |app| app != "gone.desktop").unwrap();

        assert_eq!(found.app, "firefox.desktop");
    }

    #[test]
    fn associations_answer_when_no_default_exists() {
        let layers = [
            layer(
                "/user",
                "[Removed Associations]\ntext/html=chrome.desktop;\n",
            ),
            layer(
                "/system",
                "[Added Associations]\ntext/html=chrome.desktop;firefox.desktop;\n",
            ),
        ];

        let found = effective(&layers, &keys(&["text/html"]), everything).unwrap();

        assert_eq!(found.app, "firefox.desktop");
        assert_eq!(found.origin, Origin::Added);
        assert!(effective(&layers, &keys(&["image/png"]), everything).is_none());
    }
}
