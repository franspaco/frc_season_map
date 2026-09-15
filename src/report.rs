use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::{
    geocoder::types::LocationResolution,
    map_types::{EventData, TeamData},
    tba::types::{TbaEvent, TbaTeam},
};

#[derive(Serialize)]
struct GenerationReport {
    year: u32,
    teams: Vec<ReportTeam>,
    events: Vec<ReportEvent>,
    active_team_keys_missing_from_team_feed: Vec<String>,
}

#[derive(Serialize)]
struct ReportTeam {
    key: String,
    raw: TbaTeam,
    active: bool,
    included_in_map: bool,
    location: Option<LocationResolution>,
    event_keys: Vec<String>,
    map_record: Option<TeamData>,
}

#[derive(Serialize)]
struct ReportEvent {
    key: String,
    raw: TbaEvent,
    included_in_map: bool,
    location: Option<LocationResolution>,
    team_keys: Vec<String>,
    map_record: Option<EventData>,
}

#[allow(clippy::too_many_arguments)]
pub fn build_generation_report(
    year: u32,
    raw_teams: &HashMap<String, TbaTeam>,
    raw_events: &HashMap<String, TbaEvent>,
    active_team_keys: &[String],
    team_events: &HashMap<String, Vec<String>>,
    map_teams: &HashMap<String, TeamData>,
    map_events: &HashMap<String, EventData>,
    team_locations: &HashMap<String, LocationResolution>,
    event_locations: &HashMap<String, LocationResolution>,
) -> Result<serde_json::Value> {
    let active: HashSet<&str> = active_team_keys.iter().map(String::as_str).collect();

    let mut teams: Vec<ReportTeam> = raw_teams
        .iter()
        .map(|(key, raw)| ReportTeam {
            key: key.clone(),
            raw: raw.clone(),
            active: active.contains(key.as_str()),
            included_in_map: map_teams.contains_key(key),
            location: team_locations.get(key).cloned(),
            event_keys: team_events.get(key).cloned().unwrap_or_default(),
            map_record: map_teams.get(key).cloned(),
        })
        .collect();
    teams.sort_by(|left, right| left.key.cmp(&right.key));

    let mut events: Vec<ReportEvent> = raw_events
        .iter()
        .map(|(key, raw)| {
            let map_record = map_events.get(key).cloned();
            let team_keys = map_record
                .as_ref()
                .map(|event| event.teams.clone())
                .unwrap_or_default();
            ReportEvent {
                key: key.clone(),
                raw: raw.clone(),
                included_in_map: map_record.is_some(),
                location: event_locations.get(key).cloned(),
                team_keys,
                map_record,
            }
        })
        .collect();
    events.sort_by(|left, right| left.key.cmp(&right.key));

    let mut missing: Vec<String> = active_team_keys
        .iter()
        .filter(|key| !raw_teams.contains_key(key.as_str()))
        .cloned()
        .collect();
    missing.sort();

    Ok(serde_json::to_value(GenerationReport {
        year,
        teams,
        events,
        active_team_keys_missing_from_team_feed: missing,
    })?)
}

pub fn write_generation_report(
    report_dir: &Path,
    year: u32,
    report: &serde_json::Value,
) -> Result<()> {
    let report_json = serde_json::to_string(report)?
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e");

    fs::create_dir_all(report_dir)?;
    let path = report_dir.join(format!("{}.html", year));
    fs::write(
        &path,
        format!(
            include_str!("generation_report.html"),
            year = year,
            report_json = report_json
        ),
    )
    .with_context(|| format!("Failed to write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::write_generation_report;

    #[test]
    fn writes_an_escaped_report_at_the_requested_path() {
        let directory =
            std::env::temp_dir().join(format!("frc-season-map-report-test-{}", std::process::id()));
        let report = serde_json::json!({ "name": "</script><script>unsafe()</script>" });

        write_generation_report(&directory, 2026, &report).unwrap();

        let content = fs::read_to_string(directory.join("2026.html")).unwrap();
        assert!(content.contains("FRC Season Map Generation Report: 2026"));
        assert!(content.contains("\\u003c/script\\u003e"));
        assert!(!content.contains("</script><script>unsafe()"));
        assert!(content.contains("const formatLocationResolution"));
        assert!(!content.contains("const location ="));

        fs::remove_dir_all(directory).unwrap();
    }
}
