//! static configuration that does not change at runtime
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::Context;

pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug)]
pub struct Config {
    dolphin_path: PathBuf,
    state_dir: PathBuf,
}

impl Config {
    /// create a new config by parsing environment variables:
    /// - `DOLPHIN_PATH` for the path to the dolphin file
    /// - `STATE_DIR` for the directory where state should be saved
    pub fn from_env() -> Result<Self, anyhow::Error> {
        Ok(Self {
            dolphin_path: std::env::var("DOLPHIN_PATH")
                .context("failed to read DOLPHIN_PATH environment variable")?
                .into(),
            state_dir: std::env::var("STATE_DIR")
                .context("failed to read STATE_DIR environment variable")?
                .into(),
        })
    }

    /// get the static config.
    ///
    /// this function should usually be called after [CONFIG] has been initialized.
    pub fn get() -> &'static Self {
        CONFIG.get_or_init(Self::default)
    }

    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    pub fn dolphin_path(&self) -> &Path {
        &self.dolphin_path
    }
}

// TODO: better defaults, allow partial default fallbacks
impl Default for Config {
    fn default() -> Self {
        Self {
            dolphin_path: "./resources/dolphins.txt".into(),
            state_dir: "./state/".into(),
        }
    }
}
