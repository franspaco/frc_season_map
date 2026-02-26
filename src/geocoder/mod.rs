pub mod types;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result as AnyhowResult;
use log::{error, info, warn};
use rand::Rng;
use regex::Regex;
use reqwest_middleware::ClientWithMiddleware;
use serde_json::Value;

use crate::{
    first_api::FirstApiClient,
    geocoder::types::GeocodeLocation,
    geocoder::types::{GeocodeResponse, LocationDict, LocationOverride},
    map_types::{EventData, HasLocation, TeamData},
    tba::types::{TbaEvent, TbaTeam},
};

// ── Address builders ──────────────────────────────────────────

fn make_team_address(team: &TbaTeam) -> Option<String> {
    let parts = [
        team.school_name.as_deref().unwrap_or(""),
        team.city.as_deref().unwrap_or(""),
        team.state_prov.as_deref().unwrap_or(""),
        team.postal_code.as_deref().unwrap_or(""),
        team.country.as_deref().unwrap_or(""),
    ];
    let addr: String = parts
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if addr.is_empty() { None } else { Some(addr) }
}

fn make_event_address(event: &TbaEvent) -> Option<String> {
    let parts = [
        event.venue.as_deref().unwrap_or(""),
        event.address.as_deref().unwrap_or(""),
        event.city.as_deref().unwrap_or(""),
        event.state_prov.as_deref().unwrap_or(""),
        event.postal_code.as_deref().unwrap_or(""),
        event.country.as_deref().unwrap_or(""),
    ];
    let addr = parts.join(" ").trim().to_string();
    if addr.is_empty() { None } else { Some(addr) }
}

// ── Geocoder ───────────────────────────────────────────────────

pub struct FrcGeocoder {
    client: Arc<ClientWithMiddleware>,
    gmaps_key: String,
    archive_path: PathBuf,
    team_overrides: LocationDict,
    event_overrides: LocationDict,
    team_archive: HashMap<String, LocationOverride>,
    event_archive: HashMap<String, LocationOverride>,
    pub first_api: FirstApiClient,
}

impl FrcGeocoder {
    pub fn new(
        client: Arc<ClientWithMiddleware>,
        gmaps_key: String,
        archive_path: PathBuf,
        team_overrides: LocationDict,
        event_overrides: LocationDict,
        first_api: FirstApiClient,
    ) -> Self {
        let team_archive = Self::read_team_archive(&archive_path);
        let event_archive = Self::read_event_archive(&archive_path);
        Self {
            client,
            gmaps_key,
            archive_path,
            team_overrides,
            event_overrides,
            team_archive,
            event_archive,
            first_api,
        }
    }

    // ── Archive I/O ────────────────────────────────────────────

    fn read_team_archive(archive_path: &Path) -> HashMap<String, LocationOverride> {
        let re = Regex::new(r"^all_team_locations_(\d{4})\.json$").unwrap();
        let mut years: HashMap<i64, PathBuf> = HashMap::new();

        if let Ok(entries) = std::fs::read_dir(archive_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some(caps) = re.captures(&name)
                    && let Ok(y) = caps[1].parse::<i64>()
                {
                    years.insert(y, entry.path());
                }
            }
        }

        if let Some((max_year, path)) = years.iter().max_by_key(|(y, _)| *y) {
            info!(
                "Using team location archive: {} (year {})",
                path.display(),
                *max_year
            );
            match std::fs::read_to_string(path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(map) => return map,
                    Err(e) => warn!("Failed to parse team archive: {}", e),
                },
                Err(e) => warn!("Failed to read team archive: {}", e),
            }
        } else {
            warn!(
                "Team archive not available, geocoding will take a while and incur several API requests."
            );
        }
        HashMap::new()
    }

    fn read_event_archive(archive_path: &Path) -> HashMap<String, LocationOverride> {
        let path = archive_path.join("all_event_locations.json");
        if path.is_file() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(map) => return map,
                    Err(e) => warn!("Failed to parse event archive: {}", e),
                },
                Err(e) => warn!("Failed to read event archive: {}", e),
            }
        } else {
            warn!(
                "Event archive not available, geocoding will take a while and incur several API requests."
            );
        }
        HashMap::new()
    }

    fn save_team_archive(&self, teams: &HashMap<String, TeamData>, year: u32) {
        let data: HashMap<String, Value> = teams
            .iter()
            .filter(|(_, v)| v.has_location() && v.ignore != Some(true))
            .map(|(k, v)| {
                let entry = serde_json::json!({
                    "lat": v.tba.lat,
                    "lng": v.tba.lng,
                });
                (k.clone(), entry)
            })
            .collect();

        let name = format!("all_team_locations_{}.json", year);
        let path = self.archive_path.join(name);
        std::fs::create_dir_all(&self.archive_path).ok();
        if let Err(e) = std::fs::write(&path, serde_json::to_string(&data).unwrap_or_default()) {
            error!("Failed to save team archive: {}", e);
        }
    }

    fn save_event_archive(&self, events: &HashMap<String, EventData>) {
        let data: HashMap<String, Value> = events
            .iter()
            .filter(|(_, v)| v.has_location() && v.ignore != Some(true))
            .map(|(k, v)| {
                let entry = serde_json::json!({
                    "lat": v.tba.lat,
                    "lng": v.tba.lng,
                });
                (k.clone(), entry)
            })
            .collect();

        let path = self.archive_path.join("all_event_locations.json");
        std::fs::create_dir_all(&self.archive_path).ok();
        if let Err(e) = std::fs::write(&path, serde_json::to_string(&data).unwrap_or_default()) {
            error!("Failed to save event archive: {}", e);
        }
    }

    // ── Google Maps Geocoding ──────────────────────────────────

    async fn geocode_address(&self, address: &str) -> Option<GeocodeLocation> {
        let url = format!(
            "https://maps.googleapis.com/maps/api/geocode/json?address={}&key={}",
            urlencoding::encode(address),
            &self.gmaps_key,
        );

        match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<GeocodeResponse>().await {
                Ok(parsed) => {
                    let Some(loc) = parsed
                        .results
                        .into_iter()
                        .next()
                        .and_then(|r| r.geometry)
                        .and_then(|g| g.location)
                    else {
                        warn!("Geocode response for '{}' has no results!", address);
                        return None;
                    };

                    Some(loc)
                }
                Err(e) => {
                    error!("Failed to parse geocode response: {}", e);
                    None
                }
            },
            Err(e) => {
                error!("Geocode HTTP request failed: {}", e);
                None
            }
        }
    }

    async fn geolocate_team(&self, team: &mut TeamData) {
        let addr = make_team_address(&team.tba);
        let key = team.tba.key.clone();
        match addr {
            None => {
                error!("Team {} has no address.", key);
                team.clear_location();
            }
            Some(addr) => {
                info!("Address for {}: {}", key, addr);
                match self.geocode_address(&addr).await {
                    Some(loc) => {
                        team.set_lat_lng(loc.lat, loc.lng);
                        info!("Location: ({}, {})", loc.lat, loc.lng);
                    }
                    None => {
                        error!("Could not geocode address for team {}", key);
                        team.clear_location();
                    }
                }
            }
        }
    }

    async fn geolocate_event(&self, event: &mut EventData) {
        let addr = make_event_address(&event.tba);
        let key = event.tba.key.clone();
        match addr {
            None => {
                error!("Event {} has no address.", key);
                event.clear_location();
            }
            Some(addr) => {
                info!("Address for {}: {}", key, addr);
                match self.geocode_address(&addr).await {
                    Some(loc) => {
                        event.set_lat_lng(loc.lat, loc.lng);
                        info!("Location: ({}, {})", loc.lat, loc.lng);
                    }
                    None => {
                        error!("Could not geocode address for event {}", key);
                        event.clear_location();
                    }
                }
            }
        }
    }

    // ── Location deduplication ─────────────────────────────────

    fn dedup_locations<T: HasLocation>(objects: &mut HashMap<String, T>, obj_type: &str) {
        let mut seen: HashMap<(u64, u64), String> = HashMap::new();
        let mut to_jitter: Vec<String> = Vec::new();

        // First pass: find collisions.
        for (key, obj) in objects.iter() {
            if !obj.has_location() {
                continue;
            }
            let lat = obj.lat().unwrap();
            let lng = obj.lng().unwrap();
            let loc_key = (lat.to_bits(), lng.to_bits());
            if let Some(existing) = seen.get(&loc_key) {
                warn!(
                    "{} {} location overlaps with {}, randomizing a bit!",
                    obj_type, key, existing,
                );
                to_jitter.push(key.clone());
            } else {
                seen.insert(loc_key, key.clone());
            }
        }

        // Second pass: apply jitter.
        let mut rng = rand::rng();
        for key in to_jitter {
            if let Some(obj) = objects.get_mut(&key) {
                jitter_location(obj, &mut rng);
            }
        }
    }

    // ── Public API ─────────────────────────────────────────────

    pub async fn populate_team_locations(&self, teams: &mut HashMap<String, TeamData>, year: u32) {
        info!("Geolocating teams.");

        let keys: Vec<String> = teams.keys().cloned().collect();
        for key in &keys {
            let team = teams.get_mut(key).unwrap();

            // Priority 1: manual override
            if let Some(ov) = self.team_overrides.get(key.as_str()) {
                apply_override(team, ov);
            }
            // Priority 2: archive
            else if let Some(ov) = self.team_archive.get(key.as_str()) {
                apply_override(team, ov);
            }
            // Priority 3: geocode
            else {
                warn!("Geocoding team {}", key);
                self.geolocate_team(team).await;
            }
        }

        Self::dedup_locations(teams, "Team");
        self.save_team_archive(teams, year);
        info!("Geolocating teams finished.");
    }

    pub async fn populate_event_locations(
        &self,
        events: &mut HashMap<String, EventData>,
        year: u32,
    ) {
        info!("Geolocating events.");

        let keys: Vec<String> = events.keys().cloned().collect();
        for key in &keys {
            let event = events.get_mut(key).unwrap();

            // Priority 1: manual override
            if let Some(ov) = self.event_overrides.get(key.as_str()) {
                apply_override(event, ov);
            }

            if !event.has_location() {
                // Try archive
                if let Some(ov) = self.event_archive.get(key.as_str()) {
                    apply_override(event, ov);
                }
                // Otherwise, if official, enhance + geocode
                else if event.is_official {
                    warn!("Geocoding event {}", key);
                    // Try to enhance with FIRST API data
                    let first_event_code = event.tba.first_event_code.clone();
                    if let Some(ref code) = first_event_code {
                        let mut venue = event.tba.venue.clone();
                        let mut address = event.tba.address.clone();
                        if let Err(e) = self
                            .first_api
                            .enhance_event_data(year as i64, code, &mut venue, &mut address)
                            .await
                        {
                            error!("Failed to fetch FIRST data for event {}: {}", key, e);
                        }
                        event.tba.venue = venue;
                        event.tba.address = address;
                    }
                    self.geolocate_event(event).await;
                } else {
                    error!("Event {} is not official and could not be geocoded!", key);
                }
            }

            // If event still has no location, mark it as ignored
            if !event.has_location() {
                event.ignore = Some(true);
                error!("Event {} has no location!", key);
            }
        }

        self.save_event_archive(events);
        Self::dedup_locations(events, "Event");
        info!("Geolocating events finished.");
    }
}

// ── Free helpers ───────────────────────────────────────────────

fn apply_override<T: HasLocation>(obj: &mut T, ov: &LocationOverride) {
    if let (Some(lat), Some(lng)) = (ov.lat, ov.lng) {
        obj.set_lat_lng(lat, lng);
    }
    if let Some(ignore) = ov.ignore {
        obj.set_ignore(ignore);
    }
}

fn jitter_location<T: HasLocation>(obj: &mut T, rng: &mut impl Rng) {
    use rand_distr::{Distribution, Normal};
    let normal = Normal::new(0.0, 0.001).unwrap();
    if let (Some(lat), Some(lng)) = (obj.lat(), obj.lng()) {
        let dlat: f64 = normal.sample(rng);
        let dlng: f64 = normal.sample(rng);
        obj.set_lat_lng(lat + dlat, lng + dlng);
    }
}

/// Load a location override file (JSON), stripping the `_comment` key.
pub fn load_location_file(path: &Path) -> AnyhowResult<LocationDict> {
    let content = fs::read_to_string(path)?;
    let mut map: HashMap<String, Value> = serde_json::from_str(&content)?;
    map.remove("_comment");
    let mut result = LocationDict::new();
    for (k, v) in map {
        let entry: LocationOverride = serde_json::from_value(v)?;
        result.insert(k, entry);
    }
    Ok(result)
}
