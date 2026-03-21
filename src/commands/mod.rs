mod typst;

use crate::errors::NoDolphinError;

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use poise::{
    CreateReply,
    serenity_prelude::{
        Color, CreateAttachment, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter,
    },
};
use rand::seq::IteratorRandom;
use reqwest::Response;

type Context<'a> = poise::Context<'a, (), anyhow::Error>;

// TODO: maybe improve this as this path is relative to the current working directory
const DOLPHIN_PATH: &str = "./resources/dolphins.txt";

/// alfred cat
///
/// This command fetches you a random cat from https://cataas.com/ :3
/// More parameters might be added in the future.
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn cat(ctx: Context<'_>) -> Result<(), anyhow::Error> {
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
pub async fn define(
    ctx: Context<'_>,
    #[description = "Query"] query: Vec<String>,
) -> Result<(), anyhow::Error> {
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

/// alfred delfin
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn delfin(ctx: Context<'_>) -> Result<(), anyhow::Error> {
    let f = File::open(DOLPHIN_PATH)?;
    let f = BufReader::new(f);
    let dolphin = f.lines().choose(&mut rand::rng()).ok_or(NoDolphinError)??;
    ctx.say(dolphin).await?;
    Ok(())
}

/// alfred eminem
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn eminem(ctx: Context<'_>) -> Result<(), anyhow::Error> {
    ctx.say("https://media.tenor.com/-5B-JLo2q1cAAAAC/eminem-now-this-looks-like-a-job-for-me.gif")
        .await?;
    Ok(())
}

/// alfred kleanthis
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn kleanthis(ctx: Context<'_>) -> Result<(), anyhow::Error> {
    ctx.say("https://discordemoji.com/assets/emoji/KannaSip.png")
        .await?;
    Ok(())
}

/// render a typst document
#[poise::command(slash_command, prefix_command, track_edits, broadcast_typing)]
pub async fn typst(ctx: Context<'_>, #[rest] document: String) -> Result<(), anyhow::Error> {
    // don't block the current thread with a potentially long-running compilation
    let join = tokio::task::spawn(async { typst::render_png(document) });
    let mut reply = CreateReply::default();
    let (doc, diagnostics) = join.await??;
    if !diagnostics.is_empty() {
        let embed = CreateEmbed::default().description(diagnostics);
        reply = reply.embed(if doc.is_some() {
            embed
                .color(Color::from_rgb(249, 226, 175))
                .title("Warnings")
        } else {
            embed
                .color(Color::from_rgb(243, 139, 168))
                .title("Compilation failed")
        });
    }
    if let Some(png) = doc {
        reply = reply.attachment(CreateAttachment::bytes(png, "rendered.png"));
    }
    ctx.send(reply).await?;
    Ok(())
}
