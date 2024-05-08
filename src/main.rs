mod token;

use poise::{
    serenity_prelude::{self as serenity},
    Prefix, PrefixFrameworkOptions,
};
use token::TOKEN;

// definitions copied from example
struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// alfred define
#[poise::command(slash_command, prefix_command, track_edits)]
async fn define(
    ctx: Context<'_>,
    #[description = "Query"] query: Vec<String>,
) -> Result<(), Error> {
    let query = match ctx {
        // currently too lazy too properly parse this, this is the easy way out
        Context::Application(_) => query
            .iter()
            // begin with some capacity for optimization
            // this fold could probably be optimized
            .fold(String::with_capacity(query.len()), |accu, val| {
                accu + " " + val
            }),
        // just use the args in this case
        Context::Prefix(ctx) => String::from(ctx.args),
    };
    // get the json response into a json object
    let response = reqwest::get(format!(
        "https://api.urbandictionary.com/v0/define?term={query}"
    ))
    .await?;
    let parsed = json::parse(&response.text().await?)?;
    // response structure: object with single entry called "list" containing a list of objects, simply extract the first definition from there
    let definition = &parsed["list"][0]["definition"];
    // if null, no definition was found (or api changed i guess)
    let response = if definition.is_string() {
        definition.to_string()
    } else {
        // no string
        String::from("Error: Alfred 2 was unable to find a definition")
    };
    // send response
    ctx.reply(response).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    // get some constant parameters
    let token = TOKEN;
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    // create framework
    let framework = poise::Framework::builder()
        // set options
        .options(poise::FrameworkOptions {
            commands: vec![define()],
            // set up prefix
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(String::from("alfred")),
                additional_prefixes: vec![Prefix::Literal("Alfred")],
                mention_as_prefix: true,
                execute_untracked_edits: true,
                ignore_bots: false,
                case_insensitive_commands: true,
                ..Default::default()
            },
            ..Default::default()
        })
        // TODO: figure out what this does (copied from example)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();
    // get the client
    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    // start the thing
    client.unwrap().start().await.unwrap();
}
