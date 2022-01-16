# FRC SEASON
import locations_utils
import argparse
import datetime
import jsonloader
import tbahelper
import googlemaps
import api_keys
import toml

parser = argparse.ArgumentParser(description='Generate FRC Map')
parser.add_argument('--year', metavar='YEAR', type=int,
                    help='FRC Season.', default=datetime.datetime.now().year)
parser.add_argument('--reload', action='store_true')
parser.add_argument('--teams-locations',
                    dest="teams",
                    help='Path to TOML file with team location data.', 
                    default='locations/teams.toml')
parser.add_argument('--events-locations',
                    dest="events",
                    help='Path to TOML file with event location data.',
                    default='locations/events.toml')
args = parser.parse_args()

# Just for convenience
year = args.year

# Create TBA helper
tba = tbahelper.TBAHelper(api_keys.tba_key, args.reload)

# Create GMaps connection
gmaps = googlemaps.Client(key=api_keys.gmaps_key)

# Get manual inputs
manual_teams = toml.load(args.teams)
manual_events = toml.load(args.events)

# Fetch all teams, get their location and store it in a file.
team_locations = locations_utils.get_teams_locations(
    tba, gmaps, year, manual_teams)

# Get keys of all teams active on a given year
active_teams = tba.get_active_teams_year(year)

# Big dictionary of all teams
all_teams = {item['key']: item for item in tba.get_teams()}

# Dictionary of all events on a given year
all_events = {item['key']: item for item in tba.get_events_year(year)}

# Process event location data
locations_utils.process_event_locations(all_events, manual_events)

# Dictionary of teams and the events they will be attending
team_events = tba.get_team_events_year(year)

# Active team data
team_data = dict()

# Populate active team data with location and events
for team_key in active_teams:
    team_data[team_key] = all_teams[team_key].copy()
    team_data[team_key]['events'] = team_events[team_key]
    if team_key in team_locations:
        team_data[team_key].update(team_locations[team_key])

# Populate event data with team list
for key, value in all_events.items():
    value['teams'] = tba.get_event_teams_keys(key)

data = {
    'teams': team_data,
    'events': all_events
}

jsonloader.savefile(f"../docs/data/season_{year}.json", data)
