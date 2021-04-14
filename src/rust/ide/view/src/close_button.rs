use crate::prelude::*;

use ensogl::application::Application;
use ensogl::display::camera::Camera2d;
use ensogl::display::Scene;
use ensogl::display::shape::*;
use ensogl::display::style;
use ensogl::display;
use ensogl_gui_components::shadow;
use ensogl_text as text;
use ensogl_theme as theme;
use std::future::Future;
use enso_frp as frp;

use ensogl::data::color;

// =================
// === Constants ===
// =================

/// Breadcrumb vertical margin.
pub const VERTICAL_MARGIN:f32 = 0.0;
/// Breadcrumb left margin.
pub const LEFT_MARGIN:f32 = 0.0;
/// Breadcrumb right margin.
pub const RIGHT_MARGIN  : f32 = 0.0;
const ICON_LEFT_MARGIN  : f32 = 0.0;
const ICON_RIGHT_MARGIN : f32 = 0.0;
const ICON_RADIUS       : f32 = 6.0;
const ICON_SIZE         : f32 = ICON_RADIUS * 2.0;
const ICON_RING_WIDTH   : f32 = 1.5;
const ICON_ARROW_SIZE   : f32 = 4.0;
const SEPARATOR_SIZE    : f32 = 6.0;
/// Breadcrumb padding.
pub const PADDING      : f32 = 1.0;
const SEPARATOR_MARGIN : f32 = 10.0;



mod shape {
    use super::*;

    ensogl::define_shape_system! {
        () {
            let red = 1.0;
            let green = 0.0;
            let blue = 0.0;
            let alpha = 0.0;
            let outer_circle  = Circle((ICON_RADIUS).px());
            let inner_circle  = Circle((ICON_RADIUS - ICON_RING_WIDTH).px());
            let ring          = outer_circle - inner_circle;
            let shape         = ring;
            let color         = format!("vec4({},{},{},{})",red,green,blue,alpha);
            let color : Var<color::Rgba> = color.into();
            shape.fill(color).into()
        }
    }
}

// ===========
// === FRP ===
// ===========

ensogl::define_endpoints! {
    Input {
    }
    Output {
        clicked (),
    }
}



// =============
// === Model ===
// =============

/// An internal model of Status Bar component
#[derive(Clone,CloneRef,Debug)]
struct Model {
    logger          : Logger,
    display_object  : display::object::Instance,
    root            : display::object::Instance,
    shape           : shape::View,
    label           : text::Area,
    camera          : Camera2d,
}

impl Model {

    fn new(app:&Application) -> Self {
        let scene           = app.display.scene();
        let logger          = Logger::new("StatusBar");
        let display_object  = display::object::Instance::new(&logger);
        let root            = display::object::Instance::new(&logger);
        let shape           = shape::View::new(&logger);
        let label           = text::Area::new(app);
        let camera          = scene.camera();

        // scene.layers.breadcrumbs_background.add_exclusive(&background);
        label.remove_from_scene_layer_DEPRECATED(&scene.layers.main);
        label.add_to_scene_layer_DEPRECATED(&scene.layers.breadcrumbs_text);

        let text_color_path = theme::application::status_bar::text;
        let style           = StyleWatch::new(&app.display.scene().style_sheet);
        let text_color      = style.get_color(text_color_path);
        label.frp.set_color_all.emit(text_color);
        label.frp.set_default_color.emit(text_color);

        Self {logger,display_object,root,shape,label,camera}
    }

    fn init(&self) {

    }
}

// ============
// === View ===
// ============

/// The StatusBar component view.
///
/// The status bar gathers information about events and processes occurring in the Application.
// TODO: This is a stub. Extend it when doing https://github.com/enso-org/ide/issues/1193
#[derive(Clone,CloneRef,Debug)]
pub struct View {
    frp   : Frp,
    model : Model,
}

impl View {
    /// Constructor.
    pub fn new(app: &Application) -> Self {
        let frp = Frp::new();
        let model = Model::new(app);
        let network = &frp.network;
        let scene = app.display.scene();

        // FIXME : StyleWatch is unsuitable here, as it was designed as an internal tool for shape
        //         system (#795)
        let styles = StyleWatch::new(&scene.style_sheet);
        let hover_color = styles.get_color(theme::graph_editor::breadcrumbs::hover);

        frp::extend! { network
        }


        // === Animations ===

        frp::extend! {network
        }

        Self { frp, model }
    }
}

impl display::Object for View {
    fn display_object(&self) -> &display::object::Instance<Scene> { &self.model.display_object }
}

impl Deref for View {
    type Target = Frp;

    fn deref(&self) -> &Self::Target { &self.frp }
}
