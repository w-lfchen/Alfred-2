pub mod admin;
mod typst;

use crate::{errors::NoDolphinError, state::State};

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::Context as _;
use poise::{
    CreateReply,
    serenity_prelude::{
        Color, CreateAttachment, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter,
    },
};
use rand::seq::IteratorRandom;
use reqwest::Response;
use tokio::sync::Mutex;

type Context<'a> = poise::Context<'a, Mutex<State>, anyhow::Error>;

// TODO: maybe improve this as this path is relative to the current working directory
const DOLPHIN_PATH: &str = "./resources/dolphins.txt";

pub async fn command_check(ctx: Context<'_>) -> Result<bool, anyhow::Error> {
    let lock = ctx.data().lock().await;
    Ok(ctx
        .guild_id()
        .and_then(|guild_id| lock.disabled_commands.get(&guild_id))
        // if we are not in a guild or the guild wasn't found, the command is considered enabled
        .is_none_or(|disabled_commands| !disabled_commands.contains(&ctx.command().name)))
}

/// alfred cat
///
/// This command fetches you a random cat from <https://cataas.com/> :3
/// More parameters might be added in the future.
#[poise::command(
    slash_command,
    prefix_command,
    track_edits,
    broadcast_typing,
    aliases("car")
)]
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

/// Fetches an animal with the name from Inat
#[poise::command(
    slash_command,
    prefix_command,
    track_edits,
    broadcast_typing
)]
pub async fn send(
    ctx: Context<'_>,
    #[rest]
    query: Option<String>,
) -> Result<(), anyhow::Error> {

    // Im using this a random number seed
    let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();

    // just look for frogs if it finds nothing
    let animal = query.unwrap_or("frog".to_string());

    // make request
    let url = format!("https://api.inaturalist.org/v1/taxa/autocomplete?q={}&per_page=1", animal);
    let response = reqwest::get(url).await?;
    let parsed = json::parse(&response.text().await?)?;

    // default is frogs
    let mut id = 20979;
    let taxon = &parsed["results"][0];
    if !taxon.is_null(){
        id = taxon["id"].as_u64().unwrap_or(0);
    };

    // term id 17 and term value id 18 searches for only alive animals 
    let mut photo_url = format!(
         "https://api.inaturalist.org/v1/observations?taxon_id={}&quality_grade=research&order_by=random&per_page=1&seed={}&term_id=17&term_value_id=18",
         id, seed
     );

    //plants cant be dead 47126 are all plants
    if (!taxon["id"].is_null() && taxon["id"].as_u64() == Some(47126)) || (!taxon["ancestor_ids"].is_null() && taxon["ancestor_ids"]
            .members()
            .any(|v| v.as_u64() == Some(47126))) {
               photo_url = format!(
                "https://api.inaturalist.org/v1/observations?taxon_id={}&quality_grade=research&order_by=random&per_page=1&seed={}",
                id, seed
                );
            }

    let photo_response = reqwest::get(photo_url).await?;
    let photo_parsed = json::parse(&photo_response.text().await?)?;

    let mut photo_url = photo_parsed["results"][0]["observation_photos"][0]["photo"]["url"]
        .as_str()
        .map(|u| u.replace("square", "large"))
        .unwrap();

    // id 47118 are arachnids
    if (!taxon["id"].is_null() && taxon["id"].as_u64() == Some(47118)) || (!taxon["ancestor_ids"].is_null() && taxon["ancestor_ids"]
            .members()
            .any(|v| v.as_u64() == Some(47118))) {
                photo_url = format!("|| {} ||", photo_url)
            };
    
    let name_of_obs = &photo_parsed["results"][0]["taxon"]["preferred_common_name"].as_str().unwrap_or(photo_parsed["results"][0]["taxon"]["name"].as_str().unwrap());
    
    // post the image with the name
    ctx.say(format!("Image of {}", name_of_obs)).await?;

    ctx.say(photo_url).await?;

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
#[poise::command(slash_command, prefix_command, track_edits, broadcast_typing)]
pub async fn define(
    ctx: Context<'_>,
    #[rest]
    #[description = "what to define"]
    query: Option<String>,
) -> Result<(), anyhow::Error> {
    let query = query.unwrap_or_default();
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
            .title(format!("No definition found for \"{query}\""))
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

/// alfred dog
///
/// fetches a random dog from <https://dog.ceo/>
#[poise::command(slash_command, prefix_command, track_edits, broadcast_typing)]
pub async fn dog(ctx: Context<'_>) -> Result<(), anyhow::Error> {
    let response = reqwest::get("https://dog.ceo/api/breeds/image/random").await?;
    let parsed = json::parse(&response.text().await?)?;
    // structure as per https://dog.ceo/dog-api/documentation/random:
    // json object with field "message" that is the link to the dog
    match parsed["message"].as_str() {
        Some(image_url) => ctx.say(image_url).await?,
        None => ctx.say("api broke :(").await?,
    };
    Ok(())
}

/// alfred eminem
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn eminem(ctx: Context<'_>) -> Result<(), anyhow::Error> {
    ctx.say("https://media.tenor.com/-5B-JLo2q1cAAAAC/eminem-now-this-looks-like-a-job-for-me.gif")
        .await?;
    Ok(())
}

/// alfred fox
///
/// This command fetches a random fluffy foxy with a 1/1000 chance of foxy jumpscare.
#[poise::command(
    slash_command,
    prefix_command,
    track_edits,
    broadcast_typing,
    aliases("floof")
)]
pub async fn fox(ctx: Context<'_>) -> Result<(), anyhow::Error> {
    const FOX_API_ENDPOINT: &str = "https://randomfox.ca/floof/";

    if rand::random_range(1..=1000) == 67 {
        // scary foxy (ᗒᗣᗕ)՞
        ctx.say("https://tenor.com/fZEGeo3lNTk.gif").await?;
        return Ok(());
    }

    // fluffy foxy (˶>⩊<˶)
    let response = reqwest::get(FOX_API_ENDPOINT)
        .await
        .with_context(|| format!("Failed to fetch response from {FOX_API_ENDPOINT}"))?;
    let payload = &response.text().await?;
    let parsed = json::parse(payload)
        .with_context(|| format!("Failed to parse response payload:\n{payload}"))?;
    ctx.say(
        parsed["image"]
            .as_str()
            .unwrap_or("response has unexpected structure"),
    )
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

/// hard images
///
/// fetches a random hot post from <https://reddit.com/r/hardimages2>
#[poise::command(slash_command, prefix_command, track_edits, broadcast_typing)]
pub async fn tuff(ctx: Context<'_>) -> Result<(), anyhow::Error> {
    // we might want to reuse this client across invocations
    let client = reqwest::Client::builder()
        // reddit requires a user agent and will block us if we don't provide one
        .user_agent("alfred:2.0")
        .build()?;
    let response = client
        .get("https://www.reddit.com/r/hardimages2/hot.json?limit=100")
        .send()
        .await?;
    let code = response.status();
    if !code.is_success() {
        ctx.say(format!("unexpected response code: {code}\n-# note: if this is a 403, reddit has probably blocked us")).await?;
        return Ok(());
    }
    let parsed = json::parse(&response.text().await?)?;
    // parse the array of children
    let children = &parsed["data"]["children"];
    if !children.is_array() {
        ctx.say("failed to parse reddit response").await?;
    }
    let post = children
        .members()
        .choose(&mut rand::rng())
        .context("failed to choose post")?;
    // parse the post
    ctx.say(
        post["data"]["url"]
            .as_str()
            .unwrap_or("failed to parse post"),
    )
    .await?;
    Ok(())
}

/// render a typst document.
///
/// the document must not be empty and can either be plain text or a single- or multiline code block.
#[poise::command(slash_command, prefix_command, track_edits, broadcast_typing)]
pub async fn typst(
    ctx: Context<'_>,
    #[rest]
    #[description = "the document to render"]
    document: String,
) -> Result<(), anyhow::Error> {
    // don't block the current thread with a potentially long-running compilation
    let join = tokio::task::spawn_blocking(|| typst::render_png(trim_typst_doc(document)));
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

/// if the document text (excluding leading whitespace) is a discord code block (i.e. starts and ends with ` or ```),
/// this function strips the code block and leading/trailing whitespace and returns a new [String].
/// otherwise, the document is returned unchanged.
fn trim_typst_doc(document: String) -> String {
    let trimmed = document.trim();
    if let Some(s) = trimmed.strip_prefix("```")
        && let Some(trimmed) = s.strip_suffix("```")
    {
        String::from(trimmed)
    } else if let Some(s) = trimmed.strip_prefix("`")
        && let Some(trimmed) = s.strip_suffix("`")
    {
        String::from(trimmed)
    } else {
        document
    }
}
