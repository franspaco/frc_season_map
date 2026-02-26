pub mod event_type;
pub mod types;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

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
    fn is_regular_event_key(key: &str) -> bool {
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
    pub async fn get_events(&self, year: u32) -> AnyhowResult<HashMap<String, TbaEvent>> {
        let all: Vec<TbaEvent> = self.get(&format!("events/{}", year)).await?;
        let events: HashMap<String, TbaEvent> = all
            .into_iter()
            .filter(|e| Self::is_regular_event_key(&e.key))
            .map(|e| (e.key.clone(), e))
            .collect();
        Ok(events)
    }

    /// Get event keys for a year (regular only).
    pub async fn get_event_keys(&self, year: u32) -> AnyhowResult<Vec<String>> {
        let all: Vec<String> = self.get(&format!("events/{}/keys", year)).await?;
        Ok(all
            .into_iter()
            .filter(|k| Self::is_regular_event_key(k))
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

    // ── Derived helpers (mirrors Python logic) ────────────────────

    /// Get the set of active team keys (teams that competed in at least one event).
    pub async fn get_active_teams(&self, year: u32) -> AnyhowResult<Vec<String>> {
        let events = self.get_event_keys(year).await?;
        let mut teams = HashSet::new();
        for event in &events {
            match self.get_event_team_keys(event).await {
                Ok(keys) => {
                    teams.extend(keys);
                }
                Err(e) => {
                    warn!("Failed to fetch teams for event {}: {}", event, e);
                }
            }
        }
        let mut sorted: Vec<String> = teams.into_iter().collect();
        sorted.sort();
        Ok(sorted)
    }

    /// Get a map of team_key → list of event_keys they attend.
    pub async fn get_team_events(&self, year: u32) -> AnyhowResult<HashMap<String, Vec<String>>> {
        let events = self.get_event_keys(year).await?;
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for event in &events {
            match self.get_event_team_keys(event).await {
                Ok(team_keys) => {
                    for tk in team_keys {
                        map.entry(tk).or_default().push(event.clone());
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch teams for event {}: {}", event, e);
                }
            }
        }
        Ok(map)
    }
}

fn lazy_static_regex() -> &'static Regex {
    use std::sync::OnceLock;
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^20\d\d[a-z]+$").unwrap())
}
