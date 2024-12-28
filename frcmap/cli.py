import argparse
import datetime
import json
import os
import logging

from .frcmap import FRCMap
from .frcgeocoder import LocationDict

logging.basicConfig(
    level=logging.INFO,
    format="[%(asctime)s-%(levelname)s-%(name)s] %(message)s",
    datefmt="%H:%M",
)
log = logging.getLogger(__name__)


def run(root_path):
    log.info("Starting!")

    # Chech for API keys or create file if it doesn't exist
    try:
        import api_keys
    except:
        log.critical("Api Keys file not found, creating 'api_keys.py'. Add your keys there.")
        with open("api_keys.py", "w") as f:
            f.write(
                """
                # Fill in with your API keys:
                tba_key = '<KEY>'
                gmaps_key = '<KEY>'
                """
            )
            exit()

    # Helper function to point to things relative to this file

    def relative_path(path: str) -> str:
        return os.path.join(root_path, path)

    parser = argparse.ArgumentParser(description="Generate FRC Map")
    parser.add_argument(
        "-y",
        "--year",
        metavar="YEAR",
        type=int,
        help="FRC Season.",
        default=datetime.datetime.now().year,
    )

    parser.add_argument(
        "-t",
        "--team-locations",
        dest="teams",
        help="Path to TOML file with team manual location data.",
        default=relative_path("locations/teams.json"),
    )

    parser.add_argument(
        "-e",
        "--event-locations",
        dest="events",
        help="Path to TOML file with event manual location data.",
        default=relative_path("locations/events.json"),
    )

    parser.add_argument(
        "-l",
        "--location-archive",
        dest="archive",
        help="Path to location archive directory.",
        default=relative_path("locations/archive"),
    )

    parser.add_argument(
        "-c",
        "--cache-location",
        help="TBA API cache directory location.",
        default=relative_path("cache"),
    )

    parser.add_argument(
        "-o",
        "--output-location",
        help="Directory to write JSON output to.",
        default=relative_path("docs/data"),
    )

    args = parser.parse_args()

    # Perform arg validation
    cache = args.cache_location
    if os.path.exists(cache):
        if not os.path.isdir(cache):
            log.critical("Cache path is not a directory!")
            exit()
    else:
        log.info(f"Creating: {cache}")
        os.makedirs(cache, exist_ok=True)

    teams: LocationDict = {}
    if os.path.exists(args.teams):
        log.info(f"Loading team locations from:  {args.teams}")
        with open(args.teams, "rb") as f:
            teams: LocationDict = json.load(f)
            teams.pop("_comment", None)

    events: LocationDict = {}
    if os.path.exists(args.events):
        log.info(f"Loading event locations from: {args.events}")
        with open(args.events, "rb") as f:
            events: LocationDict = json.load(f)
            events.pop("_comment", None)

    # Make sure archive path exists.
    archive_path = args.archive
    if os.path.exists(archive_path):
        if not os.path.isdir(archive_path):
            log.critical("Archive path is not a directory!")
            exit()
    else:
        os.makedirs(archive_path, exist_ok=True)

    # Make sure output path exists.
    output = args.output_location
    if os.path.exists(output):
        if not os.path.isdir(output):
            log.critical("Output path is not a directory!")
            exit()
    else:
        os.makedirs(output, exist_ok=True)

    # Create main object
    frcmap = FRCMap(
        TbaApiKey=api_keys.tba_key,
        GMapsApiKey=api_keys.gmaps_key,
        FirstToken=api_keys.first_token,
        year=args.year,
        cache=cache,
        archive=archive_path,
        teams=teams,
        events=events,
    )

    frcmap.generate()
    frcmap.write(output)
