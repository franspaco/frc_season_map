import api_keys
import googlemaps, json
import tbahelper
import jsonloader
import settings
tba = tbahelper.TBAHelper(api_keys.tba_key, False)
gmaps = googlemaps.Client(key=api_keys.gmaps_key)



def notNone(value):
    if value is not None:
        return value
    else:
        return ''

def populate_data_file(origin=None, destination=None):
    year = settings.year
    origin = origin or (year-1)
    destination = destination or year

    locations = jsonloader.loadfile(f"data/all_team_locations_{origin}.json")

    teams = tba.get_teams()

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
            
    jsonloader.savefile(f"data/all_team_locations_{destination}.json", locations)