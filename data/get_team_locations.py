import jsonloader
import random
import os
import re

def notNone(value):
    if value is not None:
        return value
    else:
        return ''


def tup(item):
    return (item["lat"], item["lng"])


def randomnize_location(team_location, sigma=0.01):
    team_location["lat"] = random.gauss(team_location["lat"], sigma)
    team_location["lng"] = random.gauss(team_location["lng"], sigma)


def extract_year_form_file(filename):
    match = re.match(r'all_team_locations_(\d\d\d\d).json', filename)
    return int(match.group(1))


def get_last_year_with_data(last=None):

    years = [extract_year_form_file(filename) for filename in os.listdir('data') if filename.startswith('all_team_locations_')]

    years.sort()

    if last is not None:
        years = list(filter(lambda x:x<=last, years))
    
    if len(years) == 0:
        return None

    return years[-1]

def populate_data_file(tba, gmaps, year=None, randomnize_overlaps=True):
    print(f"[LOC] Populating data for {year}")
    last = get_last_year_with_data(year)

    if last is not None:
        print(f"[LOC] Reusing data from {last}")
        locations = jsonloader.loadfile(f"data/all_team_locations_{last}.json")
    else:
        locations = {}

    teams = tba.get_teams()
    print(f"[LOC] TEAMS {len(teams)}")

    # To avoid location overlaps keep tabs on all unique locations
    ulocs = set()

    critical_error = []

    for data in teams:
        key = data['key']
        if key not in locations or locations[key]['lat'] is None or locations[key]['lng'] is None:
            print(f"[LOC] Looking for: {key}")
            address = f"{notNone(data['city'])} {notNone(data['state_prov'])} {notNone(data['postal_code'])} {notNone(data['country'])}"
            if address == '' or address.isspace():
                print('[LOC] No address!')
                continue
            try:
                # print(address)
                loc = gmaps.geocode(address)
                # print(loc)
                pos = loc[0]['geometry']['location']
                locations[key] = {'lat': pos['lat'], 'lng': pos['lng']}
                print('[LOC] Found!')

                if tup(locations[key]) in ulocs:
                    # Collision, ramdomnize a liiittle bit
                    randomnize_location(locations[key])
                ulocs.add(tup(locations[key]))
            except Exception as ex:
                msg = f"Failed to decode: Key={key}, address: {address}"
                print(f"[LOC] {msg}")
                print(ex)
                critical_error.append(msg)

    locations_file = f"data/all_team_locations_{year}.json"
    jsonloader.savefile(
        locations_file, locations)

    if critical_error:
        print("[LOC] Error:")
        for item in critical_error:
            print(f"\t{item}")
        exit(-1)
    
    return locations
