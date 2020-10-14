//! Select List Component
use crate::prelude::*;

use enso_frp as frp;
use ensogl_core::application::Application;
use ensogl_core::data::color;
use ensogl_core::display;
use ensogl_core::display::shape::*;
use ensogl_core::gui::component::ShapeView;
use ensogl_shape_utils::component_color::ComponentColor;
use ensogl_shape_utils::component_color;


// =================
// === Colorable ===
// =================

/// A shape that can have a single color.
// TODO implement a derive like macro for this trait that can be used for shape creation.
pub trait ColorableShape : system::Shape {
    /// Set the color of the shape.
    fn set_color(&self, color:color::Rgba);
}



// ===========
// === Frp ===
// ===========

ensogl_text::define_endpoints! {
    Input {
        set_visibility (bool),
        set_base_color (component_color::Source),
        set_size       (Vector2),
    }
    Output {
        toggle_state (bool),
        mouse_over   (),
        mouse_out    (),
    }
}


// =============
// === Model ===
// =============

#[derive(Clone,Debug)]
struct Model<Shape> {
    icon : ShapeView<Shape>,
}

impl<Shape:ColorableShape+'static> Model<Shape> {
    fn new(app:&Application) -> Self {
        let logger = Logger::new("ToggleButton");
        let icon   = ShapeView::new(&logger, app.display.scene());
        Self{icon}
    }
}



// =====================
// === Toggle Button ===
// =====================

/// A UI component that acts as a toggle which can be toggled on and of. Has a visible shape
/// that acts as button and changes color depending on the toggle state.
#[derive(Clone,CloneRef,Debug)]
pub struct ToggleButton<Shape> {
    model:Rc<Model<Shape>>,
    /// Public FRP api.
    pub frp:Frp
}

impl<Shape:ColorableShape+'static> ToggleButton<Shape>{
    /// Constructor.
    pub fn new(app:&Application) -> Self {
        let model = Rc::new(Model::<Shape>::new(app));
        let frp   = Frp::new_network();
        Self {model,frp}.init_frp(app)
    }

    fn init_frp(self, app:&Application) -> Self {
        let network = &self.frp.network;
        let frp     = &self.frp;
        let model   = &self.model;

        let component_color = ComponentColor::new(&app);
        let color_frp       = component_color.frp;
        let icon            = &model.icon.events;

        frp::extend! { network


             // === Input Processing ===

            eval frp.set_base_color ((color_source) color_frp.set_source(color_source.clone()) );
            eval frp.set_size ((size) {
                model.icon.shape.sprites().iter().for_each(|sprite| sprite.size.set(*size))
            });

             // === Mouse Interactions ===

             frp.source.mouse_over <+ icon.mouse_over;
             frp.source.mouse_out  <+ icon.mouse_out;

             eval_ icon.mouse_over ({
                 color_frp.set_state(component_color::State::Base)
            });

            frp.source.toggle_state <+ icon.mouse_down.toggle();


            // === Color ===

            invisible <- frp.set_visibility.gate_not(&frp.set_visibility);
            eval_ invisible (color_frp.set_state(component_color::State::Transparent ));

            visible    <- frp.set_visibility.gate(&frp.set_visibility);
            is_hovered <- bool(&icon.mouse_out,&icon.mouse_over);

            button_state <- all3(&visible,&is_hovered,&frp.toggle_state);

            eval button_state ([color_frp]((visible,hovered,toggle_state)) {
                match(*visible,*hovered,*toggle_state) {
                    (false,_,_)        => color_frp.set_state(component_color::State::Transparent ),
                    (true,true,_)      => color_frp.set_state(component_color::State::Base ),
                    (true,false,true)  => color_frp.set_state(component_color::State::Base ),
                    (true,false,false) => color_frp.set_state(component_color::State::Dim ),
                }
            });

            eval color_frp.color ((color) model.icon.shape.set_color(color.into()));
        }

        color_frp.set_state(component_color::State::Dim);

        self
    }

    /// Return the underlying shape view. Note that some parameters like size and color will be
    /// overwritten regularly by internal the `ToggleButton` mechanics.
    pub fn view(&self) -> ShapeView<Shape> {
        self.model.icon.clone_ref()
    }
}

impl<T> display::Object for ToggleButton<T> {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.icon.display_object()
    }
}
