//! Map view visualization.
use crate::data;
use crate::component::visualization;

/// Return a `JavaScript` Map visualization.
pub fn map_view_visualization() -> visualization::java_script::FallibleDefinition {
    let source     = r#"
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
        loadScript('https://d3js.org/d3.v6.min.js');
        loadScript('https://api.tiles.mapbox.com/mapbox-gl-js/v0.53.0/mapbox-gl.js');
        loadStyle('https://api.tiles.mapbox.com/mapbox-gl-js/v0.53.0/mapbox-gl.css');

        return class MapViewVisualization extends Visualization {
            static inputType = "Any"

            onDataReceived(data) {
                this.setPreprocessor("None");

                while (this.dom.firstChild) {
                    this.dom.removeChild(this.dom.lastChild);
                }

                const width = this.dom.getAttributeNS(null, "width");
                const height = this.dom.getAttributeNS(null, "height");

                const mapElem = document.createElement("div");
                mapElem.setAttributeNS(null,"id"       , "vis-map");
                mapElem.setAttributeNS(null,"viewBox"  , 0 + " " + 0 + " " + width + " " + height);
                mapElem.setAttributeNS(null,"width"    , "100%");
                mapElem.setAttributeNS(null,"height"   , "100%");
                mapElem.setAttributeNS(null,"transform", "matrix(1 0 0 -1 0 0)");
                mapElem.setAttributeNS(null,"style"    ,"width:" + width + "px;height: " + height + "px;overflow: scroll;border-radius:14px");
                this.dom.appendChild(mapElem);

                const inner = `<div id="map"></div>`;
                mapElem.innerHTML = inner;

                const deckgl = new deck.DeckGL({
                  mapboxApiAccessToken: 'pk.eyJ1IjoiZ28tZmluZCIsImEiOiJjazBod3EwZnAwNnA3M2JydHcweTZiamY1In0.U5O7_hDFJ-1RpA8L9zUmTQ',
                  mapStyle: 'mapbox://styles/mapbox/dark-v9',
                  container: 'map',
                  initialViewState: {
                    longitude: -74,
                    latitude: 40.76,
                    zoom: 11,
                    minZoom: 5,
                    maxZoom: 16,
                    pitch: 40.5
                  },
                  controller: true
                });

                const dataaaa = d3.csv('https://raw.githubusercontent.com/uber-common/deck.gl-data/master/examples/3d-heatmap/heatmap-data.csv');

                const COLOR_RANGE = [
                  [1, 152, 189],
                  [73, 227, 206],
                  [216, 254, 181],
                  [254, 237, 177],
                  [254, 173, 84],
                  [209, 55, 78]
                ];

                renderLayer();

                function renderLayer () {
                  const options = {
                    radius : 1000,
                    coverage : 1,
                    upperPercentile : 100
                  };

                  const hexagonLayer = new deck.HexagonLayer({
                    id: 'heatmap',
                    colorRange: COLOR_RANGE,
                    data: 'https://raw.githubusercontent.com/visgl/deck.gl-data/master/examples/scatterplot/manhattan.json',
                    elevationRange: [0, 1000],
                    elevationScale: 250,
                    extruded: true,
                    getPosition: d => [Number(d.lng), Number(d.lat)],
                    opacity: 1,
                    ...options
                  });

                  deckgl.setProps({
                    layers: [hexagonLayer]
                  });
                }


                // data.forEach(data => {
                //     ...
                // });

                var mapInnerDiv = document.getElementById("map");
                mapInnerDiv.style.position = "inherit";
            }

            setSize(size) {
                this.dom.setAttributeNS(null, "width", size[0]);
                this.dom.setAttributeNS(null, "height", size[1]);
            }
        }
    "#;

    println!("{}",source);

    visualization::java_script::Definition::new(data::builtin_library(),source)
}