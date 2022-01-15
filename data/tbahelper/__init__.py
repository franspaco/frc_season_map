
import requests, json, datetime
from collections import defaultdict
import jsonloader

class TBAHelper:
    api = 'https://www.thebluealliance.com/api/v3'
    def __init__(self, auth_key, check_update=True):
        self.auth = auth_key
        self.headers = {
            'X-TBA-Auth-Key': auth_key
        }
        self.check_update = check_update
        self.memory = dict()
    
    def __load_cache(self, key):
        try:
            return jsonloader.loadfile(f"cache/{key}__cache.json")
        except:
            return None
    
    def __save_cache(self, key, route, data, last_modified):
        output = {
            'route': route,
            'last-modified': last_modified,
            'data': data
        }
        jsonloader.savefile(f"cache/{key}__cache.json", output)

    def __get_data(self, route):
        print(f"[TBA] Querying: '{route}' ... ", end='')
        name = route.replace("/","_")
        if name in self.memory:
            print("MEMORY!")
            return self.memory[name]
        header = self.headers.copy()
        cache = self.__load_cache(name)
        if cache is not None and not self.check_update:
            self.memory[name] = cache['data']
            print("CACHE!")
            return self.memory[name]
        if cache is not None and route == cache['route']:
            header['If-Modified-Since'] = cache['last-modified']
        r = requests.get(TBAHelper.api + route, headers=header)
        if r.status_code == 304:
            self.memory[name] = cache['data']
        elif r.status_code == 200:
            self.memory[name] = r.json()
            self.__save_cache(name, route, self.memory[name], r.headers['Last-Modified'])
        else:
            print(f"Error: {route}")
            raise Exception(f"Query error!\n{r.status_code}\n{r.text}")
        print("TBA!")
        return self.memory[name]


    def get_events_year(self, year):
        return self.__get_data(f"/events/{year}")

    def get_events_year_keys(self, year):
        return self.__get_data(f"/events/{year}/keys")

    def get_event_teams_keys(self, event_key):
        return self.__get_data(f"/event/{event_key}/teams/keys")

    def get_teams_keys(self):
        if 'teams' in self.memory:
            return self.memory['teams']
        teams = list()
        index = 0
        while True:
            page = self.__get_data(f"/teams/{index}/keys")
            if type(page) is list and len(page) == 0:
                break
            teams.extend(page)
            index += 1
        self.memory['teams'] = teams
        return teams

    def get_teams(self):
        if 'teams' in self.memory:
            return self.memory['teams']
        teams = list()
        index = 0
        while True:
            page = self.__get_data(f"/teams/{index}")
            if type(page) is list and len(page) == 0:
                break
            teams.extend(page)
            index += 1
        self.memory['teams'] = teams
        return teams
    
    def get_active_teams_year(self, year):
        name = f"active_{year}"
        if name in self.memory:
            return self.memory[name]
        cache = self.__load_cache(name)
        if cache is not None:
            self.memory[name] = cache['data']
            return cache['data']
        events = self.get_events_year_keys(year)
        teams = set()
        for event in events:
            participants = self.get_event_teams_keys(event)
            teams.update(participants)
        teams = list(teams)
        teams.sort(key=lambda x:int(x[3:]))
        self.memory[name] = teams
        self.__save_cache(name, "", teams, "")
        return teams

    def get_team_events_year(self, year):
        name = f"team_events_{year}"
        if name in self.memory:
            return self.memory[name]
        cache = self.__load_cache(name)
        if cache is not None:
            self.memory[name] = cache['data']
            return cache['data']
        team_events = defaultdict(list)
        events = self.get_events_year_keys(year)
        for event in events:
            participants = self.get_event_teams_keys(event)
            for team in participants:
                team_events[team].append(event)
        return dict(team_events)

        

        
