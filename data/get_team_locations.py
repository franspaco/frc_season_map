import api_keys
import googlemaps, json
import tbahelper
import jsonloader
import settings
import random

tba = tbahelper.TBAHelper(api_keys.tba_key, False)
gmaps = googlemaps.Client(key=api_keys.gmaps_key)



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

def populate_data_file(origin=None, destination=None, randomnize_overlaps=True):
    year = settings.year
    origin = origin or (year-1)
    destination = destination or year

    locations = jsonloader.loadfile(f"data/all_team_locations_{origin}.json")

    teams = tba.get_teams()

    # To avoid location overlaps keep tabs on all unique locations
    ulocs = set()

    for data in teams:
        key = data['key']
        if key not in locations or locations[key]['lat'] is None or locations[key]['lng'] is None:
            print(f"Looking for: {key}")
            address = f"{notNone(data['city'])} {notNone(data['state_prov'])} {notNone(data['postal_code'])} {notNone(data['country'])}"
            if address == '' or address.isspace():
                print('No address!')
                continue
            try:
                # print(address)
                loc = gmaps.geocode(address)
                # print(loc)
                pos = loc[0]['geometry']['location']
                locations[key] = {'lat': pos['lat'], 'lng':pos['lng']}
                print('Found!')
            except:
                print(key + '\nError: ' + address)
                break

        if tup(locations[key]) in ulocs:
            # Collision, ramdomnize a liiittle but
            randomnize_location(locations[key])
        
        ulocs.add(tup(locations[key]))
            
    jsonloader.savefile(f"data/all_team_locations_{destination}.json", locations)