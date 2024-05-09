mod token;

use poise::{
    serenity_prelude::{
        self as serenity, Color, CreateAttachment, CreateEmbed, CreateEmbedAuthor,
        CreateEmbedFooter,
    },
    CreateReply, Prefix, PrefixFrameworkOptions,
};
use reqwest::Response;
use token::TOKEN;

// definitions copied from example
struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// alfred cat
///
/// This command fetches you a random cat from https://cataas.com/ :3
/// More parameters might be added in the future.
#[poise::command(slash_command, prefix_command, track_edits)]
async fn cat(ctx: Context<'_>) -> Result<(), Error> {
    // fetch a cat :3
    let response = reqwest::get("https://cataas.com/cat").await?;
    // determine filename, default to "cat.jpg"
    // hopefully won't cause issues
    let filename = match get_file_extension(&response) {
        Some(extension) => format!("cat.{extension}"),
        None => String::from("cat.jpg"),
    };
    // upload the file to discord's cdn
    ctx.send(
        CreateReply::default()
            .attachment(CreateAttachment::bytes(response.bytes().await?, filename)),
    )
    .await?;
    Ok(())
}

/// Attempt to get the file extension from a given reponse.
/// Won't work for every file type, use with caution.
///
/// Example: Passing a response with the Content-Type header being set to "image/jpeg" will return "jpeg".
fn get_file_extension(response: &Response) -> Option<&str> {
    response
        .headers()
        .get("Content-Type")?
        .to_str()
        .ok()?
        .split('/')
        .nth(1)
}

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
    // create embed with some data that is the same no matter the response
    let author = ctx.author();
    let embed = CreateEmbed::new()
        .author(CreateEmbedAuthor::new("Urban Dictionary"))
        .timestamp(ctx.created_at())
        .footer({
            let footer = CreateEmbedFooter::new(format!("Requested by {}", author.name));
            match author.avatar_url() {
                Some(url) => footer.icon_url(url),
                None => footer,
            }
        });
    // if json null, no definition was found (or api changed i guess)
    let embed = if definition.is_string() {
        // add the query results
        embed
            .color(Color::from_rgb(137, 180, 250))
            .title(query)
            .description(definition.to_string())
    } else {
        // no string -> error message
        embed
            .color(Color::from_rgb(243, 139, 168))
            .title(":x: No Definition found")
    };
    // send response with now fully built embed
    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// alfred eminem
#[poise::command(slash_command, prefix_command, track_edits)]
async fn eminem(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("https://media.tenor.com/-5B-JLo2q1cAAAAC/eminem-now-this-looks-like-a-job-for-me.gif")
        .await?;
    Ok(())
}

/// alfred kleanthis
#[poise::command(slash_command, prefix_command, track_edits)]
async fn kleanthis(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("https://discordemoji.com/assets/emoji/KannaSip.png")
        .await?;
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
            commands: vec![cat(), define(), eminem(), kleanthis()],
            // set up prefix
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(String::from("alfred")),
                additional_prefixes: vec![Prefix::Regex(
                    regex::Regex::new(r"(?i)\balfred\b")
                        .expect("error(prefixes): regex compilation failed for some reason"),
                )],
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
