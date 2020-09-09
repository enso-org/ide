//! Example of the visualization JS wrapper API usage
// TODO remove once we have proper visualizations or replace with a nice d3 example.
// These implementations are neither efficient nor pretty, but get the idea across.

use crate::data;
use crate::component::visualization;

/// Return a `JavaScript` Bubble visualization.
pub fn bubble_visualization() -> visualization::java_script::FallibleDefinition {
    let source = r#"
        class BubbleVisualization extends Visualization {
            static inputType = "Any"

            onDataReceived(data) {
                this.setPreprocessor("None");

                const xmlns = "http://www.w3.org/2000/svg";
                while (this.dom.firstChild) {
                    this.dom.removeChild(this.dom.lastChild);
                }
                const width = this.dom.getAttributeNS(null, "width");
                const height = this.dom.getAttributeNS(null, "height");

                const svgElem = document.createElementNS(xmlns, "svg");
                svgElem.setAttributeNS(null, "class"  , "vis-svg");
                svgElem.setAttributeNS(null, "viewBox", 0 + " " + 0 + " " + width + " " + height);
                svgElem.setAttributeNS(null, "width"  , "100%");
                svgElem.setAttributeNS(null, "height" , "100%");
                svgElem.setAttributeNS(null, "transform", "matrix(1 0 0 -1 0 0)");

                this.dom.appendChild(svgElem);

                data.forEach(data => {
                    const bubble = document.createElementNS(xmlns,"circle");
                    bubble.setAttributeNS(null,"stroke", "black");
                    bubble.setAttributeNS(null,"fill"  , "red");
                    bubble.setAttributeNS(null,"r"     , data[2]);
                    bubble.setAttributeNS(null,"cx"    , data[0]);
                    bubble.setAttributeNS(null,"cy"    , data[1]);
                    svgElem.appendChild(bubble);
                });
            }

            setSize(size) {
                this.dom.setAttributeNS(null, "width", size[0]);
                this.dom.setAttributeNS(null, "height", size[1]);
            }
        }

        return BubbleVisualization;
    "#;

    visualization::java_script::Definition::new(data::builtin_library(),source)
}

/// Return an empty minimal `JavaScript` visualization. This should not be used except for testing.
pub fn empty_visualization() -> visualization::java_script::FallibleDefinition {
    let source = r#"
        class EmptyVisualization extends Visualization {}
        return EmptyVisualization;
    "#;

    visualization::java_script::Definition::new(data::builtin_library(),source)
}

// TODO: Define scatter plot with d3.
/// Return a `JavaScript` Bubble visualization.
pub fn scatter_plot() -> visualization::java_script::FallibleDefinition {
    let source = r#"
        function loadScript(url) {
             var script = document.createElement("script");
             script.src = url;

            document.head.appendChild(script);
        }

        loadScript('https://d3js.org/d3.v4.min.js');

        class ScatterPlot extends Visualization {
            static inputType = "Any"

            onDataReceived(data) {
                this.setPreprocessor("None");

                const xmlns = "http://www.w3.org/2000/svg";
                while (this.dom.firstChild) {
                    this.dom.removeChild(this.dom.lastChild);
                }
                const width = this.dom.getAttributeNS(null, "width");
                const height = this.dom.getAttributeNS(null, "height");

                const svgElem = document.createElementNS(xmlns, "svg");
                svgElem.setAttributeNS(null, "class"  , "vis-svg");
                svgElem.setAttributeNS(null, "viewBox", 0 + " " + 0 + " " + width + " " + height);
                svgElem.setAttributeNS(null, "width"  , "100%");
                svgElem.setAttributeNS(null, "height" , "100%");
                svgElem.setAttributeNS(null, "transform", "matrix(1 0 0 -1 0 0)");

                // TODO: Remove this.
                svgElem.onmousedown = function() {
                    console.log("Clicked");
                }

                this.dom.appendChild(svgElem);

                data.forEach(data => {
                    const bubble = document.createElementNS(xmlns,"circle");
                    bubble.setAttributeNS(null,"stroke", "red");
                    bubble.setAttributeNS(null,"fill"  , "white");
                    bubble.setAttributeNS(null,"r"     , data[2]);
                    bubble.setAttributeNS(null,"cx"    , data[0]);
                    bubble.setAttributeNS(null,"cy"    , data[1]);
                    svgElem.appendChild(bubble);
                });
            }

            setSize(size) {
                this.dom.setAttributeNS(null, "width", size[0]);
                this.dom.setAttributeNS(null, "height", size[1]);
            }
        }

        return ScatterPlot;
    "#;

    visualization::java_script::Definition::new(data::builtin_library(),source)
}
