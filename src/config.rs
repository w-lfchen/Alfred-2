//! static configuration that does not change at runtime
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug)]
pub struct Config {
    state_path: PathBuf,
}

impl Config {
    /// create a new config by parsing environment variables:
    /// - `STATE_PATH` for the path where state should be saved
    pub fn from_env() -> Result<Self, anyhow::Error> {
        Ok(Self {
            state_path: std::env::var("STATE_PATH")?.into(),
        })
    }

    /// get the static config.
    ///
    /// this function should usually be called after [CONFIG] has been initialized.
    pub fn get() -> &'static Self {
        CONFIG.get_or_init(Self::default)
    }

    pub fn state_path(&self) -> &Path {
        &self.state_path
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            state_path: "./state/state.json".into(),
        }
    }
}
