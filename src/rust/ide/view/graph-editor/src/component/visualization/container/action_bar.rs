//! Definition of the QuickActionBar component.

use crate::prelude::*;

use crate::component::node;
use crate::component::visualization::container::visualization_chooser;
use crate::component::visualization;


use enso_frp as frp;
use enso_frp;
use ensogl::application::Application;
use ensogl::data::color;
use ensogl::display::shape::*;
use ensogl::display::traits::*;
use ensogl::display;
use ensogl::gui::component;
use ensogl_text as text;



// =================
// === Constants ===
// =================

const HOVER_COLOR    : color::Rgba = color::Rgba::new(1.0,0.0,0.0,0.000_001);



// ===============
// === Shapes  ===
// ===============

/// Invisible rectangular area that can be hovered.
pub mod hover_area {
    use super::*;

    ensogl::define_shape_system! {
        () {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let background    = Rect((&width,&height));
            let background    = background.fill(HOVER_COLOR);
            background.into()
        }
    }
}

/// Invisible rectangular area that can be hovered.
/// Note: needs to be an extra shape for sorting purposes.
pub mod background {
    use super::*;

    ensogl::define_shape_system! {
        () {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let radius             = node::NODE_SHAPE_RADIUS.px() ;
            let background_rounded = Rect((&width,&height)).corners_radius(&radius);
            let background_sharp   = Rect((&width,&height/2.0)).translate_y(-&height/4.0);
            let background         = background_rounded + background_sharp;
            let fill_color         = color::Rgba::from(color::Lcha::new(0.1,0.013,0.18,0.6));
            let background         = background.fill(fill_color);
            background.into()
        }
    }
}



// ===========
// === Frp ===
// ===========

ensogl_text::define_endpoints! {
    Input {
        set_label (String),
        set_size (Vector2),
        show_icons (),
        hide_icons (),
        set_visualization_alternatives (Vec<visualization::Path>),
    }
    Output {
        visualisation_selection  (Option<visualization::Path>),
        mouse_over   (),
        mouse_out    (),
    }
}



// ==============================
// === Quick Action Bar Model ===
// ==============================

#[derive(Clone,CloneRef,Debug)]
struct Model {
    hover_area                   : component::ShapeView<hover_area::Shape>,
    visualization_chooser        : visualization_chooser::VisualisationChooser,
    background                   : component::ShapeView<background::Shape>,
    visualisation_chooser_label  : text::Area,

    display_object        : display::object::Instance,

    size                  : Rc<Cell<Vector2>>,
}

impl Model {
    fn new(app:&Application) -> Self {
        let scene                         = app.display.scene();
        let logger                        = Logger::new("ActionBarModel");
        let background                    = component::ShapeView::new(&logger,scene);
        let hover_area                    = component::ShapeView::new(&logger,scene);
        let visualization_chooser         = visualization_chooser::VisualisationChooser::new(&app);
        let visualisation_chooser_label   = app.new_view::<text::Area>();

        let display_object                = display::object::Instance::new(&logger);
        let size                          = default();

        Model {
            hover_area,
            visualization_chooser,
            visualisation_chooser_label,
            display_object,
            size,
            background,
        }.init()
    }

    fn init(self) -> Self {
        self.add_child(&self.hover_area);

        self.set_label("None");

        self.visualisation_chooser_label.frp.set_default_color.emit(color::Rgba::new(1.0,1.0,1.0,1.0));

        // Remove default parent, then hide icons.
        self.show();
        self.hide();
        self
    }

    fn set_size(&self, size:Vector2) {
        self.size.set(size);
        self.hover_area.shape.size.set(size);
        self.background.shape.size.set(size);

        let height        = size.y;
        let width         = size.x;
        let right_padding = height / 2.0;
        self.visualization_chooser.frp.set_icon_size(Vector2::new(height,height));
        self.visualization_chooser.frp.set_icon_padding(Vector2::new(height/3.0,height/3.0));
        self.visualization_chooser.set_position_x((width/2.0) - right_padding);

        self.visualisation_chooser_label.set_position_y(0.25 * height);

    }

    fn set_label(&self, label:&str) {
        self.visualisation_chooser_label.set_cursor(&default());
        self.visualisation_chooser_label.select_all();
        self.visualisation_chooser_label.insert(label);
        self.visualisation_chooser_label.remove_all_cursors();
    }

    fn show(&self) {
        self.add_child(&self.visualization_chooser);
        self.add_child(&self.background);
        self.add_child(&self.visualisation_chooser_label);
    }

    fn hide(&self) {
        self.visualization_chooser.unset_parent();
        self.background.unset_parent();
        self.visualisation_chooser_label.unset_parent();
        self.visualization_chooser.frp.hide_selection_menu.emit(());
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

/// UI for executing actions on a node. Consists of label indicating the active visualization
/// and a drop-down menu for selecting a new visualisation.
/// Layout
/// ------
/// ```text
///     / ---------------------------- \
///    |            <label>         V   |
///    |--------------------------------|
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
        let model   = Rc::new(Model::new(app));
        let frp     = Frp::new_network();
        ActionBar {model,frp}.init_frp()
    }

    fn init_frp(self) -> Self {
        let network = &self.frp.network;
        let frp     = &self.frp;
        let model   = &self.model;

        let hover_area             = &model.hover_area.events;
        let visualization_chooser  = &model.visualization_chooser.frp;

        frp::extend! { network


            // === Input Processing ===
            eval frp.set_size ((size)   model.set_size(*size));
            eval frp.set_label ((label) model.set_label(label));
            eval_ frp.hide_icons ( model.hide() );
            eval_ frp.show_icons ( model.show() );

            eval frp.input.set_visualization_alternatives ((alternatives){
                visualization_chooser.input.set_alternatives.emit(alternatives);
            });


            // === Additional Layouting ===
            eval model.visualisation_chooser_label.width ((width) {
                model.visualisation_chooser_label.set_position_x(-width/2.0);
            });


            // === Mouse Interactions ===
            any_component_over <- any(&hover_area.mouse_over,&visualization_chooser.icon_mouse_over);
            any_component_out  <- any(&hover_area.mouse_out,&visualization_chooser.icon_mouse_out);

            any_hovered <- source::<bool>();
            eval_ any_component_over ( any_hovered.emit(true)  );
            eval_ any_component_out  ( any_hovered.emit(false) );

            eval_ any_component_over (model.show());

            mouse_out_no_menu <- any_component_out.gate_not(&visualization_chooser.menu_visible);
            remote_click      <- visualization_chooser.menu_closed.gate_not(&any_hovered);
            hide              <- any(mouse_out_no_menu,remote_click);
            eval_ hide (model.hide());

            frp.source.visualisation_selection <+ visualization_chooser.selected_visualization;
        }
        self
    }
}

impl display::Object for ActionBar {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object()
    }
}
