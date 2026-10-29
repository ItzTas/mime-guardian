use super::constants::{
    ADDED_ASSOCIATIONS, DEFAULT_APPLICATIONS, KEY_SEPARATOR, NEWLINE, REMOVED_ASSOCIATIONS,
    VALUE_SEPARATOR,
};
use super::parse::line;
use super::types::{Kind, Line, MimeApps};

impl MimeApps {
    pub fn values(&self, section: &str, key: &str) -> Vec<String> {
        self.positions(section, key)
            .last()
            .map(|&index| split(self.value_at(index)))
            .unwrap_or_default()
    }

    pub fn keys(&self, section: &str) -> Vec<String> {
        let mut keys: Vec<String> = Vec::new();
        for (key, _) in self.entries(section) {
            if !keys.iter().any(|known| known == key) {
                keys.push(key.to_string());
            }
        }
        keys
    }

    pub fn set_values(&mut self, section: &str, key: &str, values: &[String]) {
        let positions = self.positions(section, key);
        let Some((&first, rest)) = positions.split_first() else {
            if !values.is_empty() {
                self.insert(section, entry(key, values));
            }
            return;
        };

        for &index in rest.iter().rev() {
            self.lines.remove(index);
        }
        if values.is_empty() {
            self.lines.remove(first);
            return;
        }
        let ending = ending(&self.lines[first].raw).to_string();
        let mut replaced = entry(key, values);
        replaced.raw = format!("{}{ending}", replaced.raw.trim_end());
        self.lines[first] = replaced;
    }

    pub fn defaults(&self, mime: &str) -> Vec<String> {
        self.values(DEFAULT_APPLICATIONS, mime)
    }

    pub fn added(&self, mime: &str) -> Vec<String> {
        self.values(ADDED_ASSOCIATIONS, mime)
    }

    pub fn removed(&self, mime: &str) -> Vec<String> {
        self.values(REMOVED_ASSOCIATIONS, mime)
    }

    pub fn default_types(&self) -> Vec<String> {
        self.keys(DEFAULT_APPLICATIONS)
    }

    pub fn set_default(&mut self, mime: &str, app: &str) -> bool {
        let wanted = [app.to_string()];
        if self.defaults(mime) == wanted {
            return false;
        }
        self.set_values(DEFAULT_APPLICATIONS, mime, &wanted);
        true
    }

    pub fn promote_added(&mut self, mime: &str, app: &str) -> bool {
        let mut apps = self.added(mime);
        if apps.first().is_some_and(|first| first == app) {
            return false;
        }
        apps.retain(|known| known != app);
        apps.insert(0, app.to_string());
        self.set_values(ADDED_ASSOCIATIONS, mime, &apps);
        true
    }

    pub fn unremove(&mut self, mime: &str, app: &str) -> bool {
        let mut apps = self.removed(mime);
        if !apps.iter().any(|known| known == app) {
            return false;
        }
        apps.retain(|known| known != app);
        self.set_values(REMOVED_ASSOCIATIONS, mime, &apps);
        true
    }

    fn entries<'a>(&'a self, section: &'a str) -> impl Iterator<Item = (&'a str, &'a str)> {
        let mut current: Option<&str> = None;
        self.lines.iter().filter_map(move |line| match &line.kind {
            Kind::Header(name) => {
                current = Some(name);
                None
            }
            Kind::Entry { key, value } if current == Some(section) => {
                Some((key.as_str(), value.as_str()))
            }
            _ => None,
        })
    }

    fn positions(&self, section: &str, key: &str) -> Vec<usize> {
        let mut current: Option<&str> = None;
        let mut found = Vec::new();
        for (index, line) in self.lines.iter().enumerate() {
            match &line.kind {
                Kind::Header(name) => current = Some(name),
                Kind::Entry { key: name, .. } if current == Some(section) && name == key => {
                    found.push(index)
                }
                _ => {}
            }
        }
        found
    }

    fn value_at(&self, index: usize) -> &str {
        match &self.lines[index].kind {
            Kind::Entry { value, .. } => value,
            _ => "",
        }
    }

    fn insert(&mut self, section: &str, new: Line) {
        let Some(at) = self.section_tail(section) else {
            return self.append_section(section, new);
        };
        self.terminate(at);
        self.lines.insert(at, new);
    }

    fn section_tail(&self, section: &str) -> Option<usize> {
        let header = self
            .lines
            .iter()
            .rposition(|line| matches!(&line.kind, Kind::Header(name) if name == section))?;

        let mut tail = header + 1;
        for (index, line) in self.lines.iter().enumerate().skip(header + 1) {
            match line.kind {
                Kind::Header(_) => break,
                Kind::Entry { .. } => tail = index + 1,
                Kind::Other => {}
            }
        }
        Some(tail)
    }

    fn append_section(&mut self, section: &str, new: Line) {
        let at = self.lines.len();
        self.terminate(at);
        if self
            .lines
            .last()
            .is_some_and(|last| !last.raw.trim().is_empty())
        {
            self.lines.push(line(NEWLINE));
        }
        self.lines.push(line(&format!("[{section}]{NEWLINE}")));
        self.lines.push(new);
    }

    fn terminate(&mut self, at: usize) {
        let Some(previous) = at
            .checked_sub(1)
            .and_then(|index| self.lines.get_mut(index))
        else {
            return;
        };
        if !previous.raw.ends_with('\n') {
            previous.raw.push_str(NEWLINE);
        }
    }
}

fn entry(key: &str, values: &[String]) -> Line {
    let mut joined = values.join(&VALUE_SEPARATOR.to_string());
    joined.push(VALUE_SEPARATOR);
    line(&format!("{key}{KEY_SEPARATOR}{joined}{NEWLINE}"))
}

fn ending(raw: &str) -> &str {
    let body = raw.trim_end_matches(['\r', '\n']);
    &raw[body.len()..]
}

fn split(value: &str) -> Vec<String> {
    value
        .split(VALUE_SEPARATOR)
        .map(str::trim)
        .filter(|app| !app.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::mimeapps::{parse, render};

    fn edited(text: &str, edit: impl FnOnce(&mut crate::mimeapps::MimeApps)) -> String {
        let mut file = parse(text);
        edit(&mut file);
        render(&file)
    }

    #[test]
    fn a_default_is_replaced_in_place() {
        let text =
            "# top\n[Default Applications]\ntext/html=chrome.desktop;\r\nimage/png=a.desktop;\n";

        let out = edited(text, |file| {
            assert!(file.set_default("text/html", "firefox.desktop"));
        });

        assert_eq!(
            out,
            "# top\n[Default Applications]\ntext/html=firefox.desktop;\r\nimage/png=a.desktop;\n"
        );
    }

    #[test]
    fn an_unchanged_default_reports_nothing() {
        let mut file = parse("[Default Applications]\ntext/html=firefox.desktop;\n");

        assert!(!file.set_default("text/html", "firefox.desktop"));
    }

    #[test]
    fn a_new_entry_lands_after_the_section_entries() {
        let text = "[Default Applications]\na/b=x.desktop;\n\n# trailing note\n[Other]\nk=v\n";

        let out = edited(text, |file| {
            file.set_default("text/html", "firefox.desktop");
        });

        assert_eq!(
            out,
            "[Default Applications]\na/b=x.desktop;\ntext/html=firefox.desktop;\n\n# trailing note\n[Other]\nk=v\n"
        );
    }

    #[test]
    fn a_missing_section_is_appended() {
        let out = edited("[Other]\nk=v", |file| {
            file.set_default("text/html", "firefox.desktop");
        });

        assert_eq!(
            out,
            "[Other]\nk=v\n\n[Default Applications]\ntext/html=firefox.desktop;\n"
        );
        assert_eq!(
            edited("", |file| {
                file.set_default("text/html", "firefox.desktop");
            }),
            "[Default Applications]\ntext/html=firefox.desktop;\n"
        );
    }

    #[test]
    fn duplicates_collapse_into_the_first_line() {
        let text = "[Default Applications]\ntext/html=a.desktop;\ntext/html=b.desktop;\n";
        let file = parse(text);
        assert_eq!(file.defaults("text/html"), ["b.desktop"]);

        let out = edited(text, |file| {
            file.set_default("text/html", "c.desktop");
        });

        assert_eq!(out, "[Default Applications]\ntext/html=c.desktop;\n");
    }

    #[test]
    fn an_association_moves_to_the_front() {
        let out = edited(
            "[Added Associations]\ntext/html=chrome.desktop;firefox.desktop;\n",
            |file| {
                assert!(file.promote_added("text/html", "firefox.desktop"));
                assert!(!file.promote_added("text/html", "firefox.desktop"));
            },
        );

        assert_eq!(
            out,
            "[Added Associations]\ntext/html=firefox.desktop;chrome.desktop;\n"
        );
    }

    #[test]
    fn a_removal_is_lifted_and_an_empty_key_disappears() {
        let out = edited(
            "[Removed Associations]\ntext/html=firefox.desktop;\nimage/png=a.desktop;b.desktop;\n",
            |file| {
                assert!(file.unremove("text/html", "firefox.desktop"));
                assert!(file.unremove("image/png", "a.desktop"));
                assert!(!file.unremove("image/png", "a.desktop"));
            },
        );

        assert_eq!(out, "[Removed Associations]\nimage/png=b.desktop;\n");
    }

    #[test]
    fn default_types_are_listed_once_in_order() {
        let file = parse("[Default Applications]\nb=x;\na=y;\nb=z;\n[Added Associations]\nc=w;\n");

        assert_eq!(file.default_types(), ["b", "a"]);
    }
}
