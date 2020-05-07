//! Example of the visualisation JS wrapper API usage

use crate::component::visualization::JsRendererGeneric;

/// Returns a simple bubble chart implemented in vanilla JS.
// TODO remove once we have proper visualizations or replace with a nice d3 example.
pub fn sample_js_bubble_chart() -> JsRendererGeneric {
    let fn_set_data = r#"{
        var xmlns = "http://www.w3.org/2000/svg";
        const root = arguments[0];
        while (root.firstChild) {
             root.removeChild(root.lastChild);
        }

        var svgElem = document.createElementNS(xmlns, "svg");
        svgElem.setAttributeNS(null, "id", "vis-svg");
        svgElem.setAttributeNS(null, "viewBox", "0 0 " + 100 + " " + 100);
        svgElem.setAttributeNS(null, "width", 100);
        svgElem.setAttributeNS(null, "height", 100);
        root.appendChild(svgElem);

        const data = arguments[1];
        data.forEach(data => {
            const bubble = document.createElementNS(xmlns,"circle");
            bubble.setAttributeNS(null,"stroke","black");
            bubble.setAttributeNS(null,"fill","red");
            bubble.setAttributeNS(null,"r", data[2]);
            bubble.setAttributeNS(null,"cx",data[0]);
            bubble.setAttributeNS(null,"cy",data[1]);
            svgElem.appendChild(bubble);
        });
    }
    "#;

    let fn_set_size = r#"{
        const svg = document.getElementById("vis-svg");
        if (svg == null) {
            return;
        }
        const width  = arguments[0];
        const height = arguments[1];
        svgElem.setAttributeNS(null, "viewBox", "0 0 " + width + " " + height);
        svgElem.setAttributeNS(null, "width",   width);
        svgElem.setAttributeNS(null, "height",  height);
    }"#;
    JsRendererGeneric::new(fn_set_data, fn_set_size)
}
