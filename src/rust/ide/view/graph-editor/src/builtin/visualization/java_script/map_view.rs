//! Map view visualization.
use crate::data;
use crate::component::visualization;

/// Return a `JavaScript` Map visualization.
pub fn map_view_visualization() -> visualization::java_script::FallibleDefinition {
    let deck       = include_str!("map_view/deck.js");
    let mapbox_js  = include_str!("map_view/mapbox.js");
    let mapbox_css = include_str!("map_view/mapbox.css");
    let mapbox_css = format!("<style>{}</style>", mapbox_css);
    let mapbox_css = format!("const mapbox_css = `{}`", mapbox_css);
    let source     = r#"
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
                mapElem.setAttributeNS(null,"style"    ,"width:" + width + "px;height: " + height + "px;overflow: scroll;position: relative;border-radius:14px");
                this.dom.appendChild(mapElem);

                const inner = mapbox_css + `<div id="map"></div>`;
                mapElem.innerHTML = inner;

                const {MapboxLayer, ScatterplotLayer} = deck;

                // Get a mapbox API access token
                mapboxgl.accessToken = 'pk.eyJ1IjoiZ28tZmluZCIsImEiOiJjazBod3EwZnAwNnA3M2JydHcweTZiamY1In0.U5O7_hDFJ-1RpA8L9zUmTQ';

                // Initialize mapbox map
                const map = new mapboxgl.Map({
                  container: 'map',
                  style: 'mapbox://styles/mapbox/dark-v9',
                  center: [19.94, 50.04],
                  zoom: 8
                });

                // Create a mapbox-compatible deck.gl layer
                const myDeckLayer = new MapboxLayer({
                  id: 'my-scatterplot',
                  type: ScatterplotLayer,
                  data: [
                    {position: [19.94, 50.04], color: [255, 0, 0], radius: 10}
                  ],
                  getPosition: d => d.position,
                  getRadius: d => d.radius,
                  getColor: d => d.color
                });

                // Insert the layer before mapbox labels
                map.on('load', () => {
                  map.addLayer(myDeckLayer, 'waterway-label');
                });

                // const INITIAL_VIEW_STATE = {
                //     latitude: 37.8,
                //     longitude: -122.45,
                //     zoom: 15
                //   };
                //
                // const deckgl = new deck.DeckGL({
                //     canvas: 'deckgl',
                //     width: width,
                //     height: height,
                //     mapboxApiAccessToken: 'pk.eyJ1IjoiZ28tZmluZCIsImEiOiJjazBod3EwZnAwNnA3M2JydHcweTZiamY1In0.U5O7_hDFJ-1RpA8L9zUmTQ',
                //     mapStyle: 'mapbox://styles/mapbox/light-v9',
                //     initialViewState: {
                //       longitude: -122.45,
                //       latitude: 37.8,
                //       zoom: 15
                //     },
                //     controller: true,
                //     layers: [
                //       new deck.ScatterplotLayer({
                //         data: [
                //           {position: [-122.45, 37.8], color: [255, 0, 0], radius: 100}
                //         ],
                //         getColor: d => d.color,
                //         getRadius: d => d.radius
                //       })
                //     ]
                //   });

                // mapElem.appendChild(deckgl);
                // data.forEach(data => {
                //     const bubble = document.createElementNS(xmlns,"circle");
                //     bubble.setAttributeNS(null,"stroke", "black");
                //     bubble.setAttributeNS(null,"fill"  , "yellow");
                //     bubble.setAttributeNS(null,"r"     , data[2]);
                //     bubble.setAttributeNS(null,"cx"    , data[0]);
                //     bubble.setAttributeNS(null,"cy"    , data[1]);
                //     mapElem.appendChild(bubble);
                // });
            }

            setSize(size) {
                this.dom.setAttributeNS(null, "width", size[0]);
                this.dom.setAttributeNS(null, "height", size[1]);
            }
        }
    "#;

    let source = format!("{}\n{}\n{}\n{}\n",deck,mapbox_js,mapbox_css,source);
    println!("{}",source);

    visualization::java_script::Definition::new(data::builtin_library(),source)
}