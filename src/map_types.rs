use serde::{Deserialize, Serialize};

use crate::tba::types::{TbaEvent, TbaTeam};

/// A team enriched with a geocoded location and its list of events for the season.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamData {
    #[serde(flatten)]
    pub tba: TbaTeam,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<String>,
}

impl TeamData {
    pub fn new(tba: TbaTeam) -> Self {
        Self {
            tba,
            ignore: None,
            events: Vec::new(),
        }
    }
}

/// An event enriched with derived flags, a geocoded location and its roster of teams.
#[derive(Debug, Clone, Serialize)]
pub struct EventData {
    #[serde(flatten)]
    pub tba: TbaEvent,
    pub is_cmp: bool,
    pub is_official: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub teams: Vec<String>,
}

impl EventData {
    pub fn new(tba: TbaEvent) -> Self {
        let is_cmp = tba.event_type.map(|t| t.is_championship()).unwrap_or(false);
        let is_official = tba.event_type.map(|t| t.is_official()).unwrap_or(false);
        Self {
            tba,
            is_cmp,
            is_official,
            ignore: None,
            teams: Vec::new(),
        }
    }
}

/// Abstracts lat/lng access so geocoder helpers can be generic over both
/// `TeamData` and `EventData`.
pub trait HasLocation {
    fn lat(&self) -> Option<f64>;
    fn lng(&self) -> Option<f64>;
    fn set_lat_lng(&mut self, lat: f64, lng: f64);
    fn clear_location(&mut self);
    fn set_ignore(&mut self, val: bool);
    fn has_location(&self) -> bool {
        self.lat().is_some() && self.lng().is_some()
    }
}

impl HasLocation for TeamData {
    fn lat(&self) -> Option<f64> {
        self.tba.lat
    }
    fn lng(&self) -> Option<f64> {
        self.tba.lng
    }
    fn set_lat_lng(&mut self, lat: f64, lng: f64) {
        self.tba.lat = Some(lat);
        self.tba.lng = Some(lng);
    }
    fn clear_location(&mut self) {
        self.tba.lat = None;
        self.tba.lng = None;
    }
    fn set_ignore(&mut self, val: bool) {
        self.ignore = Some(val);
    }
}

impl HasLocation for EventData {
    fn lat(&self) -> Option<f64> {
        self.tba.lat
    }
    fn lng(&self) -> Option<f64> {
        self.tba.lng
    }
    fn set_lat_lng(&mut self, lat: f64, lng: f64) {
        self.tba.lat = Some(lat);
        self.tba.lng = Some(lng);
    }
    fn clear_location(&mut self) {
        self.tba.lat = None;
        self.tba.lng = None;
    }
    fn set_ignore(&mut self, val: bool) {
        self.ignore = Some(val);
    }
}
