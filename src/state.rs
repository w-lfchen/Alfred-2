use std::{
    collections::{HashMap, HashSet},
    io::{Error, ErrorKind},
    path::Path,
};

use poise::serenity_prelude::GuildId;
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct State {
    pub disabled_commands: HashMap<GuildId, HashSet<String>>,
}

impl State {
    /// saves the state to the given path.
    ///
    /// creates the parent directory of the given file if it does not exist.
    /// the path is expected to point to a valid file location and is not canonicalized.
    pub async fn save_to_disk(&self, path: &Path) -> Result<(), Error> {
        fs::create_dir_all(
            path.parent()
                .ok_or(Error::from(ErrorKind::InvalidFilename))?,
        )
        .await?;
        fs::write(
            path,
            serde_json::to_vec(self).map_err(|_| ErrorKind::InvalidData)?,
        )
        .await
    }
}
