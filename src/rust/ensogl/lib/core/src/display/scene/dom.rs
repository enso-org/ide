//! This module defines a DOM management utilities.

use crate::prelude::*;

use crate::display::object::traits::*;
use crate::display::camera::Camera2d;
use crate::display::camera::camera2d::Projection;
use crate::display::symbol::DomSymbol;
use crate::display::symbol::dom::eps;
use crate::display::symbol::dom::flip_y_axis;
use crate::system::gpu::data::JsBufferView;
use crate::system::web;
use crate::system::web::NodeInserter;
use crate::system::web::StyleSetter;

use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::HtmlDivElement;



// ====================
// === DomSceneData ===
// ====================

/// Internal representation for `DomScene`.
#[derive(Clone,Debug)]
pub struct DomSceneData {
    /// The root dom element of this scene.
    pub dom : HtmlDivElement,
    /// The child div of the `dom` element with view-projection Css 3D transformations applied.
    pub view_projection_dom : HtmlDivElement,
    logger : Logger
}

impl DomSceneData {
    /// Constructor.
    pub fn new(dom:HtmlDivElement, view_projection_dom:HtmlDivElement, logger:Logger) -> Self {
        Self {dom,view_projection_dom,logger}
    }
}



// ================
// === DomScene ===
// ================

/// `DomScene` is a renderer for `DomSymbol`s. It integrates with other rendering contexts,
/// such as WebGL, by placing two HtmlElements in front and behind of the Canvas element,
/// allowing the move `DomSymbol`s between these two layers, mimicking z-index ordering.
///
/// To make use of its functionalities, the API user can create a `Css3dSystem` by using
/// the `DomScene::new_system` method which creates and manages instances of
/// `DomSymbol`s.
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
pub struct DomScene {
    data : Rc<DomSceneData>,
}

impl DomScene {
    /// Constructor.
    pub fn new(logger:impl AnyLogger) -> Self {
        let logger              = Logger::sub(logger,"DomScene");
        let dom                 = web::create_div();
        let view_projection_dom = web::create_div();

        dom.set_class_name("dom-scene-layer");
        dom.set_style_or_warn("position"       , "absolute" , &logger);
        dom.set_style_or_warn("top"            , "0px"      , &logger);
        dom.set_style_or_warn("overflow"       , "hidden"   , &logger);
        dom.set_style_or_warn("overflow"       , "hidden"   , &logger);
        dom.set_style_or_warn("width"          , "100%"     , &logger);
        dom.set_style_or_warn("height"         , "100%"     , &logger);
        dom.set_style_or_warn("pointer-events" , "none"     , &logger);

        view_projection_dom.set_class_name("view_projection");
        view_projection_dom.set_style_or_warn("width"           , "100%"        , &logger);
        view_projection_dom.set_style_or_warn("height"          , "100%"        , &logger);
        view_projection_dom.set_style_or_warn("transform-style" , "preserve-3d" , &logger);

        dom.append_or_warn(&view_projection_dom,&logger);

        let data = DomSceneData::new (dom,view_projection_dom,logger);
        let data = Rc::new(data);
        Self {data}
    }

    /// Gets the number of children DomSymbols.
    pub fn children_number(&self) -> u32 {
        self.data.dom.children().length()
    }

    /// Sets the z-index of this DOM element.
    pub fn set_z_index(&self, z:i32) {
        self.data.dom.set_style_or_warn("z-index", z.to_string(), &self.logger);
    }

    /// Sets the CSS property `filter: grayscale({value})` on this element. A value of 0.0 displays
    /// the element normally. A value of 1.0 will make the element completely gray.
    pub fn filter_grayscale(&self, value:f32) {
        self.data.dom.set_style_or_warn("filter",format!("grayscale({})",value),&self.logger);
    }

    /// Creates a new instance of DomSymbol and adds it to parent.
    pub fn manage(&self, object:&DomSymbol) {
        let dom  = object.dom();
        let data = &self.data;
        if object.is_visible() {
            self.view_projection_dom.append_or_panic(dom);
        }
        object.display_object().set_on_hide(f_!(dom.remove()));
        object.display_object().set_on_show(f__!([data,dom] {
            data.view_projection_dom.append_or_panic(&dom)
        }));
    }

    /// Update the objects to match the new camera's point of view. This function should be called
    /// only after camera position change.
    pub fn update_view_projection(&self, camera:&Camera2d) {
        if self.children_number() == 0 { return }


        // === Reset configuration ===

        self.data.view_projection_dom.set_style_or_panic("transform", "");
        self.data.view_projection_dom.set_style_or_panic("left", "0px");
        self.data.view_projection_dom.set_style_or_panic("top", "0px");
        self.data.dom.set_style_or_panic("perspective", "");


        let view_matrix = camera.view_matrix();
        let view_matrix = view_matrix.map(eps);
        // In CSS, the y axis points downwards. (In EnsoGL upwards)
        let mut trans_cam = flip_y_axis(view_matrix);

        match camera.projection() {
            Projection::Perspective{..} => {
                // In order to achieve a better visual quality when the camera is in fron of the z=0
                // plane, we simulate translation in the z direction by scaling the scene around
                // the origin, as discussed at https://github.com/enso-org/ide/pull/1465.
                // In that situation, we also simulate x and y translation through `top` and `left`.

                let x_index = (0, 3);
                let y_index = (1, 3);
                let z_index = (2, 3);
                let target_x = *trans_cam.index(x_index);
                let target_y = *trans_cam.index(y_index);
                let target_z = *trans_cam.index(z_index);

                // Put the camera in front of the origin and adjust the field of view, such that one
                // px unit at z=0 maps to one px on screen.
                self.data.dom.set_style_or_panic("perspective", format!("{}px", camera.z_zoom_1()));
                // The position of the scene relative to the camera after setting "perspective".
                let current_z = -camera.z_zoom_1();

                let MAXIMUM_SCALE_FACTOR = 1000.0;
                // Our method to simulate translation through scaling works only when `target_z` is
                // less than 0. We cut off at some point before that and fall back to real 3D
                // transformations. We determine that point through the maximum scale factor that
                // that we want to allow. (See also the computation of `scale` below)
                if target_z <= current_z / MAXIMUM_SCALE_FACTOR {
                    *trans_cam.index_mut(x_index) = 0.0;
                    *trans_cam.index_mut(y_index) = 0.0;
                    *trans_cam.index_mut(z_index) = 0.0;

                    // Instead of moving the camera along the z direction, we scale the scene at the
                    // origin, such that the camera ends up at the right spot within the scene.
                    // Since we use a perspective camera, which can not perceive the total distance
                    // of objects, this scaling does not affect the outcome of the projection. As
                    // long as the camera has the right position inside the scene and the
                    // proportions between objects remain the same, we will get the correct image
                    // from the camera's perspective.

                    // How far we have to scale the scene, such that the camera ends up in the right
                    // spot.
                    let scale = current_z / target_z;
                    let translateZ = format!("scale3d({0},{0},{0})", scale);

                    let matrix3d = matrix_to_css_matrix3d(&trans_cam);
                    // We add the "translate(50%,50%)" to correctly position the origin of the
                    // scene. This has to be applied on right (that is, in world space) to keep it
                    // unaffected by the other transformations.
                    let transform = translateZ + matrix3d.as_str() + "translate(50%,50%)";
                    self.data.view_projection_dom.set_style_or_panic("transform", transform);

                    let left = target_x * scale;
                    let top = target_y * scale;
                    self.data.view_projection_dom.set_style_or_panic("position", "relative");
                    self.data.view_projection_dom.set_style_or_panic("left", format!("{}px", left));
                    self.data.view_projection_dom.set_style_or_panic("top", format!("{}px", top));
                } else {
                    let translateZ = format!("translateZ({}px)", camera.z_zoom_1());
                    let matrix3d = matrix_to_css_matrix3d(&trans_cam);
                    let transform = translateZ + matrix3d.as_str() + "translate(50%,50%)";
                    self.data.view_projection_dom.set_style_or_panic("transform", transform);
                }
            },
            Projection::Orthographic => {
                let transform = matrix_to_css_matrix3d(&trans_cam) + "translate(50%,50%)";
                self.data.dom.set_style_or_panic("transform", transform);
            }
        }
    }

    /// Prepare for movement of this DOM scene by setting the CSS property `will-change: transform`
    /// For more information:
    /// - https://developer.mozilla.org/en-US/docs/Web/CSS/will-change
    /// - https://github.com/enso-org/ide/pull/1465
    pub fn start_movement_mode(&self) {
        self.data.view_projection_dom.set_style_or_panic("will-change", "transform, left, top");
    }

    /// Unset `will-change`. (See `start_movement_mode`)
    pub fn end_movement_mode(&self) {
        self.data.view_projection_dom.set_style_or_panic("will-change", "");
    }
}

fn matrix_to_css_matrix3d(a:&Matrix4<f32>) -> String {
    let entries = a.iter().map(f32::to_string).join(",");
    format!("matrix3d({})", entries)
}
