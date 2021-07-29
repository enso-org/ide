//! This module defines a DOM management utilities.

use crate::prelude::*;

use crate::display::object::traits::*;
use crate::display::camera::Camera2d;
use crate::display::camera::camera2d::Projection;
use crate::display::symbol::DomSymbol;
use crate::display::symbol::dom::eps;
use crate::display::symbol::dom::flip_y_axis;
use crate::system::web;
use crate::system::web::NodeInserter;
use crate::system::web::StyleSetter;

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
    logger : Logger,
    simulate_perspective : Cell<bool>,
}

impl DomSceneData {
    /// Constructor.
    pub fn new(dom:HtmlDivElement, view_projection_dom:HtmlDivElement, logger:Logger) -> Self {
        let simulate_perspective = Cell::new(false);
        Self {dom,view_projection_dom,logger,simulate_perspective}
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
        let logger              = Logger::new_sub(logger,"DomScene");
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

    /// If this is set to false then we will set the CSS `perspective` property and apply all 3D
    /// transformations directly in CSS.
    ///
    /// If this is set to true then we will not set the `perspective` property. Instead, we will
    /// simulate its effect by applying the `scale` transform to the scene. This has the benefit
    /// that some browsers will render the `DomScene` with better visual quality and it works around
    /// a bug with D3 visualizations, as described here: https://github.com/enso-org/ide/pull/1465.
    ///
    /// To produce correct results with the simulated perspective, it has to be guaranteed that the
    /// camera points straight along the z axis and all objects inside this `DomScene` lie within
    /// the z=0 plane.
    pub fn set_simulate_perspective(&self, simulate:bool) {
        self.data.simulate_perspective.set(simulate);
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

        // Round very small values to 0.0 for numerical stability.
        let view_matrix = camera.view_matrix().map(eps);
        // In CSS, the y axis points downwards. (In EnsoGL upwards)
        let view_matrix = flip_y_axis(view_matrix);

        // This variable will collect transformation depend on the specific projection mode.
        let transform: String;
        match camera.projection() {
            Projection::Perspective{..} => {
                if self.data.simulate_perspective.get() {
                    // We unset `perspective` but simulate it by scaling the scene on screen
                    // manually.
                    self.data.dom.set_style_or_panic("perspective","");
                    transform = format!("scale({})",camera.zoom());
                } else {
                    let perspective = camera.z_zoom_1();
                    self.data.dom.set_style_or_panic("perspective",format!("{}px",perspective));
                    // Setting `perspective` in CSS automatically moves the camera backwards, away
                    // from the origin. We have to compensate for this by moving it forward again.
                    transform = format!("translateZ({}px)",perspective);
                }
            },
            Projection::Orthographic => {
                transform = "".to_string();
                self.data.dom.set_style_or_panic("perspective","");
            }
        }

        let transform = transform + matrix_to_css_matrix3d(&view_matrix).as_str();
        // We add the "translate(50%,50%)" to correctly position the origin of the scene. This has
        // to be applied on right (that is, in world space, rather than view space) to keep it
        // unaffected by the other transformations.
        let transform = transform + "translate(50%,50%)";
        self.data.view_projection_dom.set_style_or_panic("transform",transform);
    }

    /// Prepare for movement of this DOM scene by setting the CSS property `will-change: transform`
    /// For more information:
    /// - https://developer.mozilla.org/en-US/docs/Web/CSS/will-change
    /// - https://github.com/enso-org/ide/pull/1465
    pub fn start_movement_mode(&self) {
        self.data.view_projection_dom.set_style_or_panic("will-change","transform");
    }

    /// Unset `will-change`. (See `start_movement_mode`)
    pub fn end_movement_mode(&self) {
        self.data.view_projection_dom.set_style_or_panic("will-change","");
    }
}

fn matrix_to_css_matrix3d(a:&Matrix4<f32>) -> String {
    let entries = a.iter().map(f32::to_string).join(",");
    format!("matrix3d({})",entries)
}
