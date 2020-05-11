//! Example of the visualisation JS wrapper API usage

use crate::component::visualization::JsRenderer;

/// Returns a simple bubble chart implemented in vanilla JS.
// TODO remove once we have proper visualizations or replace with a nice d3 example.
// This implementation is neither efficient nor pretty, but it gets the idea across.
pub fn sample_js_bubble_chart() -> JsRenderer {
    let fn_set_data = r#"{
        const xmlns = "http://www.w3.org/2000/svg";
        const root = arguments[0];
        while (root.firstChild) {
             root.removeChild(root.lastChild);
        }

        const svgElem = document.createElementNS(xmlns, "svg");
        svgElem.setAttributeNS(null, "id"     , "vis-svg");
        svgElem.setAttributeNS(null, "viewBox", "0 0 " + 100 + " " + 100);
        svgElem.setAttributeNS(null, "width"  , 100);
        svgElem.setAttributeNS(null, "height" , 100);
        root.appendChild(svgElem);

        const data = arguments[1];
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
    "#;

    let fn_set_size = r#"{
        const root    = arguments[0];
        const width   = arguments[1][0];
        const height  = arguments[1][1];
        const svgElem = root.firstChild;
        svgElem.setAttributeNS(null, "viewBox", "0 0 " + width + " " + height);
        svgElem.setAttributeNS(null, "width"  , width);
        svgElem.setAttributeNS(null, "height" , height);
    }"#;
    JsRenderer::from_functions(fn_set_data, fn_set_size)
}

pub fn object_sample_js_bubble_chart() -> JsRenderer {
    let fn_prototype = r#"
    (() => {
        const obj = new Object();
        obj.set_data = (root, data) => {
            const xmlns = "http://www.w3.org/2000/svg";
            while (root.firstChild) {
                root.removeChild(root.lastChild);
            }

            const svgElem = document.createElementNS(xmlns, "svg");
            svgElem.setAttributeNS(null, "id"     , "vis-svg");
            svgElem.setAttributeNS(null, "viewBox", "0 0 " + 100 + " " + 100);
            svgElem.setAttributeNS(null, "width"  , 100);
            svgElem.setAttributeNS(null, "height" , 100);
            root.appendChild(svgElem);

            data.forEach(data => {
                const bubble = document.createElementNS(xmlns,"circle");
                bubble.setAttributeNS(null,"stroke", "black");
                bubble.setAttributeNS(null,"fill"  , "red");
                bubble.setAttributeNS(null,"r"     , data[2]);
                bubble.setAttributeNS(null,"cx"    , data[0]);
                bubble.setAttributeNS(null,"cy"    , data[1]);
                svgElem.appendChild(bubble);
            });
        };

        obj.set_size = (root, size) => {
            const width   = size[0];
            const height  = size[1];
            const svgElem = root.firstChild;
            svgElem.setAttributeNS(null, "viewBox", "0 0 " + width + " " + height);
            svgElem.setAttributeNS(null, "width"  , width);
            svgElem.setAttributeNS(null, "height" , height);
        };

         return obj;
    })()
    "#;
    JsRenderer::from_source(fn_prototype).unwrap()
}
