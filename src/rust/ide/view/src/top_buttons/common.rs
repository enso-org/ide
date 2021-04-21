use prelude::*;

use ensogl::display::object::ObjectOps;
use ensogl::application;
use ensogl::display;
use ensogl::display::style;
use ensogl::display::shape::*;
use ensogl::application::Application;

use enso_frp as frp;
use ensogl::display::style::data::DataMatch;
use ensogl::data::color;
use ensogl::data::color::Rgba;
use ensogl::gui::component::ShapeView;



// ===============
// === Prelude ===
// ===============

/// Prelude meant to be used by sibling modules that provide specific button implementations.
pub mod prelude {
    pub use crate::prelude::*;

    pub use crate::top_buttons::common::ButtonShape;
    pub use crate::top_buttons::common::State;
    pub use crate::top_buttons::common::shape::shape;


    pub use ensogl::system::gpu::shader::glsl::traits::IntoGlsl;
    pub use ensogl::display::shape::*;
    pub use ensogl::display::style::StaticPath;
    pub use ensogl::system::gpu::Attribute;
}

// =================
// === Constants ===
// =================

/// Button radius to be used if theme-provided value is not available.
pub const RADIUS_FALLBACK:f32 = 12.0;



// =============
// === State ===
// =============

/// Visual state of the button.
#[derive(Clone,Copy,Debug)]
pub enum State {
    /// Base look when button is neither hovered nor pressed.
    /// Also used when button was pressed but mouse was hovered out.
    Unconcerned,
    /// Look when button is hovered but not pressed.
    Hovered,
    /// Look when button is being pressed (held down) with mouse hovered.
    Pressed,
}

impl Default for State {
    fn default() -> Self {
        Self::Unconcerned
    }
}


pub trait ButtonShape: CloneRef + display::object::class::Object + DynamicShapeInternals +'static  {
    fn debug_name() -> &'static str;

    fn background_color_path(state:State) -> StaticPath;

    fn icon_color_path(state:State) -> StaticPath;

    fn background_color(&self) -> &DynamicParam<Attribute<Vector4<f32>>>;

    fn icon_color(&self) -> &DynamicParam<Attribute<Vector4<f32>>>;
}


// ==============
// === Shapes ===
// ==============


pub mod shape {
    use super::*;

    pub fn shape(background_color:Var<Vector4<f32>>, icon_color:Var<Vector4<f32>>
             , icon:AnyShape
             , radius:Var<Pixels>)
             -> AnyShape {
        let background_color = Var::<Rgba>::from(background_color);
        let icon_color = Var::<Rgba>::from(icon_color);
        let circle = Circle(radius).fill(background_color);
        let icon   = icon.fill(icon_color);
        (circle + icon).into()
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
        let logger = DefaultTraceLogger::new(Shape::debug_name());
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
        mouse_nearby (bool),
    }
    Output {
        clicked (),
        is_hovered (bool),
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
    frp   : Frp,
    model : Model<Shape>,
    style : StyleWatchFrp,
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

        let radius_frp = style.get(ensogl_theme::application::top_buttons::radius);

        let radius = radius_frp.value();
        println!("Initial radius from style: {:?}",radius);
        model.set_radius(&radius);
        frp.source.size.emit(Model::<Shape>::size_for_radius_event(&radius));

        let background_unconcerned_color = style.get_color(Shape::background_color_path(State::Unconcerned));
        let background_hovered_color = style.get_color(Shape::background_color_path(State::Hovered));
        let background_pressed_color = style.get_color(Shape::background_color_path(State::Pressed));

        let icon_unconcerned_color = style.get_color(Shape::icon_color_path(State::Unconcerned));
        let icon_hovered_color = style.get_color(Shape::icon_color_path(State::Hovered));
        let icon_pressed_color = style.get_color(Shape::icon_color_path(State::Pressed));
        println!("Initial color from style path {:?}",Shape::background_color_path(State::Unconcerned));
        println!("Initial color from style {:?}",background_unconcerned_color.value());
        model.set_background_color(background_unconcerned_color.value());
        let events = &model.shape.events;

        frp::extend! { TRACE_ALL network

            // Radius
            eval radius_frp ((radius) model.set_radius(radius));
            frp.source.size <+ radius_frp.map(Model::<Shape>::size_for_radius_event);

            // Mouse
            frp.source.is_hovered <+ bool(&events.mouse_out,&events.mouse_over);
            pressed_on_me         <- model.shape.events.mouse_down.gate(&frp.is_hovered);
            tracking_for_release  <- bool(&mouse.up_primary,&pressed_on_me);
            mouse_released_on_me  <- mouse.up_primary.gate(&frp.is_hovered);
            was_clicked           <- tracking_for_release.previous();
            frp.source.clicked    <+ mouse_released_on_me.gate(&was_clicked);


            state <- all_with3(&frp.is_hovered,&frp.mouse_nearby,&tracking_for_release,
                |strict_hover,nearby_hover,clicked| {
                    match (strict_hover,nearby_hover,clicked)  {
                            (true , _    , true) => State::Pressed,
                            (_    , true , _   ) => State::Hovered,
                            (_    , _    , true) => State::Hovered,
                            _                    => State::Unconcerned,
                        }
                    });

            frp.source.state <+ state;

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
