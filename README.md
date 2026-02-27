# FIRST Robotics Challenge (FRC) Season Map

*Updated for 2026*

[CLICK HERE TO VIEW ON frcmap.com](http://frcmap.com)

This map shows all FRC teams and events registered for the current season.

## Placement Errors

If a team is misplaced or missing and you have the correct location info please
submit an issue with the correct data. (and preferably some statement of
affiliation to corroborate the info)

You can also submit a pull request adding your team's correct location in
`locations/teams.toml`.

If you fork+clone the repo and have python, you can use `add_team.py` to easily
add/update a team.

## Contributing

I'm open to suggestions and contributions! Let me know if you have any ideas to
make this better. To be frank, I'm not sure I have the time or energy to drive a
major scope change in this project, feel more than welcome to contribute new
features.

### JSON Validation

This repository includes automatic JSON validation to prevent syntax errors that
can break the map application.

- All JSON files are automatically validated when you create a pull request or
  push to the main branch
- You can manually validate JSON files by running: `python3 validate_json.py`
- The validation script checks all `.json` files in the repository (excluding
  `.git` directory)
- If you're adding or modifying JSON data, make sure to test it locally first

**Common JSON errors to avoid:**

- Missing commas between objects or array elements
- Trailing commas (not allowed in JSON)
- Unclosed brackets or braces
- Invalid escape sequences in strings

### Building and Stuff

#### Requirements

- Rust >1.90
- TBA API key
- FIRST API key
- Google Maps Geocoding API key

#### Setup

```toml
# Fill in with your API keys:
tba_key = "..."
gmaps_key = "..."
# Format: "username:auth_key" (will be base64-encoded automatically)
first_token = "..."
```

#### Running

To run just execute:

```bash
cargo run
```

This will:

1. Run the script that looks up teams' locations. The script will look for
   manial overrides, then for archived location data, finally will try to get it
   from google maps.
2. Fetch data for all teams.
3. Fetch data for all events.
4. Filter teams, leaving only those registered for events in the current year.
5. Cross reference teams and events.
6. Export `docs/data/season_<year>.json`

To explore other options, run:

```bash
cargo run -- --help
```

## FAQ

<details>
<summary>Why is my team in the wrong place?</summary>

See [Placement Errors](#placement-errors) above for info on how to solve this.

The TLDR is one of the following reasons:

1. Lack of specific information. Most teams can only be placed in a
   city/town/zip code with publicly available info, anything beyond that is
   probably random. TBA/FIRST (possibly safety reasons) don't publicly expose a
   team's full address. Generally the best they can do is a city name and a ZIP
   code. Turning that into a precise location leads to a lot of variability. I
   try to use the school name, when available, for further accuracy, but that
   doesn't always work.
2. Your team has moved. I very aggressively cache known locations to not abuse
   my Google Maps API quota, I try to refresh every now and then, but generally
   avoid re-geolocating all teams.

</details>

<details>
<summary>Why is this now Rust?</summary>

No one has actually asked this, but in case anyone cares, because I was annoyed
with old Python code I did not want to continue maintaining.

This is a project I pick back up every one in a while to unwind ~~and remember
when I had hobbies and I enjoyed them.~~

</details>

<details>
<summary>When do you update this?</summary>

I try to do it early in the season, as time permits. By all means, feel free to
bother me if I'm too late.

</details>

<details>
<summary>Why is X event/team missing?</summary>

I try to capture everything, but every year something in the API changes and I
end up missing something. Please file an issue if you find something wrong.

</details>

## Acknowledgements

- [The Blue Alliance](https://www.thebluealliance.com/): the main data source
  for this project.
- @rdelfin: ~~he missed a comma once and broke the entire website for a month~~
  has helped me keep track of what's going on here.
- @Chris857 for consistently finding and correcting location errors.
