import datetime
import json
import os
import random
import re
from typing import Dict
import logging
import googlemaps

from frcmap.tbahelper import InfoDict, KeyList

LocationDict = Dict[str, Dict[str, float]]

log = logging.getLogger(__name__)


def make_team_address(data: Dict):
    def read(obj: Dict, key: str) -> str:
        def notNone(value):
            return value if value != None else ""

        if key in obj:
            return notNone(obj[key])
        else:
            return ""

    addr = " ".join(
        f"{read(data,'school_name')} {read(data,'city')} {read(data,'state_prov')} {read(data,'postal_code')} {read(data,'country')}".split()
    )
    return None if addr == "" else addr


class FRCGeocoder:
    UNKNOWN = {"lat": None, "lng": None}

    def __init__(
        self,
        gmaps_api_key: str,
        archive_path: str,
        teams: LocationDict,
        events: LocationDict,
    ) -> None:
        self.archive_path = archive_path
        self.teams = teams
        self.events = events
        self.gmaps = googlemaps.Client(key=gmaps_api_key)
        self.__read_archive()

    def __read_archive(self):
        files = [
            f
            for f in os.listdir(self.archive_path)
            if os.path.isfile(os.path.join(self.archive_path, f))
        ]
        years = {}
        for file in files:
            m = re.match(r"all_team_locations_(\d\d\d\d).json", file)
            if m == None:
                continue
            years[int(m.group(1))] = os.path.join(self.archive_path, file)

        #  Open the latest one
        if len(years) > 0:
            latest = years[max(years.keys())]
            log.info(f"Using location archive: {latest}")
            with open(latest, encoding="utf8") as f:
                self.archive = json.load(f)
        else:
            log.warning(
                f"Archive not available, geocoding will take a while and incurr in several API requests."
            )
            self.archive = {}

    def __save_archive(self, teams: InfoDict):
        data = {
            k0: {k1: v1 for k1, v1 in v0.items() if k1 in ["lat", "lng"]}
            for k0, v0 in teams.items()
        }
        name = f"all_team_locations_{datetime.datetime.now().year}.json"
        with open(os.path.join(self.archive_path, name), "w", encoding="utf8") as f:
            json.dump(data, f)

    def __randomnize_location(self, obj: Dict, sigma=0.001):
        obj["lat"] = random.gauss(obj["lat"], sigma)
        obj["lng"] = random.gauss(obj["lng"], sigma)

    def __geolocate_team(self, team: Dict) -> None:
        addr = make_team_address(team)
        if addr == None:
            log.error(f"Team {team['key']} has no address.")
            team.update(FRCGeocoder.UNKNOWN)
        else:
            log.info(f"Address for {team['key']}: {addr}")
            try:
                loc = self.gmaps.geocode(addr)[0]["geometry"]["location"]
                team.update(loc)
                log.info(f"Location: {loc}")
            except:
                log.error(f"Could not geocode address for team {team['key']}")
                team.update(FRCGeocoder.UNKNOWN)

    def __noloc(self, item):
        return item["lat"] == None or item["lng"] == None

    def __dedup_locations(self, objects: InfoDict, obj_type: str) -> None:
        def location_tuple(item):
            return (item["lat"], item["lng"])

        ulocs = dict()

        for key, obj in objects.items():
            if self.__noloc(obj):
                continue
            loc = location_tuple(obj)
            if loc in ulocs:
                log.warning(
                    f"{obj_type} {key} location overlaps with {ulocs[loc]}, randomnizing a bit!"
                )
                self.__randomnize_location(obj)
            else:
                ulocs[location_tuple(obj)] = key

    def populate_team_locations(self, teams: InfoDict) -> None:
        log.info("Geolocating teams.")
        # Find locations
        for key, team in teams.items():
            # First priority is the manual overrides
            if key in self.teams:
                team.update(self.teams[key])
            # Otherwise use archived location
            elif key in self.archive:
                team.update(self.archive[key])
            # If all else fails try geocoding the address
            else:
                log.warning(f"Geocoding team {key}")
                self.__geolocate_team(team)

        self.__dedup_locations(teams, "Team")

        self.__save_archive(teams)
        log.info("Geolocating teams finished.")

    def populate_event_locations(self, events: InfoDict) -> None:
        log.info("Geolocating events.")
        # Find locations
        for key, event in events.items():
            if key in self.events:
                event.update(self.events[key])
            if self.__noloc(event):
                event["ignore"] = True
                log.error(f"Event {key} has no location!")

        self.__dedup_locations(events, "Event")
        log.info("Geolocating events finished.")
