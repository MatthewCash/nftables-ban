use anyhow::Result;
use rusqlite::Connection;
use std::env;
use tracing_subscriber::{EnvFilter, filter::LevelFilter};

use crate::{
    config::load_config,
    database::{create_table, get_all_addrs},
    nftables::{add_addrs_to_set, flush_set},
    webhook::start_listening,
};

mod config;
mod database;
mod nftables;
mod webhook;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var("LOG_LEVEL")
                .from_env()?,
        )
        .init();

    let config =
        load_config(env::var("CONFIG_PATH").expect("Missing CONFIG_PATH env var!")).await?;

    let conn = Connection::open(&config.database.file_path)?;

    create_table(&conn, &config.database)?;

    let existing_addrs = get_all_addrs(&conn, &config.database)?;

    flush_set(&config.nftables)?;
    add_addrs_to_set(&config.nftables, &existing_addrs)?;

    log::debug!("set flushed and initialized with {existing_addrs:?}");

    start_listening(conn, config).await?;

    Ok(())
}
