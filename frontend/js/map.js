
let APP = {
    markers: {
        red: "https://franspaco.com/resources/red_marker.png",
        green: "https://franspaco.com/resources/green_marker.png"
    },
    team_markers: [],
    event_markers: [],
};

async function initMap() {
    var center = {
        lat: 26.449372,
        lng: -99.131382
    };
    APP.map = new google.maps.Map(document.getElementById('map'), {
        zoom: 4,
        center: center
    });
    APP.init();
}

APP.tba_team = function(team){
    window.open("https://www.thebluealliance.com/team/" + team, '_blank');
}

APP.tba_event = function(event){
    window.open("https://www.thebluealliance.com/event/" + event, '_blank');
}


APP.init = async function(){

    let data = await $.getJSON("/data/season_2019.json", () => {});
    APP.data = data;
    // Make Regionals
    for (const key in data.events) {
        if (data.events.hasOwnProperty(key)) {
            const element = data.events[key];
            element.edges = [];
            element.edges_visible = false;
            var marker = new google.maps.Marker({
                position: {
                    lat: Number(element.lat),
                    lng: Number(element.lng)
                },
                icon: APP.markers.red,
                map: this.map,
                title: `${element.name} (${element.week})`
            });
            this.event_markers.push(marker);
            marker.addListener("click", () => {
                APP.tba_event(key);
            });
            marker.addListener("mouseover", () => {
                element.edges.forEach(item => {
                    item.setVisible(true);
                });
            });
            marker.addListener("mouseout", () => {
                element.edges.forEach(item => {
                    item.setVisible(false);
                });
            });
        }
    }

    // Make Teams
    for (const key in data.teams) {
        if (data.teams.hasOwnProperty(key)) {
            const element = data.teams[key];
            element.edges = [];
            var marker = new google.maps.Marker({
                position: {
                    lat: Number(element.lat),
                    lng: Number(element.lng)
                },
                icon: APP.markers.green,
                map: this.map,
                title: `${element.nickname} (${element.team_number})`
            });
            marker.addListener("click", () => {
                APP.tba_team(element.team_number);
            });
            marker.addListener("mouseover", () => {
                element.edges.forEach(item => {
                    item.setVisible(true);
                });
                console.log('WOOP');
            });
            marker.addListener("mouseout", () => {
                element.edges.forEach(item => {
                    item.setVisible(false);
                });
            });
            $(marker).mouseenter(()=>{
                console.log('ENTER');
            });
            element.marker = marker;
            this.team_markers.push(marker);
        }
    }

    // Make Lines
    for (const key in data.teams) {
        if (data.teams.hasOwnProperty(key)) {
            const element = data.teams[key];
            if(!element.hasOwnProperty('events')){
                continue;
            }
            for (const event of element.events) {
                var edge = new google.maps.Polyline({
                    path: [
                        {
                            lat: Number(element.lat),
                            lng: Number(element.lng)
                        },
                        {
                            lat: Number(data.events[event].lat),
                            lng: Number(data.events[event].lng)
                        }
                    ],
                    geodesic: false,
                    strokeColor: '#FF0000',
                    strokeOpacity: 1.0,
                    strokeWeight: 1,
                    map: APP.map,
                });
                edge.setVisible(false);
                data.events[event].edges.push(edge);
                element.edges.push(edge);
            }
        }
    }


    // Get toggles
    this.toggles = {
        teams: $('#switch-teams'),
        events: $('#switch-regionals'),
        tba: $('#switch-tba')
    }

    this.toggles.teams.change(function() {
        APP.toggle_markers(APP.team_markers, this.checked);
    });
    this.toggles.events.change(function() {
        APP.toggle_markers(APP.event_markers, this.checked);
    });
}.bind(APP);


APP.toggle_markers = function(array, value){
    array.forEach(element => {
        element.setVisible(value);
    });
}.bind(APP);