mod config;
mod first_api;
mod frcmap;
mod geocoder;
mod http_client;
mod map_types;
mod tba;

use std::path::Path;

use anyhow::Result;
use clap::Parser;
use env_logger::{Builder, Env};
use log::info;

use config::{ApiKeys, Cli};
use frcmap::FrcMap;

#[tokio::main]
async fn main() -> Result<()> {
    Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    let cli = Cli::parse();
    info!("Starting FRC Season Map Generator (year = {})", cli.year);

    // Load API keys
    let keys = ApiKeys::load(&cli.api_keys)?;

    // Ensure directories exist
    ensure_dir(&cli.cache, "cache")?;
    ensure_dir(&cli.archive, "archive")?;
    ensure_dir(&cli.output, "output")?;

    // Load manual location overrides
    let team_overrides = if cli.teams.exists() {
        info!("Loading team locations from: {}", cli.teams.display());
        geocoder::load_location_file(&cli.teams)?
    } else {
        Default::default()
    };

    let event_overrides = if cli.events.exists() {
        info!("Loading event locations from: {}", cli.events.display());
        geocoder::load_location_file(&cli.events)?
    } else {
        Default::default()
    };

    // Build shared HTTP client with persistent cache
    let client = http_client::build_cached_client(&cli.cache)?;

    // Create main object
    let mut map = FrcMap::new(
        client,
        keys.tba_key,
        keys.gmaps_key,
        &keys.first_token,
        cli.year,
        cli.archive,
        team_overrides,
        event_overrides,
        cli.debug_path,
    );

    map.generate().await?;
    map.write(&cli.output)?;

    info!("Done!");
    Ok(())
}

fn ensure_dir(path: &Path, label: &str) -> Result<()> {
    if path.exists() {
        anyhow::ensure!(
            path.is_dir(),
            "{} path is not a directory: {}",
            label,
            path.display()
        );
    } else {
        info!("Creating {}: {}", label, path.display());
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}
