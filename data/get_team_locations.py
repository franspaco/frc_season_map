import api_keys
import googlemaps, json
import tbahelper
tba = tbahelper.TBAHelper(api_keys.tba_key, False)
gmaps = googlemaps.Client(key=api_keys.gmaps_key)

with open('data/all_team_locations.json', 'r', encoding='utf-8') as f:
    locations = json.load(f)

teams = tba.get_teams()

def notNone(value):
    if value is not None:
        return value
    else:
        return ''

for data in teams:
    if data['key'] not in locations:
        print(data['key'])
        address = f"{notNone(data['city'])} {notNone(data['state_prov'])} {notNone(data['postal_code'])} {notNone(data['country'])}"
        if address == '' or address.isspace():
            print('No address!')
            continue
        try:
            loc = gmaps.geocode(address)[0]['geometry']['location']
            locations[data['key']] = {'lat': loc['lat'], 'lng':loc['lng']}
            print('Done!')
        except:
            print('Error: ' + address)
        

with open('data/all_team_locations_2019.json', 'w', encoding='utf-8') as f:
    json.dump(locations, f)