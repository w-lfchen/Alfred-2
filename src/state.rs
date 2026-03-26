use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, Error, ErrorKind},
    path::Path,
};

use poise::serenity_prelude::GuildId;
use serde::{Deserialize, Serialize};

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
        tokio::fs::create_dir_all(
            path.parent()
                .ok_or(Error::from(ErrorKind::InvalidFilename))?,
        )
        .await?;
        tokio::fs::write(
            path,
            serde_json::to_vec(self).map_err(|_| ErrorKind::InvalidData)?,
        )
        .await
    }

    /// attempt to deserialize a state from the given path
    pub fn read_from_disk(path: &Path) -> Result<Self, anyhow::Error> {
        let reader = BufReader::new(File::open(path)?);
        serde_json::from_reader(reader).map_err(From::from)
    }
}
