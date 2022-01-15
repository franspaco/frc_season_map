# FRC SEASON
import get_team_locations
import argparse
import datetime
import jsonloader
import tbahelper
import googlemaps
import api_keys

parser = argparse.ArgumentParser(description='Generate FRC Map')
parser.add_argument('--year', metavar='YEAR', type=int,
                    help='FRC Season.', default=datetime.datetime.now().year)
parser.add_argument('--reload', action='store_true')
args = parser.parse_args()

# Just for convenience
year = args.year

# Create TBA helper
tba = tbahelper.TBAHelper(api_keys.tba_key, args.reload)

# Create GMaps connection
gmaps = googlemaps.Client(key=api_keys.gmaps_key)

# Fetch all teams, get their location and store it in a file.
team_locations = get_team_locations.populate_data_file(tba, gmaps, year)

# Get keys of all teams active on a given year
active_teams = tba.get_active_teams_year(year)

# Big dictionary of all teams
all_teams = {item['key']: item for item in tba.get_teams()}

# Dictionary of all events on a given year
all_events = {item['key']: item for item in tba.get_events_year(year)}

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
