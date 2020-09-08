function loadScript(url) {
    var script = document.createElement("script");
    script.src = url;

    document.head.appendChild(script);
}

function loadStyle(url) {
    var link  = document.createElement("link");
    link.href = url;
    link.rel  = 'stylesheet';

    document.head.appendChild(link);
}

loadScript('https://unpkg.com/deck.gl@latest/dist.min.js');
loadScript('https://d3js.org/d3.v5.min.js');
loadScript('https://api.tiles.mapbox.com/mapbox-gl-js/v1.6.1/mapbox-gl.js');
loadStyle('https://api.tiles.mapbox.com/mapbox-gl-js/v1.6.1/mapbox-gl.css');



class MapViewVisualization extends Visualization {
    static inputType = "Any"

    onDataReceived(data) {
        this.setPreprocessor("None");

        while (this.dom.firstChild) {
            this.dom.removeChild(this.dom.lastChild);
        }

        const width = this.dom.getAttributeNS(null, "width");
        const height = this.dom.getAttributeNS(null, "height");

        const mapElem = document.createElement("div");
        mapElem.setAttributeNS(null,"id"       , "map");
        mapElem.setAttributeNS(null,"style"    ,"width:" + width + "px;height: " + height + "px;");
        this.dom.appendChild(mapElem);

        const parsedData = JSON.parse(data);

        const deckgl = new deck.DeckGL({
            container: 'map',
            mapboxApiAccessToken: 'pk.eyJ1IjoidWJlcmRhdGEiLCJhIjoiY2pudzRtaWloMDAzcTN2bzN1aXdxZHB5bSJ9.2bkj3IiRC8wj3jLThvDGdA',
            mapStyle: parsedData.mapStyle || 'mapbox://styles/mapbox/dark-v9',
            initialViewState: {
                longitude: parsedData.longitude || -1.4157,
                latitude: parsedData.latitude || 52.2324,
                zoom: parsedData.zoom || 3,
                pitch: parsedData.pitch || 0
            },
            controller: parsedData.controller || false
        });

        const layer = new deck.ScatterplotLayer({
            id: 'scatterplot',
            data: parsedData.data || [],
            getColor: d => d.color,
            getRadius: d => d.radius,
            opacity: 1
        });

        deckgl.setProps({
            layers: [layer]
        });
    }

    setSize(size) {
        this.dom.setAttributeNS(null, "width", size[0]);
        this.dom.setAttributeNS(null, "height", size[1]);
    }
}

return MapViewVisualization;
