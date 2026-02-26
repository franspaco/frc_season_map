use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Response wrapper from the FIRST API `events` endpoint.
/// The actual events are inside a top-level `Events` array.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct FirstEventsResponse {
    #[serde(default)]
    pub events: Option<Vec<FirstEvent>>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// A single event from the FIRST API.
/// We only care about `venue` and `address` for geocoding enhancement.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FirstEvent {
    #[serde(default)]
    pub venue: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub state_prov: Option<String>,
    #[serde(default)]
    pub country: Option<String>,

    /// Catch-all for any extra fields.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
