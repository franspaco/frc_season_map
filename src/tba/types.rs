use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::tba::event_type::EventType;

/// TBA Team object — only fields we actually use.
/// `#[serde(default)]` on every optional field + `flatten` for extras means
/// the struct will never fail to deserialize even if TBA adds new fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TbaTeam {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_number: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub school_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_prov: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gmaps_place_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gmaps_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lng: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rookie_year: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motto: Option<String>,

    /// Catch-all for any extra fields TBA may add in the future.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Webcast sub-object inside Event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Webcast {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub webcast_type: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// TBA Event object — fields used by the map frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TbaEvent {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_event_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_type: Option<EventType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_type_string: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_prov: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub district: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub week: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gmaps_place_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gmaps_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lng: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webcasts: Option<Vec<Webcast>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub division_keys: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_event_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playoff_type: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playoff_type_string: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remap_teams: Option<Value>,

    /// Catch-all for any extra fields TBA may add in the future.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
