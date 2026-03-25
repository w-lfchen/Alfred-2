use std::collections::{HashMap, HashSet};

use poise::serenity_prelude::GuildId;

#[derive(Debug, Default)]
pub struct State {
    pub disabled_commands: HashMap<GuildId, HashSet<String>>,
}
