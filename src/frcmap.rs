use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result as AnyhowResult};
use log::{error, info};
use reqwest_middleware::ClientWithMiddleware;
use serde_json::{Value, json};

use crate::{
    first_api::FirstApiClient,
    geocoder::{FrcGeocoder, types::LocationDict},
    map_types::{EventData, TeamData},
    tba::TbaClient,
};

pub struct FrcMap {
    year: u32,
    tba: TbaClient,
    geocoder: FrcGeocoder,
    data: Option<Value>,
    debug_path: PathBuf,
}

impl FrcMap {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        client: Arc<ClientWithMiddleware>,
        tba_key: String,
        gmaps_key: String,
        first_token: &str,
        year: u32,
        archive: PathBuf,
        team_overrides: LocationDict,
        event_overrides: LocationDict,
        debug_path: PathBuf,
    ) -> Self {
        let tba = TbaClient::new(Arc::clone(&client), tba_key);
        let first_api = FirstApiClient::new(Arc::clone(&client), first_token);
        let geocoder = FrcGeocoder::new(
            Arc::clone(&client),
            gmaps_key,
            archive,
            team_overrides,
            event_overrides,
            first_api,
        );

        std::fs::create_dir_all(&debug_path).ok();

        Self {
            year,
            tba,
            geocoder,
            data: None,
            debug_path,
        }
    }

    /// Dump an intermediate value to the debug directory as pretty JSON.
    fn debug_dump(&self, name: &str, data: &impl serde::Serialize) {
        let path = self.debug_path.join(format!("{}.json", name));
        match serde_json::to_string_pretty(data) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    error!("Failed to write debug file {}: {}", path.display(), e);
                }
            }
            Err(e) => error!("Failed to serialise debug data for {}: {}", name, e),
        }
    }

    /// Main generation pipeline (mirrors Python `FRCMap.generate()`).
    pub async fn generate(&mut self) -> AnyhowResult<()> {
        // 1. Fetch all teams
        info!("Fetching all teams...");
        let raw_teams = self.tba.get_teams().await?;
        info!("Found {} teams.", raw_teams.len());
        let mut teams: HashMap<String, TeamData> = raw_teams
            .into_iter()
            .map(|(k, v)| (k, TeamData::new(v)))
            .collect();
        self.debug_dump("teams", &teams);

        // 2. Fetch all events for this year
        info!("Fetching all events in {}...", self.year);
        let raw_events = self.tba.get_events(self.year).await?;
        info!("Found {} events.", raw_events.len());
        let mut events: HashMap<String, EventData> = raw_events
            .into_iter()
            .map(|(k, v)| (k, EventData::new(v)))
            .collect();

        // 3. Fetch active teams
        info!("Fetching active teams in {}...", self.year);
        let active = self.tba.get_active_teams(self.year).await?;
        info!("Found {} active teams.", active.len());
        self.debug_dump("active_teams", &active);

        // 4. Geocode team locations
        self.geocoder
            .populate_team_locations(&mut teams, self.year)
            .await;
        self.debug_dump("teams_geocoded", &teams);

        // 5. Geocode event locations
        self.geocoder
            .populate_event_locations(&mut events, self.year)
            .await;
        self.debug_dump("events_geocoded", &events);

        // 6. Get team→events mapping
        let team_events = self.tba.get_team_events(self.year).await?;
        self.debug_dump("team_events", &team_events);

        // 7. Build active-team data with their events list
        let mut team_data: HashMap<String, TeamData> = HashMap::new();
        for tkey in &active {
            match teams.get(tkey) {
                Some(team_obj) => {
                    let mut obj = team_obj.clone();
                    obj.events = team_events.get(tkey).cloned().unwrap_or_default();
                    team_data.insert(tkey.clone(), obj);
                }
                None => {
                    error!("Failed to find key '{}' in team list.", tkey);
                }
            }
        }

        // 8. Add team lists to each event
        for (ekey, event) in events.iter_mut() {
            match self.tba.get_event_team_keys(ekey).await {
                Ok(team_keys) => {
                    event.teams = team_keys;
                }
                Err(e) => {
                    error!("Failed to fetch teams for event {}: {}", ekey, e);
                }
            }
        }

        self.data = Some(json!({
            "teams": team_data,
            "events": events,
        }));

        Ok(())
    }

    /// Write the output JSON files (pretty + minified).
    pub fn write(&self, output_dir: &Path) -> AnyhowResult<()> {
        let data = self
            .data
            .as_ref()
            .context("Data has not been generated yet!")?;

        std::fs::create_dir_all(output_dir)?;

        let pretty_path = output_dir.join(format!("season_{}_pretty.json", self.year));
        let compact_path = output_dir.join(format!("season_{}.json", self.year));

        let pretty = serde_json::to_string_pretty(data)?;
        std::fs::write(&pretty_path, &pretty)
            .with_context(|| format!("Failed to write {}", pretty_path.display()))?;
        info!("Wrote {}", pretty_path.display());

        let compact = serde_json::to_string(data)?;
        std::fs::write(&compact_path, &compact)
            .with_context(|| format!("Failed to write {}", compact_path.display()))?;
        info!("Wrote {}", compact_path.display());

        Ok(())
    }
}
