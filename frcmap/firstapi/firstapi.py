from typing import Dict
from urllib.parse import urljoin
from requests_cache import CachedSession
import base64
import re


def update_field(a: Dict[str, str], b: Dict[str, str], name: str):
    if name in a:
        b[name] = a[name]


class FirstAPI:
    api = "https://frc-api.firstinspires.org/v3.0/"

    def __init__(self, *, first_api_token: str, cache_path: str):
        self.session = CachedSession(
            cache_path,
            backend="filesystem",
            serializer="json",
            cache_control=True,
        )
        token = base64.b64encode(first_api_token.encode()).decode()
        self.session.headers["Authorization"] = f"Basic {token}"
        self.data_cache = {}

    def __get(self, path: str):
        r = self.session.get(urljoin(FirstAPI.api, path))
        return r.json()

    def get_event(self, year: int, code: str):
        return self.__get(f"{year}/events/{code}")

    def enhance_event_data(self, event: Dict):
        # m = re.match(r"(\d{4})([a-z]+)", event["key"])
        # if m is None:
        #     raise Exception(f"Event key {event['key']} is not in format 'YYYYcode'")
        # first_data = self.get_event(int(m.group(1)), m.group(2))
        first_data = self.get_event(event["year"], event["first_event_code"])
        update_field(event, first_data, "venue")
        update_field(event, first_data, "address")
