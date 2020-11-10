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
loadScript('https://api.tiles.mapbox.com/mapbox-gl-js/v1.6.1/mapbox-gl.js');
loadStyle('https://api.tiles.mapbox.com/mapbox-gl-js/v1.6.1/mapbox-gl.css');

const styleHead = document.createElement("style")
styleHead.innerText = `.mapboxgl-map {
            border-radius: 14px;
        }`
document.head.appendChild(styleHead);

const TOKEN = 'pk.eyJ1IjoiZW5zby1vcmciLCJhIjoiY2tmNnh5MXh2MGlyOTJ5cWdubnFxbXo4ZSJ9.3KdAcCiiXJcSM18nwk09-Q';

const GEO_POINT         = "GeoPoint";
const GEO_MAP           = "GeoMap";
const SCATTERPLOT_LAYER = "ScatterplotLayer";
const DEFAULT_RADIUS    = 150;


/**
 * Provides a mapbox & deck.gl-based map visualization for IDE.
 *
 * > Example creates a map with described properties with a scatter plot overlay:
 * {
 * "type": "GeoMap",
 * "latitude": 37.8,
 * "longitude": -122.45,
 * "zoom": 15,
 * "controller": true,
 * "layers": [{
 *     "type": "ScatterplotLayer",
 *     "data": [{
 *         "type": "GeoPoint",
 *         "latitude": -122.45,
 *         "longitude": 37.8,
 *         "color": [255, 0, 0],
 *         "radius": 100
 *     }]
 * }]
 * }
 */
class MapViewVisualization extends Visualization {
    static inputType = "Any"

    onDataReceived(data) {
        while (this.dom.firstChild) {
            this.dom.removeChild(this.dom.lastChild);
        }

        const width   = this.dom.getAttributeNS(null, "width");
        const height  = this.dom.getAttributeNS(null, "height");
        const mapElem = document.createElement("div");
        mapElem.setAttributeNS(null,"id"   , "map");
        mapElem.setAttributeNS(null,"style","width:" + width + "px;height: " + height + "px;");
        this.dom.appendChild(mapElem);

        let parsedData = data;
        if (typeof data === "string") {
            parsedData = JSON.parse(data);
        }

        let defaultMapStyle = 'mapbox://styles/mapbox/light-v9';
        let accentColor     = [1,234,146];
        if (document.getElementById("root").classList.contains("dark")){
            defaultMapStyle = 'mapbox://styles/mapbox/dark-v9';
            accentColor     = [222,162,47];
        }

        let preparedDataPoints = []
        let computed           = this.prepareDataPoints(parsedData,preparedDataPoints,accentColor);

        const scatterplotLayer = new deck.ScatterplotLayer({
            data: preparedDataPoints,
            getFillColor: d => d.color,
            getRadius: d => d.radius
        })
        //
        let latitudeMatch  = parsedData.latitude !== undefined && parsedData.latitude !== null;
        let latitude       = latitudeMatch ? parsedData.latitude : computed.latitude;
        let longitudeMatch = parsedData.longitude !== undefined && parsedData.longitude !== null;
        let longitude      = longitudeMatch ? parsedData.longitude : computed.longitude;
        // TODO : Compute zoom somehow.
        let zoomMatch      = parsedData.zoom !== undefined && parsedData.zoom !== null;
        let zoom           = zoomMatch ? parsedData.zoom : computed.zoom;

        const deckgl = new deck.DeckGL({
            container: 'map',
            mapboxApiAccessToken: TOKEN,
            mapStyle: parsedData.mapStyle || defaultMapStyle,
            initialViewState: {
                longitude: longitude,
                latitude: latitude,
                zoom: zoom,
                pitch: parsedData.pitch || 0
            },
            controller: parsedData.controller || true
        });

        deckgl.setProps({
            layers: [scatterplotLayer]
        });
    }

    /**
     * Prepares data points to be shown on the map.
     * @param preparedDataPoints - List holding geoPoints.
     * @param parsedData - All the parsed data to create GeoPoints from.
     * @param accentColor - accent color of IDE if element doesn't specify one.
     */
    prepareDataPoints(parsedData, preparedDataPoints, accentColor) {
        let latitude  = 0.0;
        let longitude = 0.0;
        let zoom      = 11;

        if (parsedData.type === GEO_POINT) {
            this.pushGeoPoint(preparedDataPoints,parsedData,accentColor);
            latitude  = parsedData.latitude;
            longitude = parsedData.longitude;
        } else if (Array.isArray(parsedData) && parsedData.length && parsedData[0].type === GEO_POINT) {
            const computed = this.prepareDataPointsHelper(parsedData,preparedDataPoints,accentColor);
            latitude       = computed.latitude;
            longitude      = computed.longitude;
        } else {
            if (parsedData.type === SCATTERPLOT_LAYER && parsedData.data.length) {
                const computed = this.prepareDataPointsHelper(parsedData.data,preparedDataPoints,accentColor);
                latitude       = computed.latitude;
                longitude      = computed.longitude;
            } else if (parsedData.type === GEO_MAP && parsedData.layers !== undefined) {
                parsedData.layers.forEach(layer => {
                    if (layer.type === SCATTERPLOT_LAYER) {
                        let dataPoints = layer.data || [];
                        const computed = this.prepareDataPointsHelper(dataPoints,preparedDataPoints,accentColor);
                        latitude       = computed.latitude;
                        longitude      = computed.longitude;
                    } else {
                        console.log("Currently unsupported deck.gl layer.")
                    }
                })
            }
        }
        return {latitude,longitude,zoom}
    }

    /**
     * Helper for prepareDataPoints, calculating also central point.
     * @returns {{latitude: number, longitude: number}} - center.
     */
    prepareDataPointsHelper(dataPoints,preparedDataPoints,accentColor) {
        let latitudes = [];
        let longitudes = [];
        dataPoints.forEach(e => {
            this.pushGeoPoint(preparedDataPoints, e, accentColor);
            latitudes.push(e.latitude);
            longitudes.push(e.longitude);
        });
        let latitude = 0.0;
        let longitude = 0.0;
        if (latitudes.length && longitudes.length) {
            latitude = latitudes.reduce((a, b) => a + b) / latitudes.length;
            longitude = longitudes.reduce((a, b) => a + b) / longitudes.length;
        }
        return {latitude,longitude};
    }

    /**
     * Pushes a new deck.gl-compatible point from GeoPoint.
     * @param preparedDataPoints - List holding geoPoints.
     * @param element - element to create new deck.gl point from.
     * @param accentColor - accent color of IDE if element doesn't specify one.
     */
    pushGeoPoint(preparedDataPoints,element,accentColor) {
        let radius = isNaN(element.radius) ? DEFAULT_RADIUS : element.radius;
        preparedDataPoints.push({
            position: [element.longitude,element.latitude],
            color: element.color || accentColor,
            radius: radius
        });
    }

    /**
     * Sets size of the visualization.
     * @param size - new size.
     */
    setSize(size) {
        this.dom.setAttributeNS(null, "width", size[0]);
        this.dom.setAttributeNS(null, "height", size[1]);
    }
}

return MapViewVisualization;
