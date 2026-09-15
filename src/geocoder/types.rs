use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A location override entry — e.g. from `locations/teams.json`.
/// May contain `lat`/`lng`, or just `"ignore": true`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationOverride {
    #[serde(default)]
    pub lat: Option<f64>,
    #[serde(default)]
    pub lng: Option<f64>,
    #[serde(default)]
    pub ignore: Option<bool>,

    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Type alias matching the Python `LocationDict = Dict[str, Dict[str, float]]`.
pub type LocationDict = HashMap<String, LocationOverride>;

/// The final source used to resolve an object's location during a generation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationSource {
    TbaProvided,
    ManualOverride,
    Archive,
    GoogleGeocode,
    GeocodeFailed,
    NonOfficialEvent,
}

/// Report-only metadata explaining how an object's final location was chosen.
#[derive(Debug, Clone, Serialize)]
pub struct LocationResolution {
    pub source: LocationSource,
    pub manual_override_applied: bool,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub ignored: bool,
    pub randomized_to_avoid_collision: bool,
}

// ── Google Maps Geocoding API response types ───────────────────

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct GeocodeResponse {
    #[serde(default)]
    pub results: Vec<GeocodeResult>,
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct GeocodeResult {
    #[serde(default)]
    pub geometry: Option<GeocodeGeometry>,
}

#[derive(Debug, Deserialize)]
pub struct GeocodeGeometry {
    #[serde(default)]
    pub location: Option<GeocodeLocation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeocodeLocation {
    pub lat: f64,
    pub lng: f64,
}
