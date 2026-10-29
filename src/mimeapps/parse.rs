use super::constants::{COMMENT, KEY_SEPARATOR};
use super::types::{Kind, Line, MimeApps};

pub fn parse(text: &str) -> MimeApps {
    MimeApps {
        lines: text.split_inclusive('\n').map(line).collect(),
    }
}

pub(super) fn line(raw: &str) -> Line {
    Line {
        raw: raw.to_string(),
        kind: kind(raw.trim()),
    }
}

fn kind(trimmed: &str) -> Kind {
    if trimmed.is_empty() || trimmed.starts_with(COMMENT) {
        return Kind::Other;
    }
    if let Some(name) = trimmed
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    {
        return Kind::Header(name.trim().to_string());
    }
    let Some((key, value)) = trimmed.split_once(KEY_SEPARATOR) else {
        return Kind::Other;
    };

    Kind::Entry {
        key: key.trim().to_string(),
        value: value.trim().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mimeapps::render;

    const FIXTURE: &str = "# managed by hand\n\
\n\
[Default Applications]\n\
text/html=firefox.desktop;\n\
x-scheme-handler/https = firefox.desktop;chromium.desktop\n\
\n\
# keep this one\n\
[X-Custom Section]\n\
weird line without separator\n\
key=value\r\n\
[Added Associations]\n\
image/png=org.gnome.Loupe.desktop;";

    #[test]
    fn rendering_gives_back_the_same_bytes() {
        assert_eq!(render(&parse(FIXTURE)), FIXTURE);
        assert_eq!(render(&parse("")), "");
        assert_eq!(render(&parse("\n\n")), "\n\n");
    }

    #[test]
    fn entries_are_read_with_their_section() {
        let file = parse(FIXTURE);

        assert_eq!(
            file.values("Default Applications", "x-scheme-handler/https"),
            ["firefox.desktop", "chromium.desktop"]
        );
        assert_eq!(
            file.values("Added Associations", "image/png"),
            ["org.gnome.Loupe.desktop"]
        );
        assert_eq!(file.values("X-Custom Section", "key"), ["value"]);
        assert!(file.values("Default Applications", "image/png").is_empty());
    }
}
