# FIRST Robotics Challenge (FRC) Season Map

_Updated for 2022_

[CLICK HERE TO VIEW ON frcmap.com](http://frcmap.com)

This map shows all FRC teams and events registered for the current season.

Here is the code I use to fetch all the required data from The Blue Alliance (thanks!).
It basically feteches all teams and all events for the desired year and outputs it as a simple JSON that's used by the front end.

A Google Maps API key is required to get geolocations from the teams' addresses.
A TBA api key is required to get team data.

### Placement Errors
If a team is misplaced or missing and you have the correct location info feel free to submit an issue with the correct data. (and preferably some statement of affiliation to corroborate the info)

### Contribution
I'm open to suggestions and contributions! Let me know if you have any ideas to make this better.

### Setup
To use the data collector, create the file `data/api_keys.py` and save it with the contents:
```python
tba_key = '<YOUR KEY HERE>'
gmaps_key = '<YOUR KEY HERE>'
```

### Running
To run just execute: `main.py`

This will:

1. Run the script that looks up teams' locations. Only the teams not found in `data/data/all_team_locations_<year-1>.json` will be looked for.
2. Fetch data for all teams.
3. Fetch data for all events.
4. Filter teams, leaving only those registered for events in the current year.
5. Cross reference teams and events.
6. Export `docs/data/season_<year>.json`


### Other details
Currently, the code assumes cached data is always valid. To force the program to check for updates from TBA, you must change this line:

```python
tba = tbahelper.TBAHelper(api_keys.tba_key, False)
```
to:
```python
tba = tbahelper.TBAHelper(api_keys.tba_key, True)
```
in the files:
* `main.py`
* `get_team_locations.py`

This will use TBA's own update-checking mechanism detailed in their API docs.


