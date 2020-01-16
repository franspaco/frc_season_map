# Tool to count collisions in team geo-locations

import data.jsonloader as jsonloader

data = jsonloader.loadfile("data/data/all_team_locations_2020.json")

locs = dict()

def tup(item):
    return hash((item["lat"], item["lng"]))

collisions = set()
for key,value in data.items():
    loc = tup(value)
    if loc in locs:
        collisions.add(loc)
        locs[loc].append(key)
    else:
        locs[loc] = [key]

collisions = list(collisions)

print(f"Found: {len(collisions)} collisions.")

for col in collisions:
    for team in locs[col]:
        print(team)
    print()
