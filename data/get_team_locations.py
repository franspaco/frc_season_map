import api_keys
import googlemaps, json
import tbahelper
import jsonloader
tba = tbahelper.TBAHelper(api_keys.tba_key, False)
gmaps = googlemaps.Client(key=api_keys.gmaps_key)

locations = jsonloader.loadfile('data/all_team_locations_2019.json')

teams = tba.get_teams()

def notNone(value):
    if value is not None:
        return value
    else:
        return ''

for data in teams:
    key = data['key']
    if key not in locations or locations[key]['lat'] is None or locations[key]['lng'] is None:
        print(key)
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
            print('Done!')
        except:
            print('Error: ' + address)
        break
        
jsonloader.savefile('data/all_team_locations_2019.json', locations)