mod commands;
mod errors;

use poise::{
    Prefix, PrefixFrameworkOptions,
    serenity_prelude::{ClientBuilder, GatewayIntents},
};

use crate::commands::{cat, define, delfin, eminem, kleanthis, typst};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv()?;
    // get some constant parameters
    let token = std::env::var("BOT_TOKEN")?;
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // create framework
    let framework = poise::Framework::builder()
        // set options
        .options(poise::FrameworkOptions {
            commands: vec![cat(), define(), delfin(), eminem(), kleanthis(), typst()],
            // set up prefix
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(String::from("alfred")),
                additional_prefixes: vec![Prefix::Regex(regex::Regex::new(r"(?i)\balfred\b")?)],
                mention_as_prefix: true,
                execute_untracked_edits: true,
                ignore_bots: false,
                case_insensitive_commands: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(())
            })
        })
        .build();
    ClientBuilder::new(token, intents)
        .framework(framework)
        .await?
        .start()
        .await?;
    Ok(())
}
