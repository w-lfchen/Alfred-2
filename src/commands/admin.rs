//! commands that perform administrative functions

use anyhow::{Context as _, bail};

use crate::commands::Context;

#[poise::command(slash_command, broadcast_typing)]
pub async fn toggle_command(ctx: Context<'_>, name: String) -> Result<(), anyhow::Error> {
    let guild_id = ctx
        .guild_id()
        .context("toggle may only be used in guilds")?;
    if name == "toggle_command" {
        bail!("toggling this command is not allowed");
    }
    if ctx
        .framework()
        .options
        .commands
        .iter()
        .find(|command| command.name == name)
        .is_some()
    {
        let mut lock = ctx.data().lock().await;
        let disabled_commands = lock.disabled_commands.entry(guild_id).or_default();
        if disabled_commands.contains(&name) {
            disabled_commands.remove(&name);
            ctx.say(format!(
                "command successfully enabled.\ncurrently disabled commands: {disabled_commands:?}"
            ))
            .await?;
        } else {
            disabled_commands.insert(name);
            ctx.say(format!(
                "command successfully disabled.\ncurrently disabled commands: {disabled_commands:?}",
            ))
            .await?;
        }
    } else {
        // no command with this name was found
        ctx.say(format!("unknown command name: {name}")).await?;
    }
    Ok(())
}
