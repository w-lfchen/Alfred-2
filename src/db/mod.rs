//! module for database functions

use std::path::PathBuf;

use anyhow::{Context, Ok};
use poise::serenity_prelude::GuildId;
use sqlx::SqlitePool;
use tokio::fs::File;

use crate::config::Config;

/// path to the db, relative to the state directory
const DB_PATH: &str = "db.sqlite";

/// set up the database:
/// - create the file if needed
/// - run the table setup
pub async fn setup_db() -> Result<SqlitePool, anyhow::Error> {
    let mut db_path = PathBuf::from(Config::get().state_dir());
    db_path.push(DB_PATH);
    let db_path = db_path;
    // ensure db file exists
    if !db_path.exists() {
        tokio::fs::create_dir_all(
            db_path
                .parent()
                .context("unable to obtain db path parent")?,
        )
        .await
        .context("unable to create state path dir")?;
        File::create_new(&db_path)
            .await
            .context("unable to create db file")?;
    }

    // this currently is a small-scale applications, so we don't need to set options for now
    let pool = SqlitePool::connect(db_path.to_str().context("db path is not valid utf8")?).await?;
    // id is text because sqlx doesn't allow u64
    sqlx::query!("create table if not exists disabled_commands(guild_id text, command_name text)")
        .execute(&pool)
        .await?;
    Ok(pool)
}

/// checks whether a command is enabled in a guild
pub async fn is_enabled(pool: &SqlitePool, id: GuildId, name: &str) -> Result<bool, anyhow::Error> {
    let commands = sqlx::query!(
        "select command_name from disabled_commands where guild_id = ?",
        id.to_string()
    )
    .fetch_all(pool)
    .await?;
    Ok(!commands
        .iter()
        .any(|x| x.command_name.as_deref() == Some(name)))
}

/// returns whether command is enabled
pub async fn toggle(pool: &SqlitePool, id: GuildId, name: &str) -> Result<bool, anyhow::Error> {
    let id = id.to_string();

    let mut transaction = pool.begin().await?;
    let is_disabled = sqlx::query!(
        "select * from disabled_commands where guild_id = ? and command_name = ?",
        id,
        name
    )
    .fetch_optional(&mut *transaction)
    .await?
    .is_some();
    let query = if is_disabled {
        sqlx::query!(
            "delete from disabled_commands where guild_id = ? and command_name = ?",
            id,
            name,
        )
    } else {
        sqlx::query!("insert into disabled_commands values (?, ?)", id, name)
    };
    query.execute(&mut *transaction).await?;
    transaction.commit().await?;

    // return is_disabled as this should now be the inverse of whether the command is enabled
    Ok(is_disabled)
}
