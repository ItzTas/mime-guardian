use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;

use super::types::System;
use crate::config::{Config, expand};
use crate::desktop::{Installed, app_dirs};
use crate::mimeapps::{self, Effective};
use crate::mimedb::MimeDb;
use crate::utils::Xdg;

const WILDCARD: char = '*';

impl System {
    pub fn load(xdg: Xdg) -> Result<Self> {
        let installed = Installed::scan(&app_dirs(&xdg));
        let db = MimeDb::load(&xdg)?;
        let layers = mimeapps::load(mimeapps::sources(&xdg))?;

        Ok(Self {
            xdg,
            installed,
            db,
            layers,
        })
    }

    pub fn installed(&self, app: &str) -> bool {
        self.installed.contains(app)
    }

    pub fn keys(&self, mime: &str) -> Vec<String> {
        std::iter::once(mime.to_string())
            .chain(self.db.aliases_of(mime))
            .collect()
    }

    pub fn effective(&self, mime: &str) -> Option<Effective> {
        mimeapps::effective(&self.layers, &self.keys(mime), |app| self.installed(app))
    }

    pub fn known_types(&self) -> BTreeSet<String> {
        let mut known = self.db.types.clone();
        known.extend(self.installed.mime_types().map(str::to_string));
        for layer in &self.layers {
            known.extend(layer.file.default_types());
        }
        known.retain(|mime| !mime.contains(WILDCARD));
        known
    }

    pub fn wanted(&self, config: &Config) -> BTreeMap<String, String> {
        expand(&config.defaults, &self.known_types(), |mime| {
            self.db.canonical(mime)
        })
    }

    pub fn current_defaults(&self) -> BTreeMap<String, String> {
        let mut types = BTreeSet::new();
        for layer in &self.layers {
            types.extend(
                layer
                    .file
                    .default_types()
                    .iter()
                    .map(|mime| self.db.canonical(mime)),
            );
        }
        types
            .into_iter()
            .filter_map(|mime| {
                let found = self.effective(&mime)?;
                Some((mime, self.installed.canonical(&found.app)))
            })
            .collect()
    }
}
