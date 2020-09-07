//! Map view visualization.
use crate::data;
use crate::component::visualization;

/// Return a `JavaScript` Map visualization.
pub fn map_view_visualization() -> visualization::java_script::FallibleDefinition {
    let source = r#"
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
                mapElem.setAttributeNS(null,"id"       , "map");
                mapElem.setAttributeNS(null,"style"    ,"width:" + width + "px;height: " + height + "px;");
                this.dom.appendChild(mapElem);

                const deckgl = new deck.DeckGL({
                  container: 'map',
                  mapboxApiAccessToken: 'pk.eyJ1IjoidWJlcmRhdGEiLCJhIjoiY2pudzRtaWloMDAzcTN2bzN1aXdxZHB5bSJ9.2bkj3IiRC8wj3jLThvDGdA',
                  mapStyle: 'mapbox://styles/mapbox/dark-v9',
                  initialViewState: {
                    longitude: -1.4157,
                    latitude: 52.2324,
                    zoom: 3,
                    minZoom: 5,
                    maxZoom: 15,
                    pitch: 40.5
                  },
                  controller: false
                });

                const dataaaa = d3.csv('https://raw.githubusercontent.com/uber-common/deck.gl-data/master/examples/3d-heatmap/heatmap-data.csv');

                console.log(data);

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
                  const hexagonLayer = new deck.HexagonLayer({
                    id: 'heatmap',
                    colorRange: COLOR_RANGE,
                    data: dataaaa,
                    elevationRange: [0, 1000],
                    elevationScale: 250,
                    extruded: true,
                    getPosition: d => [Number(d.lng), Number(d.lat)],
                    opacity: 1
                  });

                  deckgl.setProps({
                    layers: [hexagonLayer]
                  });
                }

                // data.forEach(data => {
                //     ...
                // });
            }

            setSize(size) {
                this.dom.setAttributeNS(null, "width", size[0]);
                this.dom.setAttributeNS(null, "height", size[1]);
            }
        }
    "#;
    visualization::java_script::Definition::new(data::builtin_library(),source)
}