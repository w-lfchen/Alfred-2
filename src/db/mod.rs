//! module for database functions

use std::path::PathBuf;

use anyhow::Context;
use sqlx::SqlitePool;
use tokio::fs::File;

use crate::config::Config;

/// path to the db, relative to the state directory
const DB_PATH: &str = "db.sqlite";

/// set up the database:
/// - create the file if needed
/// - run the table setup
pub async fn setup_db() -> Result<SqlitePool, anyhow::Error> {
    let mut db_path = PathBuf::from(Config::get().state_path());
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
    sqlx::query!(
        "create table if not exists disabled_commands(guild_id integer, command_name text)"
    )
    .execute(&pool)
    .await?;
    Ok(pool)
}
