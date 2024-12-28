let APP = {
    year: 2025,
    records: [2025, 2024, 2023, 2022, 2020, 2019],
    markers: {
        red: "markers/red_marker.png",
        green: "markers/green_marker.png",
        first: "markers/first_marker.png",
    },
    team_markers: [],
    event_markers: [],
    champ_markers: [],
    legends: {
        l_rookie: "#7C008F",
        l_0: "#0000FF",
        l_1: "#0033CC",
        l_2: "#006699",
        l_3: "#009966",
        l_4: "#00CC33",
        l_5: "#00FF00",
        l_event: "#FF0000",
        l_champ: "#FF6600",
    },
    click_mode: false,
    mobile: false,
};

async function initMap() {
    var center = {
        lat: 26.449372,
        lng: -99.131382,
    };
    APP.map = new google.maps.Map(document.getElementById("map"), {
        zoom: 4,
        center: center,
        mapTypeControl: false,
        streetViewControl: false,
    });
    APP.init();
}

APP.tba_link = function (type, key) {
    window.open(`https://www.thebluealliance.com/${type}/${key}`, "_blank");
}.bind(APP);

APP.create_marker_listeners = function (marker, element, type, key) {
    if (!APP.mobile) {
        marker.addListener("click", () => {
            element.visible = !element.visible;
        });
        marker.addListener("mouseover", () => {
            if (!element.visible) {
                APP.show_edges(element);
            }
        });
        marker.addListener("mouseout", () => {
            if (!element.visible) {
                APP.hide_edges(element);
            }
        });
        marker.addListener("contextmenu", () => {
            APP.tba_link(type, key);
        });
    } else {
        marker.addListener("click", () => {
            element.visible = !element.visible;
            if (element.visible) {
                APP.show_edges(element);
            } else {
                APP.hide_edges(element);
            }
        });
        marker.addListener("dblclick", () => {
            APP.tba_link(type, key);
        });
    }
}.bind(APP);

APP.show_edges = function (element) {
    element.edges.forEach((item) => {
        item.AddViewer();
        //item.setVisible(true);
    });
}.bind(APP);

APP.hide_edges = function (element) {
    element.edges.forEach((item) => {
        item.RemoveViewer();
        // item.setVisible(false);
    });
}.bind(APP);

function AddViewer() {
    this.viewer_count++;
    //console.log(this.name, this.viewer_count);
    this.setVisible(true);
}

function RemoveViewer() {
    this.viewer_count--;
    //console.log(this.name, this.viewer_count);
    if (this.viewer_count === 0) {
        this.setVisible(false);
    }
}

APP.toggle_markers = function (array, value) {
    array.forEach((element) => {
        element.setVisible(value);
    });
}.bind(APP);

APP.getMarker = function (rookie_year) {
    if (rookie_year === APP.year) {
        // #7C008F
        return "markers/rookie.png";
    }
    switch (true) {
        case rookie_year < 2000:
            return "markers/0.png"; // #0000FF
        case rookie_year < 2005:
            return "markers/1.png"; // #0033CC
        case rookie_year < 2010:
            return "markers/2.png"; // #006699
        case rookie_year < 2015:
            return "markers/3.png"; // #009966
        case rookie_year < 2020:
            return "markers/4.png"; // #00CC33
        case rookie_year < 2025:
            return "markers/5.png"; // #00FF00
    }
};

APP.init = async function () {
    APP.mobile =
        window.matchMedia("only screen and (max-width: 481px)").matches ||
        window.matchMedia("(pointer: coarse)").matches;

    console.log("LOADING: " + (APP.mobile ? "MOBILE" : "DESKTOP"));

    $(APP.mobile ? ".desktop" : ".mobile").hide();

    // Get snackbar object
    APP.snackbarContainer = document.querySelector("#search-toast");

    let query_data = parse_query();

    if (query_data.hasOwnProperty("year")) {
        let year = parseInt(query_data["year"]);
        if (!isNaN(year)) {
            APP.year = year;
        }
    }

    // Set Year in UI
    $(".year").text(APP.year.toString());

    document.title = `FRC Map ${APP.year.toString()}`;

    // Make legends
    $(".mini-box").each((index, obj) => {
        // console.log($(obj));
        $(obj).css({ "background-color": APP.legends[$(obj).attr("id")] });
    });

    APP.records.forEach((element) => {
        $("#years").append(
            `<li><a href="?year=${element}">${element}</a></li>`
        );
    });

    let data = await $.getJSON(`data/season_${APP.year}.json`, () => {});
    APP.data = data;

    let locations = await $.getJSON(
        "https://firstmap.github.io/data/custom_locations.json"
    );

    for (const key in locations) {
        if (locations.hasOwnProperty(key)) {
            const element = locations[key];
            let data_key = "frc" + String(key);
            if (data.teams.hasOwnProperty(data_key)) {
                data.teams[data_key].lat = element.lat;
                data.teams[data_key].lng = element.lng;
                // console.log("Updated: " + data_key);
            }
        }
    }

    // Make Events
    for (const key in data.events) {
        if (data.events.hasOwnProperty(key)) {
            const element = data.events[key];
            element.visible = false;
            element.edges = [];

            if (element.ignore === true) {
                console.log(`Ignoring event: ${key}`);
                continue;
            }
            let marker_icon = APP.markers.red;
            let title = `${element.name} (week ${element.week + 1})`;

            if (element.is_cmp) {
                marker_icon = APP.markers.first;
                title = `${element.name} (${element.start_date})`;
            }

            var marker = new google.maps.Marker({
                position: {
                    lat: Number(element.lat),
                    lng: Number(element.lng),
                },
                icon: marker_icon,
                map: this.map,
                title: title,
            });

            // Make sure championships appear on top!
            if (element.is_cmp) {
                marker.setZIndex(google.maps.Marker.MAX_ZINDEX);
                this.champ_markers.push(marker);
            } else {
                this.event_markers.push(marker);
            }

            APP.create_marker_listeners(marker, element, "event", key);
        }
    }

    APP.team_autocomplete = [];
    // Make Teams
    for (const key in data.teams) {
        if (data.teams.hasOwnProperty(key)) {
            const element = data.teams[key];
            if (element.ignore === true) {
                console.log(`Ignoring team: ${key}`);
                continue;
            }

            APP.team_autocomplete.push({
                value: key,
                label: `${element.team_number} | ${element.nickname}`,
            });
            element.visible = false;
            element.edges = [];
            var marker = new google.maps.Marker({
                position: {
                    lat: Number(element.lat),
                    lng: Number(element.lng),
                },
                icon: APP.getMarker(element.rookie_year),
                map: this.map,
                title: `${element.nickname} (${element.team_number})`,
            });

            element.marker = marker;
            this.team_markers.push(marker);
            APP.create_marker_listeners(
                marker,
                element,
                "team",
                element.team_number
            );
        }
    }

    // Make Lines
    for (const key in data.teams) {
        if (data.teams.hasOwnProperty(key)) {
            const element = data.teams[key];
            if (!element.hasOwnProperty("events")) {
                continue;
            }
            for (const event of element.events) {
                var path = [
                    {
                        lat: Number(element.lat),
                        lng: Number(element.lng),
                    },
                    {
                        lat: Number(data.events[event].lat),
                        lng: Number(data.events[event].lng),
                    },
                ];
                var len = google.maps.geometry.spherical.computeLength(path);
                var edge = new google.maps.Polyline({
                    path: path,
                    geodesic: false,
                    strokeColor: getColor(0, 10000000, len),
                    strokeOpacity: 1.0,
                    strokeWeight: 1,
                    map: APP.map,
                    clickable: false,
                    visible: false,
                });
                // This is a custom thing
                edge.name = `${key}-${event}`;
                edge.viewer_count = 0;
                edge.AddViewer = AddViewer;
                edge.RemoveViewer = RemoveViewer;
                // Add to event & team
                data.events[event].edges.push(edge);
                element.edges.push(edge);
            }
        }
    }

    // Get toggles
    this.toggles = {
        teams: $("#switch-teams"),
        events: $("#switch-regionals"),
        champs: $("#switch-championships"),
        tba: $("#switch-tba"),
    };

    // Team toggle listener
    this.toggles.teams.on("change", function () {
        console.log("Toggling team markers!");
        APP.toggle_markers(APP.team_markers, this.checked);
    });
    // Event toggle listener
    this.toggles.events.on("change", function () {
        console.log("Toggling event markers!");
        APP.toggle_markers(APP.event_markers, this.checked);
    });
    // Championship toggle listener
    this.toggles.champs.on("change", function () {
        console.log("Toggling champ markers!");
        APP.toggle_markers(APP.champ_markers, this.checked);
    });

    this.goto = function (key) {
        if (APP.data.teams.hasOwnProperty(key)) {
            APP.map.setCenter(APP.data.teams[key].marker.position);
            APP.map.setZoom(14);
            var data = {
                message: `${APP.data.teams[key].team_number} | ${APP.data.teams[key].nickname}`,
            };
            APP.snackbarContainer.MaterialSnackbar.showSnackbar(data);
        } else {
            var data = { message: "Could not find team " + key.substring(3) };
            APP.snackbarContainer.MaterialSnackbar.showSnackbar(data);
        }
    };

    // Search listener
    $("#search-bar").on("submit", (e) => {
        e.preventDefault();
        let number = $("#search-field").val();
        let key = "frc" + number;
        APP.goto(key);
    });
    $("#search-field").autocomplete({
        // source: APP.team_autocomplete,
        source: function (request, response) {
            var results = $.ui.autocomplete.filter(
                APP.team_autocomplete,
                request.term
            );
            response(results.slice(0, 10));
        },
        focus: function (event, ui) {
            $("#search-field").val(ui.item.label);
            return false;
        },
        select: (event, ui) => {
            event.preventDefault();
            $("#search-field").val(ui.item.label);
            // $("#search-field").val("");
            APP.goto(ui.item.value);
            return false;
        },
    });
    $(".ui-menu").addClass("mdl-card mdl-shadow--2dp");
}.bind(APP);

function parse_query() {
    var query = window.location.search.substring(1);
    var vars = query.split("&");
    let data = {};
    for (var i = 0; i < vars.length; i++) {
        var pair = vars[i].split("=");
        data[decodeURIComponent(pair[0])] = decodeURIComponent(pair[1]);
    }
    return data;
}
