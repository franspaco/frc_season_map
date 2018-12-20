import jsonloader
import tbahelper
import api_keys
tba = tbahelper.TBAHelper(api_keys.tba_key, False)

year = 2019

active_teams = tba.get_active_teams_year(year)

all_teams = tba.get_teams()
all_teams = {item['key']:item for item in all_teams}

all_events = tba.get_events_year(year)
all_events = {item['key']:item for item in all_events}

team_locations = jsonloader.loadfile('data/all_team_locations_2019.json')
team_events = tba.get_team_events_year(year)

for team_key in active_teams:
    if team_key in team_locations:
        all_teams[team_key]['lat'] = team_locations[team_key]['lat']
        all_teams[team_key]['lng'] = team_locations[team_key]['lng']
    all_teams[team_key]['events'] = team_events[team_key]

for key,value in all_events.items():
    value['teams'] = tba.get_event_teams_keys(key)

data = {
    'teams': all_teams,
    'events': all_events
}

jsonloader.savefile('data/season_2019.json', data)
