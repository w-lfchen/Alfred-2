//! commands that perform administrative functions

use anyhow::{Context as _, bail};

use crate::{commands::Context, db};

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
        let enabled = db::toggle(ctx.data().pool(), guild_id, &name).await?;
        ctx.say(format!(
            "command `{name}` has been successfully {}abled.",
            if enabled { "en" } else { "dis" }
        ))
        .await?;
    } else {
        // no command with this name was found
        ctx.say(format!("unknown command name: {name}")).await?;
    }
    Ok(())
}
