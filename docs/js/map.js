
let APP = {
    markers: {
        red: "https://franspaco.com/resources/red_marker.png",
        green: "https://franspaco.com/resources/green_marker.png"
    },
    team_markers: [],
    event_markers: [],
    year: 2019,
    legends: {
        l_rookie: '#7C008F',
        l_0: '#0000FF',
        l_1: '#0033CC',
        l_2: '#006699',
        l_3: '#009966',
        l_4: '#00CC33',
        l_5: '#00FF00',
        l_event: '#FF0000',
    },
    infowindow: null,
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

APP.getMarker = function(rookie_year){
    if(rookie_year === APP.year){
        // #7C008F 
        return 'markers/rookie.png'
    }
    switch(true){
        case (rookie_year < 2000):
            return 'markers/0.png'; // #0000FF
        case (rookie_year < 2005):
            return 'markers/1.png'; // #0033CC
        case (rookie_year < 2010):
            return 'markers/2.png'; // #006699
        case (rookie_year < 2015):
            return 'markers/3.png'; // #009966
        case (rookie_year < 2020):
            return 'markers/4.png'; // #00CC33
        case (rookie_year < 2025):
            return 'markers/5.png'; // #00FF00
    }
}

APP.makeEventInfoWindow = function(data){
    this.infowindow.close();
    this.infowindow.setContent(`
        <div class="infowindow">
            <h3>${data.name}</h3>
            <p>Week: ${data.week}</p>
        </div>
    `);
    this.infowindow.open(this.map, data.marker);
}.bind(APP);

APP.makeTeamInfoWindow = function(data){
    this.infowindow.close();
    this.infowindow.open(this.map, data.marker);
}.bind(APP);

APP.init = async function(){

    this.infowindow = new google.maps.InfoWindow();

    // Set Year in UI
    $('.year').text(APP.year.toString());

    // Make legends
    $('.mini-box').each((index, obj) => {
        $(obj).css({'background-color': APP.legends[$(obj).attr('id')]});
    });

    let data = await $.getJSON("data/season_2019.json", () => {});
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
            element.marker = marker;
            this.event_markers.push(marker);
            marker.addListener("click", (event) => {
                APP.tba_event(key);
            });
            marker.addListener("rightclick", (event) => {
                APP.makeEventInfoWindow(element);
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
                icon: APP.getMarker(element.rookie_year),
                map: this.map,
                title: `${element.nickname} (${element.team_number})`
            });
            element.marker = marker;
            this.team_markers.push(marker);
            marker.addListener("click", () => {
                APP.tba_team(element.team_number);
            });
            marker.addListener("rightclick", (event) => {
                APP.makeTeamInfoWindow(element);
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
            $(marker).mouseenter(()=>{
                console.log('ENTER');
            });
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