//! Text label component. Appears as text with background.

use crate::prelude::*;

use enso_frp as frp;
use enso_frp;
use ensogl_core::application::Application;
use ensogl_core::data::color;
use ensogl_core::display::shape::*;
use ensogl_core::display::traits::*;
use ensogl_core::display;
use ensogl_text as text;


// ==========================
// === Shapes Definitions ===
// ==========================


// === Constants ===

const TEXT_OFFSET : f32 = 10.0;
const TEXT_SIZE   : f32 = 12.0;
const HEIGHT      : f32 = TEXT_SIZE * 3.0;
const PADDING     : f32 = 15.0;
const SHADOW_SIZE : f32 = 10.0;


// === Background ===

mod background {
    use super::*;

    ensogl_core::define_shape_system! {
        (style:Style,bg_color:Vector4) {
            use ensogl_theme::graph_editor::node as node_theme;

            let width  = Var::<Pixels>::from("input_size.x");
            let height = Var::<Pixels>::from("input_size.y");
            let width  = width  - PADDING.px() * 2.0;
            let height = height - PADDING.px() * 2.0;
            let radius = &height / 2.0;
            let shape  = Rect((&width,&height)).corners_radius(&radius);
            let shape  = shape.fill(Var::<color::Rgba>::from(bg_color.clone()));


            // === Shadow ===
            let alpha         = Var::<f32>::from(format!("({0}.w)",bg_color));
            let border_size_f = 16.0;
            let shadow_size   = SHADOW_SIZE.px();
            let shadow_width  = &width  + &shadow_size * 2.0;
            let shadow_height = &height + &shadow_size * 2.0;
            let shadow_radius = &shadow_height / 2.0;
            let shadow        = Rect((shadow_width,shadow_height)).corners_radius(shadow_radius);
            let base_color    = color::Rgba::from(style.get_color(node_theme::shadow));
            let base_color    = Var::<color::Rgba>::from(base_color);
            let base_color    = base_color.multiply_alpha(&alpha);
            let fading_color  = color::Rgba::from(style.get_color(node_theme::shadow::fading));
            let fading_color  = Var::<color::Rgba>::from(fading_color);
            let fading_color  = fading_color.multiply_alpha(&alpha);
            let exponent      = style.get_number_or(node_theme::shadow::exponent,2.0);
            let shadow_color  = color::LinearGradient::new()
                .add(0.0,fading_color.into_linear())
                .add(1.0,base_color.into_linear());
            let shadow_color = color::SdfSampler::new(shadow_color)
                .max_distance(border_size_f)
                .slope(color::Slope::Exponent(exponent));
            let shadow        = shadow.fill(shadow_color);

            (shadow+shape).into()
        }
    }
}



// ===========
// === FRP ===
// ===========

ensogl_core::define_endpoints! {
    Input {
        set_content(String),
        set_opacity(f32)
    }
    Output {
        size (Vector2)
    }
}



// =============
// === Model ===
// =============

#[derive(Clone,Debug)]
struct Model {
    background     : background::View,
    label          : text::Area,
    display_object : display::object::Instance,
    app            : Application
}

impl Model {
    fn new(app: Application) -> Self {
        let app            = app.clone_ref();
        let scene          = app.display.scene();
        let logger         = Logger::new("TextLabel");
        let display_object = display::object::Instance::new(&logger);
        let label          = app.new_view::<text::Area>();
        let background     = background::View::new(&logger);
        display_object.add_child(&background);
        display_object.add_child(&label);

        // Depth sorting of labels to in front of the background.
        label.remove_from_scene_layer_DEPRECATED(&scene.layers.main);
        label.add_to_scene_layer_DEPRECATED(&scene.layers.label);

        Model { label, display_object, background, app }
    }

    pub fn height(&self) -> f32 {
        HEIGHT
    }

    fn set_width(&self, width:f32) -> Vector2 {
        let height      = self.height();
        let size        = Vector2(width*1.25,height);
        let padded_size = size + Vector2(PADDING,PADDING) * 2.0;
        self.background.size.set(padded_size);
        let text_origin = Vector2(PADDING / 2.0 + TEXT_OFFSET - size.x/2.0,TEXT_SIZE/2.0);
        self.label.set_position_xy(text_origin);
        padded_size
    }

    fn set_content(&self, t:&str) -> Vector2 {
        self.label.set_content(t);
        self.set_width(self.label.width.value())
    }

    fn set_opacity(&self, value:f32) {
        let style           = StyleWatch::new(&self.app.display.scene().style_sheet);
        let text_color_path = ensogl_theme::graph_editor::node::text;
        let text_color      = style.get_color(text_color_path).multiply_alpha(value);
        let text_color      = color::Rgba::from(text_color);
        self.label.frp.set_color_all.emit(text_color);
        self.label.frp.set_default_color.emit(text_color);

        let bg_color_path = ensogl_theme::graph_editor::node::background;
        let bg_color      = style.get_color(bg_color_path).multiply_alpha(value);
        let bg_color      = color::Rgba::from(bg_color);
        self.background.bg_color.set(bg_color.into())
    }
}



// ============================
// === Text Label Component ===
// ============================

#[allow(missing_docs)]
#[derive(Clone,CloneRef,Debug)]
pub struct TextLabel {
    model   : Rc<Model>,
    pub frp : Rc<Frp>,
}


impl TextLabel {
    /// Constructor.
    pub fn new(app:Application) -> Self {
        let frp   = Rc::new(Frp::new());
        let model = Rc::new(Model::new(app.clone_ref()));
        TextLabel {frp,model}.init()
    }

    fn init(self) -> Self {
        let frp     = &self.frp;
        let network = &frp.network;
        let model   = &self.model;

        frp::extend! { network
            frp.source.size <+ frp.set_content.map(f!((t)
                model.set_content(t)
            ));

            eval frp.set_opacity((value) model.set_opacity(*value));
        }

        self
    }
}

impl display::Object for TextLabel {
    fn display_object(&self) -> &display::object::Instance { &self.model.display_object }
}
