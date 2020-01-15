# FRC SEASON

import jsonloader
import tbahelper
import api_keys
import settings

year = settings.year

# Run the other script
import get_team_locations

# Edit params to adjust desired source
get_team_locations.populate_data_file(year, year)

tba = tbahelper.TBAHelper(api_keys.tba_key, False)

active_teams = tba.get_active_teams_year(year)

all_teams = tba.get_teams()
all_teams = {item['key']:item for item in all_teams}

all_events = tba.get_events_year(year)
all_events = {item['key']:item for item in all_events}

team_locations = jsonloader.loadfile(f"data/all_team_locations_{year}.json")
team_events = tba.get_team_events_year(year)

team_data = dict()

for team_key in active_teams:
    team_data[team_key] = all_teams[team_key].copy()
    team_data[team_key]['events'] = team_events[team_key]
    if team_key in team_locations:
        team_data[team_key].update(team_locations[team_key])

for key,value in all_events.items():
    value['teams'] = tba.get_event_teams_keys(key)

data = {
    'teams': team_data,
    'events': all_events
}

jsonloader.savefile(f"../docs/data/season_{year}.json", data)
