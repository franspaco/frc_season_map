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

    let report = GenerationReport {
        year,
        teams,
        events,
        active_team_keys_missing_from_team_feed: missing,
    };
    Ok(serde_json::to_value(report)?)
}

pub fn write_generation_report(
    report_dir: &Path,
    year: u32,
    report: &serde_json::Value,
) -> Result<()> {
    let report_json = serde_json::to_string(report)?;
    let report_json = report_json
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e");

    fs::create_dir_all(&report_dir)?;
    let path = report_dir.join(format!("{}.html", year));
    fs::write(&path, render_html(year, &report_json))
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

fn render_html(year: u32, report_json: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>FRC Season Map Generation Report - {year}</title>
  <style>
    :root {{ color-scheme: light dark; font-family: system-ui, sans-serif; }}
    body {{ margin: 2rem auto; max-width: 1600px; padding: 0 1rem; }}
    input {{ font: inherit; padding: .5rem; width: min(100%, 40rem); }}
    table {{ border-collapse: collapse; width: 100%; margin: 1rem 0 2rem; }}
    th, td {{ border: 1px solid #8886; padding: .45rem; text-align: left; vertical-align: top; }}
    th {{ position: sticky; top: 0; background: Canvas; }}
    .summary {{ display: flex; flex-wrap: wrap; gap: 1rem; }}
    .summary p {{ border: 1px solid #8886; border-radius: .4rem; margin: 0; padding: .75rem; }}
    .ok {{ color: #198754; }} .warn {{ color: #b7791f; }} .bad {{ color: #c53030; }}
    pre {{ margin: .5rem 0 0; max-width: 70rem; overflow: auto; white-space: pre-wrap; }}
    details {{ min-width: 18rem; }} .hidden {{ display: none; }}
  </style>
</head>
<body>
  <h1>FRC Season Map Generation Report: {year}</h1>
  <p>This report retains every team and event received from TBA, each location decision, and the
  final map records. Expand a raw record to inspect all source fields captured during generation.</p>
  <label>Filter teams and events <input id="filter" type="search" placeholder="team key, event key, name, city, source..."></label>
  <div id="summary" class="summary"></div>
  <h2>Teams received from TBA</h2>
  <table id="teams"><thead><tr><th>Team</th><th>Active</th><th>Map</th><th>Location resolution</th><th>Season events</th><th>Raw and map records</th></tr></thead><tbody></tbody></table>
  <h2>Events received from TBA</h2>
  <table id="events"><thead><tr><th>Event</th><th>Map</th><th>Location resolution</th><th>Teams returned for event</th><th>Raw and map records</th></tr></thead><tbody></tbody></table>
  <script id="report-data" type="application/json">{report_json}</script>
  <script>
    const report = JSON.parse(document.getElementById('report-data').textContent);
    const filter = document.getElementById('filter');
    const text = value => value == null ? '—' : String(value);
    const node = (tag, value) => {{ const element = document.createElement(tag); element.textContent = text(value); return element; }};
    const location = value => !value ? 'not processed' : `${{value.source}}; ${{value.latitude ?? '—'}}, ${{value.longitude ?? '—'}}${{value.ignored ? '; ignored' : ''}}${{value.randomized_to_avoid_collision ? '; randomized to avoid collision' : ''}}`;
    const jsonDetails = (label, value) => {{
      const details = document.createElement('details');
      details.append(node('summary', label));
      const pre = node('pre', JSON.stringify(value, null, 2));
      details.append(pre);
      return details;
    }};
    const renderTable = (id, rows, cells) => {{
      const body = document.querySelector(`#${{id}} tbody`);
      for (const row of rows) {{
        const tr = document.createElement('tr');
        for (const cell of cells(row)) {{ const td = document.createElement('td'); td.append(cell); tr.append(td); }}
        body.append(tr);
      }}
    }};
    const count = report.teams.reduce((totals, team) => {{
      totals.active += team.active; totals.inMap += team.included_in_map;
      totals.withoutLocation += !team.location?.latitude || !team.location?.longitude;
      totals.randomized += Boolean(team.location?.randomized_to_avoid_collision);
      return totals;
    }}, {{active: 0, inMap: 0, withoutLocation: 0, randomized: 0}});
    document.getElementById('summary').replaceChildren(
      node('p', `Teams received: ${{report.teams.length}}`),
      node('p', `Active teams: ${{count.active}}`),
      node('p', `Teams in map: ${{count.inMap}}`),
      node('p', `Teams without location: ${{count.withoutLocation}}`),
      node('p', `Randomized locations: ${{count.randomized}}`),
      node('p', `Events received: ${{report.events.length}}`),
      node('p', `Events in map: ${{report.events.filter(event => event.included_in_map).length}}`),
      node('p', `Active teams missing from team feed: ${{report.active_team_keys_missing_from_team_feed.length}}`)
    );
    renderTable('teams', report.teams, team => [
      node('span', `${{team.key}} — ${{team.raw.nickname ?? team.raw.name ?? 'unnamed'}}`),
      node('span', team.active ? 'yes' : 'no'),
      node('span', team.included_in_map ? 'included' : 'excluded'),
      node('span', location(team.location)),
      node('span', team.event_keys.join(', ') || 'none'),
      jsonDetails('Show raw TBA and final map record', {{raw: team.raw, map: team.map_record}})
    ]);
    renderTable('events', report.events, event => [
      node('span', `${{event.key}} — ${{event.raw.name ?? 'unnamed'}}`),
      node('span', event.included_in_map ? 'included' : 'excluded (non-regular event key)'),
      node('span', location(event.location)),
      node('span', event.team_keys.join(', ') || 'not requested'),
      jsonDetails('Show raw TBA and final map record', {{raw: event.raw, map: event.map_record}})
    ]);
    filter.addEventListener('input', () => {{
      const query = filter.value.toLowerCase();
      document.querySelectorAll('tbody tr').forEach(row => row.classList.toggle('hidden', !row.textContent.toLowerCase().includes(query)));
    }});
  </script>
</body>
</html>"#
    )
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

        fs::remove_dir_all(directory).unwrap();
    }
}
