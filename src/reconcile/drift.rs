use std::collections::BTreeMap;
use std::path::PathBuf;

use super::types::{Drift, FileEdit, Plan, Skipped};
use crate::config::Guard;
use crate::mimeapps::{self, MimeApps, render};
use crate::system::System;

struct UserFile {
    path: PathBuf,
    generic: bool,
    before: Option<String>,
    file: MimeApps,
}

pub fn plan(system: &System, wanted: &BTreeMap<String, String>, guard: &Guard) -> Plan {
    let mut files = user_files(system);
    let mut plan = Plan::default();

    for (mime, app) in wanted {
        if !system.installed(app) {
            plan.skipped.push(Skipped {
                mime: mime.clone(),
                app: app.clone(),
            });
            continue;
        }
        let keys = system.keys(mime);
        let touched: Vec<PathBuf> = files
            .iter_mut()
            .filter_map(|user| pin(user, &keys, app, guard).then(|| user.path.clone()))
            .collect();
        if touched.is_empty() {
            continue;
        }
        plan.drifts.push(Drift {
            mime: mime.clone(),
            wanted: app.clone(),
            found: system.effective(mime),
            files: touched,
        });
    }

    plan.files = files.into_iter().filter_map(edit).collect();
    plan
}

fn user_files(system: &System) -> Vec<UserFile> {
    let generic = mimeapps::generic_user_file(&system.xdg);
    mimeapps::user_files(&system.xdg)
        .into_iter()
        .map(|path| {
            let layer = system.layers.iter().find(|layer| layer.source.path == path);
            UserFile {
                generic: path == generic,
                before: layer.map(|layer| render(&layer.file)),
                file: layer.map(|layer| layer.file.clone()).unwrap_or_default(),
                path,
            }
        })
        .collect()
}

fn pin(user: &mut UserFile, keys: &[String], app: &str, guard: &Guard) -> bool {
    let mut changed = false;
    for (index, key) in keys.iter().enumerate() {
        let created = index == 0 && user.generic;
        changed |= pin_key(&mut user.file, key, app, guard, created);
    }
    changed
}

fn pin_key(file: &mut MimeApps, key: &str, app: &str, guard: &Guard, created: bool) -> bool {
    let mut changed = false;
    if created || !file.defaults(key).is_empty() {
        changed |= file.set_default(key, app);
    }
    if guard.added_associations && !file.added(key).is_empty() {
        changed |= file.promote_added(key, app);
    }
    changed |= file.unremove(key, app);
    changed
}

fn edit(user: UserFile) -> Option<FileEdit> {
    let after = render(&user.file);
    if user.before.as_deref().unwrap_or_default() == after {
        return None;
    }
    Some(FileEdit {
        path: user.path,
        before: user.before,
        after,
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::desktop::{App, Installed};
    use crate::mimeapps::{Layer, Source, parse};
    use crate::mimedb::MimeDb;
    use crate::utils::Xdg;

    fn system(layers: &[(&str, &str)], apps: &[&str]) -> System {
        let xdg = Xdg {
            config_home: "/c".into(),
            config_dirs: vec![],
            data_home: "/d".into(),
            data_dirs: vec![],
            state_home: "/s".into(),
            desktops: vec!["kde".into()],
        };
        let installed = Installed {
            apps: apps
                .iter()
                .map(|id| {
                    let app = App {
                        path: Path::new("/apps").join(id),
                        hidden: false,
                        mime_types: vec![],
                        program: None,
                    };
                    (id.to_string(), app)
                })
                .collect(),
        };
        let layers = layers
            .iter()
            .map(|(path, text)| Layer {
                source: Source {
                    path: PathBuf::from(path),
                    writable: path.starts_with("/c/"),
                },
                file: parse(text),
            })
            .collect();
        System {
            xdg,
            installed,
            db: MimeDb::default(),
            layers,
        }
    }

    fn wanted(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(mime, app)| (mime.to_string(), app.to_string()))
            .collect()
    }

    #[test]
    fn a_desktop_override_is_put_back_everywhere_it_shows() {
        let system = system(
            &[
                (
                    "/c/kde-mimeapps.list",
                    "[Default Applications]\ntext/html=chrome.desktop;\n",
                ),
                (
                    "/c/mimeapps.list",
                    "[Default Applications]\ntext/html=firefox.desktop;\n[Added Associations]\ntext/html=firefox.desktop;\n",
                ),
            ],
            &["firefox.desktop", "chrome.desktop"],
        );

        let plan = plan(
            &system,
            &wanted(&[("text/html", "firefox.desktop")]),
            &Guard::default(),
        );

        assert_eq!(plan.files.len(), 1);
        assert_eq!(plan.files[0].path, PathBuf::from("/c/kde-mimeapps.list"));
        assert_eq!(
            plan.files[0].after,
            "[Default Applications]\ntext/html=firefox.desktop;\n"
        );
        let drift = &plan.drifts[0];
        assert!(drift.overridden());
        assert_eq!(drift.previous(), Some("chrome.desktop"));
    }

    #[test]
    fn a_missing_pin_is_written_to_the_generic_file() {
        let system = system(
            &[(
                "/d/applications/mimeapps.list",
                "[Default Applications]\nimage/png=loupe.desktop;\n",
            )],
            &["loupe.desktop"],
        );

        let plan = plan(
            &system,
            &wanted(&[("image/png", "loupe.desktop")]),
            &Guard::default(),
        );

        assert_eq!(plan.files.len(), 1);
        assert_eq!(plan.files[0].path, PathBuf::from("/c/mimeapps.list"));
        assert_eq!(plan.files[0].before, None);
        assert_eq!(
            plan.files[0].after,
            "[Default Applications]\nimage/png=loupe.desktop;\n"
        );
        assert_eq!(plan.unpinned().count(), 1);
        assert_eq!(plan.overridden().count(), 0);
    }

    #[test]
    fn a_settled_system_needs_nothing() {
        let system = system(
            &[(
                "/c/mimeapps.list",
                "[Default Applications]\ntext/html=firefox.desktop;\n",
            )],
            &["firefox.desktop"],
        );
        let guard = Guard {
            added_associations: false,
            ..Guard::default()
        };

        let plan = plan(
            &system,
            &wanted(&[("text/html", "firefox.desktop")]),
            &guard,
        );

        assert!(plan.is_clean());
        assert!(plan.drifts.is_empty());
    }

    #[test]
    fn an_application_that_is_not_installed_is_skipped() {
        let system = system(&[], &[]);

        let plan = plan(
            &system,
            &wanted(&[("text/html", "firefox.desktop")]),
            &Guard::default(),
        );

        assert!(plan.is_clean());
        assert_eq!(plan.skipped[0].app, "firefox.desktop");
    }

    #[test]
    fn a_removed_association_is_lifted() {
        let system = system(
            &[(
                "/c/mimeapps.list",
                "[Default Applications]\ntext/html=firefox.desktop;\n[Removed Associations]\ntext/html=firefox.desktop;\n",
            )],
            &["firefox.desktop"],
        );
        let guard = Guard {
            added_associations: false,
            ..Guard::default()
        };

        let plan = plan(
            &system,
            &wanted(&[("text/html", "firefox.desktop")]),
            &guard,
        );

        assert_eq!(
            plan.files[0].after,
            "[Default Applications]\ntext/html=firefox.desktop;\n[Removed Associations]\n"
        );
    }
}
