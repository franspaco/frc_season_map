import re
from collections import defaultdict
from typing import Dict, List
from requests_cache import CachedSession
from urllib.parse import urljoin

from .event_type import EventType

KeyList = List[str]
InfoDict = Dict[str, Dict[str, str]]
InfoList = List[Dict[str, str]]


def extend_event(event: Dict[str, str]):
    event["is_cmp"] = EventType.isChampionship(event["event_type"])
    event["is_official"] = EventType.isOfficial(event["event_type"])
    return event

REGULAR_EVENT_REGEX = re.compile(r"^20\d\d[a-z]+$")

def is_regular_event_key(event_key: str) -> bool:
    return REGULAR_EVENT_REGEX.match(event_key) is not None

class TBAHelper:
    api = "https://www.thebluealliance.com/api/v3/"

    def __init__(self, *, tba_api_key: str, cache_path: str):
        self.session = CachedSession(
            cache_path,
            backend="filesystem",
            serializer="json",
            cache_control=True,
        )
        self.session.headers["X-TBA-Auth-Key"] = tba_api_key
        self.data_cache = {}

    def __get(self, path: str):
        r = self.session.get(urljoin(TBAHelper.api, path))
        return r.json()

    def get_events(self, year: int) -> InfoDict:
        return {event["key"]: extend_event(event) for event in self.__get(f"events/{year}") if is_regular_event_key(event["key"])}

    def get_event_keys(self, year: int) -> KeyList:
        return [event_key for event_key in self.__get(f"events/{year}/keys") if is_regular_event_key(event_key)]

    def get_event_team_keys(self, event_key: str) -> KeyList:
        teams: KeyList = self.__get(f"event/{event_key}/teams/keys")
        for team in teams:
            if not team.startswith("frc"):
                raise Exception(f"Got invalid team key '{team}' in event '{event_key}' from TBA")
        return teams

    def get_teams_page(self, page: int) -> InfoDict:
        return {team["key"]: team for team in self.__get(f"teams/{page}")}

    def get_team_keys_page(self, page: int) -> KeyList:
        return self.__get(f"teams/{page}/keys")

    def get_teams(self) -> InfoDict:
        teams = dict()
        index = 0
        while True:
            page = self.get_teams_page(index)
            if type(page) is dict and len(page) == 0:
                break
            teams.update(page)
            index += 1
        return teams

    def get_team_keys(self) -> KeyList:
        teams = list()
        index = 0
        while True:
            page = self.get_team_keys_page(index)
            if type(page) is list and len(page) == 0:
                break
            teams.extend(page)
            index += 1
        return teams

    def get_active_teams(self, year: int) -> KeyList:
        events = self.get_event_keys(year)
        teams = set()
        for event in events:
            teams.update(self.get_event_team_keys(event))
        return sorted(list(teams))

    def get_team_events(self, year: int) -> Dict[str, List[str]]:
        events = self.get_event_keys(year)
        teams = defaultdict(list)
        for event in events:
            for team in self.get_event_team_keys(event):
                teams[team].append(event)
        return dict(teams)
