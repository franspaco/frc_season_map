import json
import logging
import os
from .tbahelper import TBAHelper
from .frcgeocoder import FRCGeocoder, LocationDict

log = logging.getLogger(__name__)

class FRCMap:
    def __init__(self, *, TbaApiKey: str, GMapsApiKey: str, year: int, cache: str, archive: LocationDict, teams: LocationDict, events: LocationDict):
        self.year = year
        self.geocoder = FRCGeocoder(GMapsApiKey, archive, teams, events)
        self.tba = TBAHelper(TbaApiKey=TbaApiKey, cache_path=cache)
        self.data = None

    def generate(self) -> None:
        log.info("Fetching all teams...")
        teams = self.tba.get_teams()
        log.info(f"Found {len(teams)} teams.")

        log.info(f"Fetching all events in {self.year}...")
        events = self.tba.get_events(self.year)
        log.info(f"Found {len(events)} events.")

        log.info(f"Fetching active teams in {self.year}...")
        active = self.tba.get_active_teams(self.year)
        log.info(f"Found {len(active)} active teams.")

        self.geocoder.populate_team_locations(teams)
        self.geocoder.populate_event_locations(events)

        team_events = self.tba.get_team_events(self.year)

        team_data = dict()
        for tkey in active:
            team_data[tkey] = teams[tkey]
            team_data[tkey]['events'] = team_events[tkey]
        
        for ekey, event in events.items():
            event['teams'] = self.tba.get_event_team_keys(ekey)
        
        self.data = {
            'teams': team_data,
            'events': events
        }


    def write(self, output:str) -> None:
        if self.data == None:
            raise RuntimeError("Data has not been generated yet!")
        with open(os.path.join(output, f"season_{self.year}_pretty.json"), 'w', encoding='utf8') as f:
            json.dump(self.data, f, indent=4)
        with open(os.path.join(output, f"season_{self.year}.json"), 'w', encoding='utf8') as f:
            json.dump(self.data, f)
