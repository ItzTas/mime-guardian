use super::constants::USERAPP_PREFIX;
use super::types::Installed;

impl Installed {
    pub fn canonical(&self, id: &str) -> String {
        self.packaged_twin(id).unwrap_or(id).to_string()
    }

    fn packaged_twin(&self, id: &str) -> Option<&str> {
        if !id.starts_with(USERAPP_PREFIX) {
            return None;
        }
        let program = self.apps.get(id)?.program.as_deref()?;
        let mut twins = self.apps.iter().filter(|(other, app)| {
            !other.starts_with(USERAPP_PREFIX)
                && !app.hidden
                && app.program.as_deref() == Some(program)
        });
        let (twin, _) = twins.next()?;
        twins.next().is_none().then_some(twin.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const USERAPP: &str = "userapp-Thunderbird-LBD2P3.desktop";

    fn installed(entries: &[(&str, &str)]) -> Installed {
        let root = tempfile::tempdir().unwrap();
        for (id, text) in entries {
            std::fs::write(root.path().join(id), text).unwrap();
        }
        Installed::scan(&[root.path().to_path_buf()])
    }

    fn entry(exec: &str) -> String {
        format!("[Desktop Entry]\nName=Thunderbird\nExec={exec}\n")
    }

    fn userapp() -> (&'static str, String) {
        (USERAPP, entry("/usr/lib/thunderbird/thunderbird %u"))
    }

    #[test]
    fn a_userapp_entry_becomes_the_packaged_entry_that_runs_the_same_program() {
        let (id, text) = userapp();
        let installed = installed(&[
            (id, &text),
            ("org.mozilla.Thunderbird.desktop", &entry("thunderbird %u")),
            ("firefox.desktop", &entry("/usr/lib/firefox/firefox %u")),
        ]);

        assert_eq!(
            installed.canonical(USERAPP),
            "org.mozilla.Thunderbird.desktop"
        );
    }

    #[test]
    fn a_quoted_program_path_is_matched_by_its_file_name() {
        let (id, text) = userapp();
        let installed = installed(&[
            (id, &text),
            (
                "thunderbird.desktop",
                &entry("\"/opt/Thunder Bird/thunderbird\" %u"),
            ),
        ]);

        assert_eq!(installed.canonical(USERAPP), "thunderbird.desktop");
    }

    #[test]
    fn a_userapp_entry_stays_when_several_entries_run_the_program() {
        let (id, text) = userapp();
        let installed = installed(&[
            (id, &text),
            ("org.mozilla.Thunderbird.desktop", &entry("thunderbird %u")),
            (
                "thunderbird-compose.desktop",
                &entry("thunderbird -compose"),
            ),
        ]);

        assert_eq!(installed.canonical(USERAPP), USERAPP);
    }

    #[test]
    fn a_userapp_entry_stays_when_no_packaged_entry_runs_the_program() {
        let (id, text) = userapp();
        let installed = installed(&[
            (id, &text),
            (
                "userapp-Thunderbird-OSAYP3.desktop",
                &entry("/usr/lib/thunderbird/thunderbird %u"),
            ),
            (
                "org.mozilla.Thunderbird.desktop",
                "[Desktop Entry]\nExec=thunderbird %u\nHidden=true\n",
            ),
        ]);

        assert_eq!(installed.canonical(USERAPP), USERAPP);
    }

    #[test]
    fn entries_that_are_not_userapp_entries_are_kept() {
        let installed = installed(&[
            ("thunderbird.desktop", &entry("thunderbird %u")),
            ("org.mozilla.Thunderbird.desktop", &entry("thunderbird %u")),
        ]);

        assert_eq!(
            installed.canonical("thunderbird.desktop"),
            "thunderbird.desktop"
        );
        assert_eq!(installed.canonical("gone.desktop"), "gone.desktop");
    }
}
