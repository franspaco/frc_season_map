pub mod event_type;
pub mod types;

use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result as AnyhowResult};
use log::{info, warn};
use regex::Regex;
use reqwest_middleware::ClientWithMiddleware;
use serde::de::DeserializeOwned;

use crate::tba::types::{TbaEvent, TbaTeam};

const TBA_API_BASE: &str = "https://www.thebluealliance.com/api/v3/";

pub struct TbaClient {
    client: Arc<ClientWithMiddleware>,
    api_key: String,
}

impl TbaClient {
    pub fn new(client: Arc<ClientWithMiddleware>, api_key: String) -> Self {
        Self { client, api_key }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> AnyhowResult<T> {
        let url = format!("{}{}", TBA_API_BASE, path);
        let resp = self
            .client
            .get(&url)
            .header("X-TBA-Auth-Key", &self.api_key)
            .send()
            .await
            .with_context(|| format!("TBA request failed: {}", url))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("TBA API error {} for {}: {}", status, url, body);
        }

        resp.json::<T>()
            .await
            .with_context(|| format!("Failed to parse TBA response from {}", url))
    }

    /// Returns true if an event key matches the standard pattern (e.g. `2025cafr`).
    pub fn is_regular_event_key(key: &str) -> bool {
        lazy_static_regex().is_match(key)
    }

    // ── Teams ──────────────────────────────────────────────────────

    /// Get all teams, paginated (500 per page).
    pub async fn get_teams(&self) -> AnyhowResult<HashMap<String, TbaTeam>> {
        let mut teams = HashMap::new();
        let mut page = 0u32;
        loop {
            let batch: Vec<TbaTeam> = self.get(&format!("teams/{}", page)).await?;
            if batch.is_empty() {
                break;
            }
            for team in batch {
                teams.insert(team.key.clone(), team);
            }
            page += 1;
        }
        info!("Fetched {} teams across {} pages", teams.len(), page);
        Ok(teams)
    }

    // ── Events ─────────────────────────────────────────────────────

    /// Get all events for a year, filtering to regular event keys.
    pub async fn get_all_events(&self, year: u32) -> AnyhowResult<HashMap<String, TbaEvent>> {
        let all: Vec<TbaEvent> = self.get(&format!("events/{}", year)).await?;
        Ok(all
            .into_iter()
            .map(|event| (event.key.clone(), event))
            .collect())
    }

    /// Get team keys for a single event.
    pub async fn get_event_team_keys(&self, event_key: &str) -> AnyhowResult<Vec<String>> {
        let keys: Vec<String> = self.get(&format!("event/{}/teams/keys", event_key)).await?;
        for k in &keys {
            if !k.starts_with("frc") {
                warn!("Got invalid team key '{}' in event '{}'", k, event_key);
            }
        }
        Ok(keys)
    }
}

fn lazy_static_regex() -> &'static Regex {
    use std::sync::OnceLock;
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^20\d\d[a-z]+$").unwrap())
}
