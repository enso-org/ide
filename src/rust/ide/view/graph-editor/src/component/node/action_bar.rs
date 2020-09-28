//! Definition of the `ActionBar` component for the `visualization::Container`.

use crate::prelude::*;

use crate::component::node;

use enso_frp as frp;
use enso_frp;
use ensogl::application::Application;
use ensogl::data::color;
use ensogl::display::shape::*;
use ensogl::display::traits::*;
use ensogl::display;
use ensogl::gui::component;


mod compound_shape {
    use crate::prelude::*;
    use enso_frp as frp;


    use ensogl::gui::component::{ShapeViewEvents, ShapeView};

    ensogl_text::define_endpoints! {
        Input {
            dummy ()
        }
        Output {
            mouse_over (),
            mouse_out  (),
        }
    }

    #[derive(Clone,CloneRef,Debug)]
    pub struct CompoundShapeEvents {
        pub frp : Frp,
    }

    impl CompoundShapeEvents {

        pub fn new() -> Self {
            let frp = Frp::new_network();
            Self{frp}
        }

        /// Connect the given `ShapeViewEvents` to the mouse events of all sub-shapes.
        pub fn add_sub_shape<T>(&self, view:&ShapeView<T>) {
            let network       = &self.frp.network;
            let compound_frp  = &self.frp;
            let sub_frp       = &view.events;

            /// FIXME avoid in/out events when switching shape
            /// TODO check for memory leaks
            frp::extend! { network

                compound_frp.source.mouse_over <+ sub_frp.mouse_over;
                compound_frp.source.mouse_out  <+ sub_frp.mouse_out;
            }
        }
    }
}


// =================
// === Constants ===
// =================

const HOVER_COLOR : color::Rgba = color::Rgba::new(1.0,0.0,0.0,0.000_001);



// ===============
// === Shapes  ===
// ===============


/// Invisible rectangular area that can be hovered.
mod hover_rect {
    use super::*;

    ensogl::define_shape_system! {
        (corner_radius:f32) {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let rect           = Rect((&width,&height));
            let rect_rounded   = rect.corners_radius(corner_radius);
            let rect_filled    = rect_rounded.fill(HOVER_COLOR);
            rect_filled.into()
        }
    }
}

// ===========
// === Frp ===
// ===========

ensogl_text::define_endpoints! {
    Input {
        set_size   (Vector2),
        show_icons (),
        hide_icons (),
    }
    Output {
        mouse_over                (),
        mouse_out                 (),
        action_visbility_clicked  (),
        action_freeze_clicked     (),
        action_skip_clicked       (),
    }
}



// ========================
// === Action Bar Model ===
// ========================

#[derive(Clone,CloneRef,Debug)]
struct Model {
    hover_area            : component::ShapeView<hover_rect::Shape>,

    icons                 : display::object::Instance,
    icon_freeze           : component::ShapeView<node::icon::action::freeze::Shape>,
    icon_visibility       : component::ShapeView<node::icon::action::visibility::Shape>,
    icon_skip             : component::ShapeView<node::icon::action::skip::Shape>,

    display_object        : display::object::Instance,
    size                  : Rc<Cell<Vector2>>,

    all_shapes            : compound_shape::CompoundShapeEvents,

}

impl Model {
    fn new(app:&Application) -> Self {
        let scene                 = app.display.scene();
        let logger                = Logger::new("ActionBarModel");
        let hover_area            = component::ShapeView::new(&logger,scene);
        let icon_freeze           = component::ShapeView::new(&logger,scene);
        let icon_visibility       = component::ShapeView::new(&logger,scene);
        let icon_skip             = component::ShapeView::new(&logger,scene);

        let display_object        = display::object::Instance::new(&logger);
        let icons                 = display::object::Instance::new(&logger);

        let size                  = default();

        let all_shapes            = compound_shape::CompoundShapeEvents::new();

        all_shapes.add_sub_shape(&hover_area);
        all_shapes.add_sub_shape(&icon_freeze);
        all_shapes.add_sub_shape(&icon_visibility);
        all_shapes.add_sub_shape(&icon_skip);


        Self{hover_area,icons,display_object,size,icon_freeze,icon_visibility,
             icon_skip,all_shapes}.init()
    }

    fn init(self) -> Self {
        self.add_child(&self.hover_area);

        self.add_child(&self.icons);
        self.icons.add_child(&self.icon_freeze);
        self.icons.add_child(&self.icon_skip);
        self.icons.add_child(&self.icon_visibility);

        // Default state s hidden.
        self.hide();
        self
    }

    fn set_size(&self, size:Vector2) {
        self.size.set(size);
        self.hover_area.shape.size.set(size);

        self.icons.set_position_x(-size.x/2.0);
        let icon_size = Vector2::new(size.y, size.y);

        self.icon_skip.shape.size.set(icon_size);
        self.icon_skip.mod_position(|p| p.x = 0.5 * icon_size.x);

        self.icon_visibility.shape.size.set(icon_size);
        self.icon_visibility.mod_position(|p| p.x  = 2.0 * icon_size.x);

        self.icon_freeze.shape.size.set(icon_size);
        self.icon_freeze.mod_position(|p| p.x  = 3.5 * icon_size.x);

    }

    fn show(&self) {
        self.add_child(&self.icons);
    }

    fn hide(&self) {
        self.icons.unset_parent();
    }
}

impl display::Object for Model {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}



// ==================
// === Action Bar ===
// ==================

/// UI for executing actions on a node.
///
/// Layout
/// ------
/// ```text
///    / ----------------------------- \
///    | <ico1> <ico2> <ico3>          |
///    \ ----------------------------- /
///
/// ```
#[derive(Clone,CloneRef,Debug)]
pub struct ActionBar {
    model : Rc<Model>,
    pub frp    : Frp
}

impl ActionBar {

    /// Constructor.
    pub fn new(app:&Application) -> Self {
        let model = Rc::new(Model::new(app));
        let frp   = Frp::new_network();
        ActionBar {model,frp}.init_frp()
    }

    fn init_frp(self) -> Self {
        let network = &self.frp.network;
        let frp     = &self.frp;
        let model   = &self.model;

        let hover_area     = &model.hover_area.events;
        let compound_shape = &model.all_shapes.frp;

        frp::extend! { TRACE_ALL network


            // === Input Processing ===

            eval  frp.set_size ((size)   model.set_size(*size));
            eval_ frp.hide_icons ( model.hide() );
            eval_ frp.show_icons ( model.show() );


            // === Mouse Interactions ===

            eval_ compound_shape.mouse_over (model.show());
            eval_ compound_shape.mouse_out (model.hide());


            // === Icon Actions ===
            frp.source.action_skip_clicked      <+ model.icon_skip.events.mouse_down;
            frp.source.action_freeze_clicked    <+ model.icon_freeze.events.mouse_down;
            frp.source.action_visbility_clicked <+ model.icon_visibility.events.mouse_down;

        }
        self
    }
}

impl display::Object for ActionBar {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object()
    }
}
