use std::path::PathBuf;

use chrono::Datelike;
use clap::Parser;
use serde::Deserialize;

/// FRC Season Map Generator - Rust port
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// FRC Season year
    #[arg(short, long, default_value_t = chrono::Utc::now().year() as u32)]
    pub year: u32,

    /// Path to JSON file with team manual location overrides
    #[arg(
        short = 't',
        long = "team-locations",
        default_value = "locations/teams.json"
    )]
    pub teams: PathBuf,

    /// Path to JSON file with event manual location overrides
    #[arg(
        short = 'e',
        long = "event-locations",
        default_value = "locations/events.json"
    )]
    pub events: PathBuf,

    /// Path to location archive directory
    #[arg(
        short = 'l',
        long = "location-archive",
        default_value = "locations/archive"
    )]
    pub archive: PathBuf,

    /// HTTP cache directory location
    #[arg(short = 'c', long = "cache-location", default_value = "cache")]
    pub cache: PathBuf,

    /// Directory to write JSON output to
    #[arg(short = 'o', long = "output-location", default_value = "docs/data")]
    pub output: PathBuf,

    /// Directory to write debug output to
    #[arg(short = 'd', long = "debug-path", default_value = "debug")]
    pub debug_path: PathBuf,

    /// Path to TOML file containing API keys
    #[arg(short = 'k', long = "api-keys", default_value = "api-keys.toml")]
    pub api_keys: PathBuf,
}

/// API keys loaded from TOML config file
#[derive(Debug, Deserialize)]
pub struct ApiKeys {
    pub tba_key: String,
    pub gmaps_key: String,
    pub first_token: String,
}

impl ApiKeys {
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            anyhow::anyhow!("Failed to read API keys from {}: {}", path.display(), e)
        })?;
        let keys: ApiKeys = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse API keys TOML: {}", e))?;
        Ok(keys)
    }
}
