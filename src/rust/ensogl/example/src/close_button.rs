//! Example scene showing simple usage of a shape system.

use ensogl_core::prelude::*;

use ensogl_core::display::navigation::navigator::Navigator;
use ensogl_core::system::web;
use wasm_bindgen::prelude::*;
use ensogl_core::display::object::ObjectOps;
use ensogl_core::{display, application};
use ensogl_core::display::shape::ShapeSystem;
use ensogl_core::display::world::*;
use ensogl_core::display::shape::*;
use ensogl_core::data::color;
use std::f32::consts::PI;
use ensogl_core::define_shape_system;
use ensogl_core::application::{Application, shortcut};

use enso_frp as frp;
use ensogl_text_msdf_sys::run_once_initialized;

// ==============
// === Shapes ===
// ==============

mod shape {
    use super::*;

    define_shape_system! {
        () {
            let circle_color = color::Rgb::from_integral(0xEC,0x6A,0x5F);
            let circle    = Circle(12.px());
            let circle    = circle.fill(color::Rgb::from(circle_color));
            let angle     = Radians::from(45.0.degrees());

            let cross_color = color::Rgb::from_integral(0x8D,0x1A,0x10);
            let bar = Rect((16.pixels(), 2.5.px()))
                .corners_radius(2.px())
                .fill(color::Rgb::from(cross_color));
            let cross      = bar.rotate(angle) + bar.rotate(-angle);
            (circle + cross).into()
        }
    }
}

// =============
// === Model ===
// =============

/// An internal model of Status Bar component
#[derive(Clone,CloneRef,Debug)]
struct Model {
    app   : Application,
    logger          : DefaultTraceLogger,
    display_object  : display::object::Instance,
    shape           : shape::View,
}

impl Model {
    pub fn new(app:&Application) -> Self {
        let app            = app.clone_ref();
        let logger         = DefaultTraceLogger::new("CloseButton");
        let display_object = display::object::Instance::new(&logger);
        let shape          = shape::View::new(&logger);
        shape.size.set(Vector2::new(100.0, 100.0));
        display_object.add_child(&shape);
        Self{app,logger,display_object,shape}
    }
}



// ===========
// === FRP ===
// ===========

ensogl_core::define_endpoints! { [TRACE_ALL]
    Input {
        enabled (bool),
    }
    Output {
        clicked (),
        is_hovered (bool),
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
        println!("Setting View up");
        let frp = Frp::new();
        let model = Model::new(app);
        let network = &frp.network;
        let scene = app.display.scene();

        let events = &model.shape.events;

        let foo = || println!("Hello 3");

        frp::extend! { network
            trace model.shape.events.mouse_up;
            trace model.shape.events.mouse_down;
            trace model.shape.events.mouse_over;
            trace model.shape.events.mouse_out;
            trace model.shape.events.on_drop;

            frp.source.is_hovered <+ bool(&events.mouse_out,&events.mouse_over);

            trace scene.mouse.frp.down_primary;
            eval_ model.shape.events.mouse_down(frp.source.clicked.emit(()));
            eval_ model.shape.events.mouse_down([]foo());
        }

        Self { frp, model }
    }
}

impl display::Object for View {
    fn display_object(&self) -> &display::object::Instance { &self.model.display_object }
}

impl Deref for View {
    type Target = Frp;
    fn deref(&self) -> &Self::Target { &self.frp }
}

impl application::command::FrpNetworkProvider for View {
    fn network(&self) -> &frp::Network { &self.frp.network }
}

impl application::View for View {
    fn label() -> &'static str { "CloseButton" }
    fn new(app:&Application) -> Self { View::new(app) }
    fn app(&self) -> &Application { &self.model.app }
}


// ===================
// === Entry Point ===
// ===================

/// The example entry point.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_close_button() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    run_once_initialized(|| {
        println!("Initializing");
        let app = Application::new(&web::get_html_element_by_id("root").unwrap());

        let shape:View = app.new_view();
        shape.model.shape.size.set(Vector2::new(300.0, 300.0));
        app.display.add_child(&shape);

        let scene         = app.display.scene();
        let camera        = scene.camera().clone_ref();
        let navigator     = Navigator::new(&scene,&camera);

        let logger : Logger = Logger::new("CloseButton");
        let network = enso_frp::Network::new("test");
        enso_frp::extend! {network
            eval shape.clicked([logger](()) {
                info!(logger, "Clicked")
            });
        }

        std::mem::forget(shape);
        std::mem::forget(network);
        std::mem::forget(navigator);
        mem::forget(app);
        println!("Initializing");
    });
}
