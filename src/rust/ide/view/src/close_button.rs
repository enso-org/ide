use crate::prelude::*;

use ensogl::display::object::ObjectOps;
use ensogl::{display, application, Animation};
use ensogl::display::{style};
use ensogl::display::shape::*;
use ensogl::define_shape_system;
use ensogl::application::Application;

use ensogl::system::gpu::shader::glsl::traits::IntoGlsl;

use enso_frp as frp;
use ensogl_theme::application as theme;
use ensogl::display::style::data::DataMatch;
use ensogl::data::color;
use ensogl::data::color::Rgba;
use ensogl::display::style::StaticPath;
use ensogl::gui::component::ShapeView;
use ensogl::system::gpu::Attribute;

pub type Close = shape::close::DynamicShape;
pub type Fullscreen = shape::fullscreen::DynamicShape;
pub type Test = Fullscreen;

#[derive(Clone,Copy,Debug)]
pub enum State {
    Unconcerned, Hovered, Pressed, PressedButMovedOut,
}

impl Default for State {
    fn default() -> Self {
        Self::Unconcerned
    }
}


// ==============
// === Constants ===
// ==============

/// Button radius to be used if theme-provided value is not available.
pub const RADIUS_FALLBACK:f32 = 12.0;

pub trait ButtonShape: CloneRef + display::object::class::Object + display::shape::system::DynamicShapeInternals +'static  {
    fn background_color_path(state:State) -> style::StaticPath;

    fn icon_color_path(state:State) -> style::StaticPath;

    fn background_color(&self) -> &DynamicParam<Attribute<Vector4<f32>>>;

    fn icon_color(&self) -> &DynamicParam<Attribute<Vector4<f32>>>;
}

impl ButtonShape for shape::close::DynamicShape {
    fn background_color_path(state:State) -> StaticPath {
        match state {
            State::Unconcerned => theme::top_buttons::close::normal::background_color,
            State::Hovered => theme::top_buttons::close::hovered::background_color,
            State::Pressed => theme::top_buttons::close::pressed::background_color,
            _ => theme::top_buttons::close::normal::background_color,
        }
    }

    fn icon_color_path(state:State) -> StaticPath {
        match state {
            State::Unconcerned => theme::top_buttons::close::normal::icon_color,
            State::Hovered => theme::top_buttons::close::hovered::icon_color,
            State::Pressed => theme::top_buttons::close::pressed::icon_color,
            _ => theme::top_buttons::close::normal::icon_color,
        }
    }

    fn background_color(&self) -> &DynamicParam<Attribute<Vector4<f32>>> {
        &self.background_color
    }

    fn icon_color(&self) -> &DynamicParam<Attribute<Vector4<f32>>> {
        &self.icon_color
    }
}

impl ButtonShape for shape::fullscreen::DynamicShape {
    fn background_color_path(state:State) -> StaticPath {
        match state {
            State::Unconcerned => theme::top_buttons::fullscreen::normal::background_color,
            State::Hovered => theme::top_buttons::fullscreen::hovered::background_color,
            State::Pressed => theme::top_buttons::fullscreen::pressed::background_color,
            _ => theme::top_buttons::fullscreen::normal::background_color,
        }
    }

    fn icon_color_path(state:State) -> StaticPath {
        match state {
            State::Unconcerned => theme::top_buttons::fullscreen::normal::icon_color,
            State::Hovered => theme::top_buttons::fullscreen::hovered::icon_color,
            State::Pressed => theme::top_buttons::fullscreen::pressed::icon_color,
            _ => theme::top_buttons::fullscreen::normal::icon_color,
        }
    }

    fn background_color(&self) -> &DynamicParam<Attribute<Vector4<f32>>> {
        &self.background_color
    }

    fn icon_color(&self) -> &DynamicParam<Attribute<Vector4<f32>>> {
        &self.icon_color
    }
}

// ==============
// === Shapes ===
// ==============


mod shape {
    use super::*;

    fn shape(background_color:Var<Vector4<f32>>, icon_color:Var<Vector4<f32>>
             , icon:display::shape::primitive::def::AnyShape
            , radius:Var<Pixels>)
             -> display::shape::primitive::def::AnyShape {
        let background_color = Var::<color::Rgba>::from(background_color);
        let icon_color = Var::<color::Rgba>::from(icon_color);
        let circle = Circle(radius).fill(background_color);
        let icon   = icon.fill(icon_color);
        (circle + icon).into()
    }

    pub mod close {
        use super::*;
        define_shape_system! {
            (background_color:Vector4<f32>, icon_color:Vector4<f32>) {
                let radius = Var::min(Var::input_width(),Var::input_height()) / 2.0;
                let angle = Radians::from(45.0.degrees());
                let bar_length = &radius * 4.0 / 3.0;
                let bar_width = &bar_length / 6.5;
                let bar = Rect((bar_length, &bar_width)).corners_radius(bar_width);
                let cross = (bar.rotate(angle) + bar.rotate(-angle)).into();
                shape(background_color, icon_color, cross, radius)
            }
        }
    }

    pub mod fullscreen {
        use super::*;
        define_shape_system! {
            (background_color:Vector4<f32>, icon_color:Vector4<f32>) {
                let radius = Var::min(Var::input_width(),Var::input_height()) / 2.0;
                let triangle_size = &radius * 2.0 / 3.0;
                let icon = Triangle(triangle_size.glsl(), triangle_size.glsl());

                shape(background_color, icon_color, icon.into(), radius)
            }
        }
    }
}



// =============
// === Model ===
// =============

/// An internal model of Status Bar component
#[derive(Clone,CloneRef,Debug)]
#[clone_ref(bound="Shape:CloneRef")]
#[allow(missing_docs)]
pub struct Model<Shape> {
    pub app            : Application,
    pub logger         : DefaultTraceLogger,
    pub display_object : display::object::Instance,
    pub shape          : ShapeView<Shape>,
}

#[allow(missing_docs)]
impl<Shape: ButtonShape> Model<Shape> {
    pub fn new(app:&Application) -> Self {
        let app    = app.clone_ref();
        let logger = DefaultTraceLogger::new("CloseButton");
        let display_object  = display::object::Instance::new(&logger);
        let shape  = ShapeView::new(&logger);
        display_object.add_child(&shape);
        Self{app,logger,display_object,shape}
    }

    pub fn set_background_color(&self, color:impl Into<Rgba>) {
        let rgba = color.into();
        println!("Setting circle color: {:?}", rgba);
        self.shape.background_color().set(rgba.into());
    }

    pub fn set_icon_color(&self, color:impl Into<Rgba>) {
        self.shape.icon_color().set(color.into().into());
    }

    fn get_radius(radius:&Option<style::data::Data>) -> f32 {
        radius.as_ref().and_then(DataMatch::number).unwrap_or(RADIUS_FALLBACK)
    }

    pub fn set_radius(&self, radius:&Option<style::data::Data>) {
        let radius = Self::get_radius(radius);
        //println!("Setting radius to {}", radius);
        self.shape.size().set(Vector2::new(radius * 2.0, radius * 2.0));
        self.shape.set_position_x(radius);
        self.shape.set_position_y(-radius);
    }

    pub fn size_for_radius(radius:f32) -> Vector2<f32> {
        Vector2(radius * 2.0,radius * 2.0)
    }

    pub fn size_for_radius_event(radius:&Option<style::data::Data>) -> Vector2<f32> {
        Self::size_for_radius(Self::get_radius(radius))
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
        clicked (),
        state(State),
        size (Vector2<f32>),
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
#[clone_ref(bound="Shape:CloneRef")]
#[allow(missing_docs)]
pub struct View<Shape> {
    pub frp   : Frp,
    pub model : Model<Shape>,
    pub style  : StyleWatchFrp,
}

impl<Shape:ButtonShape> View<Shape> {

    /// Constructor.
    pub fn new(app: &Application) -> Self {
        let frp = Frp::new();
        let model  : Model<Shape> = Model::new(app);
        let network = &frp.network;
        let scene = app.display.scene();
        let style = StyleWatchFrp::new(&app.display.scene().style_sheet);

        let mouse = &scene.mouse.frp;
        frp.enabled(true);

        // let highlight_stage = Animation::new(&network);

        // let circle_color = color::Animation::new(&network);
        let default_icon_color = style.get_color(Shape::icon_color_path(State::Unconcerned)).value();
        let icon_color = color::Animation::new(&network);
        icon_color.target(color::Lcha::from(default_icon_color));
        model.set_icon_color(default_icon_color);

        let default_background_color = style.get_color(Shape::background_color_path(State::Unconcerned)).value();
        let background_color = color::Animation::new(&network);
        background_color.target(color::Lcha::from(default_background_color));
        model.set_icon_color(default_background_color);

        let radius_frp = style.get(theme::top_buttons::radius);

        let radius = radius_frp.value();
        println!("Initial radius from style: {:?}",radius);
        model.set_radius(&radius);
        frp.source.size.emit(Model::<Shape>::size_for_radius_event(&radius));

        model.set_background_color(style.get_color(Shape::background_color_path(State::Unconcerned)).value());

        let background_unconcerned_color = style.get_color(Shape::background_color_path(State::Unconcerned));
        let background_hovered_color = style.get_color(Shape::background_color_path(State::Hovered));
        let background_pressed_color = style.get_color(Shape::background_color_path(State::Pressed));

        let icon_unconcerned_color = style.get_color(Shape::icon_color_path(State::Unconcerned));
        let icon_hovered_color = style.get_color(Shape::icon_color_path(State::Hovered));
        let icon_pressed_color = style.get_color(Shape::icon_color_path(State::Pressed));

        let events = &model.shape.events;
        frp::extend! { TRACE_ALL network

            // Radius
            eval radius_frp ((radius) model.set_radius(radius));
            frp.source.size <+ radius_frp.map(Model::<Shape>::size_for_radius_event);

            // Mouse
            is_hovered <- bool(&events.mouse_out,&events.mouse_over);
            tracking_for_release <- gate(&model.shape.events.mouse_down,&is_hovered);



            //released <- gate(&mouse.up_primary,&is_pressed);

            is_mouse_pressed <- bool(&mouse.up_primary,&model.shape.events.mouse_down);
            is_pressed <- sample(&is_hovered, &model.shape.events.mouse_down);


            frp.source.clicked <+ gate(&model.shape.events.mouse_up,&is_hovered);

            //frp.source.clicked <+ gate(&model.shape.events.mouse_up,&is_hovered);

            mouse_state <- all(&is_hovered, &is_pressed);
            frp.source.state <+ mouse_state.map(|(hovered,pressed)| {
                match (hovered,pressed) {
                    (false,false) => State::Unconcerned,
                    (true,false) => State::Hovered,
                    (false,true) => State::PressedButMovedOut,
                    (true,true) => State::Pressed,
                }
            });

            // Color animations
            background_color_helper <- all4(&frp.source.state,&background_unconcerned_color,&background_hovered_color,&background_pressed_color);
            background_color.target <+ background_color_helper.map(|(state,unconcerned,hovered,pressed)| {
                match state {
                    State::Hovered => hovered,
                    State::Pressed => pressed,
                    _              => unconcerned,
                }.into()
            });

            icon_color_helper <- all4(&frp.source.state,&icon_unconcerned_color,&icon_hovered_color,&icon_pressed_color);
            icon_color.target <+ icon_color_helper.map(|(state,unconcerned,hovered,pressed)| {
                match state {
                    State::Hovered => hovered,
                    State::Pressed => pressed,
                    _              => unconcerned,
                }.into()
            });

            eval icon_color.value ((color) model.set_icon_color(color));
            eval background_color.value ((color) model.set_background_color(color));
        }

        Self {frp,model,style}
    }
}

impl<Shape> display::Object for View<Shape> {
    fn display_object(&self) -> &display::object::Instance { &self.model.display_object }
}

impl<Shape> Deref for View<Shape> {
    type Target = Frp;
    fn deref(&self) -> &Self::Target { &self.frp }
}

impl<Shape> application::command::FrpNetworkProvider for View<Shape> {
    fn network(&self) -> &frp::Network { &self.frp.network }
}

impl<Shape:ButtonShape> application::View for View<Shape> {
    fn label() -> &'static str { "CloseButton" }
    fn new(app:&Application) -> Self { View::new(app) }
    fn app(&self) -> &Application { &self.model.app }
}
