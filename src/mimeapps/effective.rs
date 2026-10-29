use super::types::{Effective, Layer, Origin};

pub fn effective(
    layers: &[Layer],
    mime: &str,
    installed: impl Fn(&str) -> bool,
) -> Option<Effective> {
    from_defaults(layers, mime, &installed).or_else(|| from_added(layers, mime, &installed))
}

fn from_defaults(
    layers: &[Layer],
    mime: &str,
    installed: &impl Fn(&str) -> bool,
) -> Option<Effective> {
    layers.iter().find_map(|layer| {
        let app = layer
            .file
            .defaults(mime)
            .into_iter()
            .find(|app| installed(app))?;
        Some(Effective {
            app,
            path: layer.source.path.clone(),
            origin: Origin::Default,
        })
    })
}

fn from_added(
    layers: &[Layer],
    mime: &str,
    installed: &impl Fn(&str) -> bool,
) -> Option<Effective> {
    let mut removed: Vec<String> = Vec::new();
    for layer in layers {
        removed.extend(layer.file.removed(mime));
        let found = layer
            .file
            .added(mime)
            .into_iter()
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

        let found = effective(&layers, "text/html", everything).unwrap();

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

        let found = effective(&layers, "text/html", |app| app != "gone.desktop").unwrap();

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

        let found = effective(&layers, "text/html", everything).unwrap();

        assert_eq!(found.app, "firefox.desktop");
        assert_eq!(found.origin, Origin::Added);
        assert!(effective(&layers, "image/png", everything).is_none());
    }
}
