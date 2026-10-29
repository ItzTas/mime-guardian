use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;

use crate::config::{self, Config};
use crate::reconcile::{self, Plan};
use crate::system::System;
use crate::utils::Xdg;

pub struct Context {
    pub config_path: PathBuf,
    pub config: Config,
    pub system: System,
    pub wanted: BTreeMap<String, String>,
}

impl Context {
    pub fn load() -> Result<Self> {
        let xdg = Xdg::from_env()?;
        let config_path = config::path(&xdg);
        let config = config::load(&config_path)?;
        let system = System::load(xdg.with_desktops(&config.guard.desktops))?;
        let wanted = system.wanted(&config);

        Ok(Self {
            config_path,
            config,
            system,
            wanted,
        })
    }

    pub fn plan(&self) -> Plan {
        reconcile::plan(&self.system, &self.wanted, &self.config.guard)
    }
}
