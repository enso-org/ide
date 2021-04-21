pub mod close;
pub mod common;
pub mod fullscreen;


use ensogl::prelude::*;

use ensogl::display::navigation::navigator::Navigator;
use ensogl::system::web;
use wasm_bindgen::prelude::*;
use ensogl::display::object::ObjectOps;
use ensogl::{display, application};
use ensogl::display::shape::ShapeSystem;
use ensogl::display::world::*;
use ensogl::display::shape::*;
use ensogl::data::color;
use std::f32::consts::PI;
use ensogl::define_shape_system;
use ensogl::application::{Application, shortcut};

use enso_frp as frp;
use ensogl_text_msdf_sys::run_once_initialized;
use ensogl::display::geometry::glsl::Layout;

// ==============
// === Shapes ===
// ==============

mod shape {
    use super::*;

    define_shape_system! {
        () {
            Plane().fill(color::Rgb(0.0,0.2,0.0)).into()
        }
    }
}

#[derive(Clone,Copy,Debug)]
pub struct LayoutParams<T> {
    pub spacing        : T,
    pub padding_left   : T,
    pub padding_top    : T,
    pub padding_right  : T,
    pub padding_bottom : T,
}

impl Default for LayoutParams<f32> {
    fn default() -> Self {
        Self {
            spacing        : 8.0,
            padding_left   : 13.0,
            padding_top    : 13.0,
            padding_right  : 13.0,
            padding_bottom : 13.0,
        }
    }
}

impl<T> LayoutParams<T> {
    fn map<U>(&self, f:impl Fn(&T)->U) -> LayoutParams<U> {
        LayoutParams {
            spacing        : f(&self.spacing),
            padding_left   : f(&self.padding_left),
            padding_top    : f(&self.padding_top),
            padding_right  : f(&self.padding_right),
            padding_bottom : f(&self.padding_bottom),
        }
    }

    fn track_style(&self, network:&frp::Network, style:&StyleWatchFrp) {

    }

    fn from_tuple(tuple:(T,T,T,T,T)) -> LayoutParams<T> {
        let (spacing,padding_left,padding_top,padding_right,padding_bottom) = tuple;
        LayoutParams {spacing,padding_left,padding_top,padding_right,padding_bottom}
    }
}

impl LayoutParams<frp::Sampler<f32>> {
    fn from_theme(style:&StyleWatchFrp) -> Self {
        let default        = LayoutParams::default();
        let spacing        = style.get_number_or(ensogl_theme::application::top_buttons::spacing, default.spacing);
        let padding_left   = style.get_number_or(ensogl_theme::application::top_buttons::padding::left, default.padding_left);
        let padding_top    = style.get_number_or(ensogl_theme::application::top_buttons::padding::top, default.padding_top);
        let padding_right  = style.get_number_or(ensogl_theme::application::top_buttons::padding::right, default.padding_right);
        let padding_bottom = style.get_number_or(ensogl_theme::application::top_buttons::padding::bottom, default.padding_bottom);
        Self {spacing,padding_left,padding_top,padding_right,padding_bottom}
    }

    fn sample_value(&self) -> LayoutParams<f32> {
        self.map(|sampler| sampler.value())
    }

    fn flatten(&self, network:&frp::Network) -> frp::Stream<LayoutParams<f32>> {
        let style_tuple = network.all5("layout_style", &self.spacing, &self.padding_left, &self.padding_top, &self.padding_right, &self.padding_bottom);
        let style = network.map("layout_style",&style_tuple, |v| LayoutParams::from_tuple(*v));
        style
    }
}
// =============
// === Model ===
// =============

/// An internal model of Status Bar component
#[derive(Clone,CloneRef,Debug)]
struct Model {
    app             : Application,
    logger          : DefaultTraceLogger,
    display_object  : display::object::Instance,
    shape           : shape::View,
    close           : close::View,
    fullscreen      : fullscreen::View,
}

impl Model {
    pub fn new(app:&Application) -> Self {

        let app            = app.clone_ref();
        let logger         = DefaultTraceLogger::new("TopButtons");
        let display_object = display::object::Instance::new(&logger);

        ensogl::shapes_order_dependencies! {
            app.display.scene() => {
                shape -> close::shape;
                shape -> fullscreen::shape;
            }
        };
        let close = close::View::new(&app);
        display_object.add_child(&close);

        let fullscreen = fullscreen::View::new(&app);
        display_object.add_child(&fullscreen);

        let shape          = shape::View::new(&logger);
        shape.set_position(default());
        display_object.add_child(&shape);

        let ret = Self{app,logger,display_object,shape,close,fullscreen};
        ret
    }

    pub fn set_layout(&self, layout:LayoutParams<f32>) -> Vector2 {
        println!("Updating layout: {:?}",layout);
        let LayoutParams{spacing,padding_left,padding_top,padding_right,padding_bottom} = layout;
        let close_size = self.close.size.value();
        let fullscreen_size = self.fullscreen.size.value();

        self.close.set_position_xy(Vector2(padding_left,-padding_top));
        let fullscreen_x = padding_left + close_size.x + spacing;
        self.fullscreen.set_position_xy(Vector2(fullscreen_x,-padding_top));

        let width = fullscreen_x + fullscreen_size.x + padding_right;
        let height = padding_top + max(close_size.y, fullscreen_size.y) + padding_bottom;
        println!("Close size {:?}", close_size);
        println!("Fullscreen size {:?}", fullscreen_size);
        println!("==={} {} {}", fullscreen_x,width,height);

        let size = Vector2(width, height);
        println!("+++{:?}", size);
        self.shape.set_position_xy(Vector2(size.x, -size.y) / 2.0);
        self.shape.size.set(size);
        size
    }
}



// ===========
// === FRP ===
// ===========

ensogl::define_endpoints! { [TRACE_ALL]
    Input {
        enabled (bool),
    }
    Output {
        close(),
        fullscreen(),
        size(Vector2<f32>),
        mouse_near_buttons(bool),
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


        let fallback = 13.0;

        let style = StyleWatchFrp::new(&app.display.scene().style_sheet);
        let style_frp = LayoutParams::from_theme(&style);
        let initial_style = style_frp.sample_value();
        let initial_size = model.set_layout(initial_style);
        frp.source.size.emit(initial_size);
        let layout_style = style_frp.flatten(&network);


        frp::extend! { TRACE_ALL network
            button_resized <- any_(&model.close.size,&model.fullscreen.size);
            layout_on_button_change <- sample(&layout_style,&button_resized);
            need_relayout <- any(&layout_style,&layout_on_button_change);
            frp.source.size <+ need_relayout.map(f!((layout) model.set_layout(*layout)));
            frp.source.mouse_near_buttons <+ bool(&model.shape.events.mouse_out,&model.shape.events.mouse_over);
        }

        let ret = Self { frp, model };
        ret
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
