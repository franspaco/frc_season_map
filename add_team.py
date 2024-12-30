#!/usr/bin/env python3

import bisect
import json
import argparse
from collections import OrderedDict

parser = argparse.ArgumentParser("Add a team to the override list")

parser.add_argument("team", help="The team number to add")
parser.add_argument("lat", help="The latitude of the team", type=str)
parser.add_argument("lng", help="The longitude of the team", type=str)
parser.add_argument(
    "--file",
    help="The file to write to",
    default="locations/teams.json",
)

args = parser.parse_args()

new_team = (
    f"frc{args.team}",
    {
        "lat": float(args.lat.replace(",", "")),
        "lng": float(args.lng.replace(",", "")),
    },
)

def team_key_as_int(team):
    return int(team[0][3:])

with open(args.file, "r") as f:
    teams = [(k,v) for k,v in json.load(f, object_pairs_hook=OrderedDict).items()]

bisect.insort(teams, new_team, key=team_key_as_int)

teams = OrderedDict(teams)

with open(args.file, "w") as f:
    json.dump(teams, f, indent=4)
