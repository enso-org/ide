//! Definition of the icon + menu displayed to choose the visualisation for a
//! node.

use crate::prelude::*;

use crate::component::visualization;


use enso_frp as frp;
use enso_frp;
use ensogl::application::Application;
use ensogl::data::color;
use ensogl::display::shape::*;
use ensogl::display::traits::*;
use ensogl::display;
use ensogl::gui::component;
use ensogl_gui_list_view as list_view;


// =================
// === Constants ===
// =================

const HOVER_COLOR    : color::Rgba = color::Rgba::new(1.0,0.0,0.0,0.000_001);
// TODO: Char size based on values in `port.rs`. Should be calculated in based on actual font.
const CHAR_WIDTH     : f32 = 7.224_609_4 * (8.0 / 12.0);

/// Width of the text.
pub fn text_width (label:&str) -> f32 {
    label.chars().count() as f32 * CHAR_WIDTH
}



// ==============
// === Shapes ===
// ==============

pub mod icon {
    use super::*;

    ensogl::define_shape_system! {
    () {
        let width            = Var::<Pixels>::from("input_size.x");
        let height           = Var::<Pixels>::from("input_size.y");
        let triangle         = Triangle(width.clone(),height.clone());
        let triangle_down    = triangle.rotate(Var::<f32>::from(std::f32::consts::PI));

        let fill_color       = color::Rgba::from(color::Lcha::new(0.8,0.013,0.18,1.0));
        let triangle_colored = triangle_down.fill(fill_color);

        triangle_colored.into()
    }
}
}

/// Invisible rectangular area that can be hovered.
/// Note: needs to be an extra shape for sorting purposes.
pub mod chooser_hover_area {
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

// =============================
// === VisualisationPathList ===
// =============================


#[derive(Clone,Debug,Default)]
struct VisualisationPathList {
    pub content: Vec<visualization::Path>
}

impl From<Vec<visualization::Path>> for VisualisationPathList {
    fn from(content:Vec<visualization::Path>) -> Self {
        Self{content}
    }
}

impl list_view::entry::ModelProvider for VisualisationPathList {
    fn entry_count(&self) -> usize {
        self.content.len()
    }

    fn get(&self, id:usize) -> Option<list_view::entry::Model> {
        let path  = self.content.get(id)?;
        let label = format!("{}", path);
        println!("{}", label);
        Some(list_view::entry::Model::new(label))
    }
}

// ===========
// === FRP ===
// ===========

ensogl_text::define_endpoints! {
    Input {
        set_alternatives    (Vec<visualization::Path>),
        set_icon_size       (Vector2),
        set_icon_padding    (Vector2),
        hide_selection_menu (),
    }
    Output {
        menu_open               (bool),
        menu_closed             (),
        selected_visualization  (Option<visualization::Path>),
        icon_mouse_over         (),
        icon_mouse_out          (),
    }
}



// =============
// === Model ===
// =============

#[derive(Clone,Debug)]
struct Model {
    logger         : Logger,
    app            : Application,
    display_object : display::object::Instance,

    icon         : component::ShapeView<icon::Shape>,
    icon_overlay : component::ShapeView<chooser_hover_area::Shape>,

    selection_menu             : list_view::ListView,
    visualization_alternatives : RefCell<Option<VisualisationPathList>>,
    }

impl Model {
    pub fn new(app:&Application) -> Self {
        let logger                     = Logger::new("visualization_chooser::Model");
        let scene                      = app.display.scene().clone_ref();
        let app                        = app.clone_ref();
        let display_object             = display::object::Instance::new(&logger);
        let visualization_alternatives = default();
        let icon                       = component::ShapeView::new(&logger,&scene);
        let icon_overlay               = component::ShapeView::new(&logger,&scene);
        let selection_menu             = list_view::ListView::new(&app);

        Self{logger,app,display_object,visualization_alternatives,icon,
             icon_overlay,selection_menu}.init()
    }

    fn init(self) -> Self {
        self.add_child(&self.icon);
        self.add_child(&self.icon_overlay);
        self
    }

    fn show_selection_menu(&self) {
        self.add_child(&self.selection_menu);
    }

    fn hide_selection_menu(&self) {
        self.selection_menu.unset_parent()
    }
}

impl display::Object for Model {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}


// ============================
// === VisualisationChooser ===
// ============================

#[derive(Clone,CloneRef,Debug)]
pub struct VisualisationChooser {
        model : Rc<Model>,
    pub frp   : Frp,
}

impl VisualisationChooser {
    pub fn new(app:&Application) -> Self {
        let frp           = Frp::new_network();
        let model         = Rc::new(Model::new(app));
        Self {frp,model}.init(app)
    }

    fn init(self, app:&Application) -> Self {
        let network  = &self.frp.network;
        let frp      = &self.frp;
        let model    = &self.model;

        let scene            = app.display.scene();
        let mouse            = &scene.mouse.frp;

        frp::extend! { network

            icon_hovered <- source::<bool>();
            eval_ model.icon_overlay.events.mouse_over ( icon_hovered.emit(true) );
            eval_ model.icon_overlay.events.mouse_out ( icon_hovered.emit(false) );

            selection_menu_visible         <- source::<bool>();
            selection_menu_visible_sampler <- selection_menu_visible.sampler();

            icon_size <- all(frp.input.set_icon_size,frp.input.set_icon_padding);
            eval icon_size (((size,padding)) {
                model.icon.shape.sprite.size.set(size-2.0*padding);
                model.icon_overlay.shape.sprite.size.set(*size);
            });

            frp.source.icon_mouse_over <+ model.icon_overlay.events.mouse_over;
            frp.source.icon_mouse_out  <+ model.icon_overlay.events.mouse_out;


            eval  frp.input.set_alternatives ([model](alternatives) {
                let item_count = alternatives.len();
                let alternatives:VisualisationPathList = alternatives.clone().into();
                model.visualization_alternatives.set(alternatives.clone().into());

                let alternatives:list_view::entry::AnyModelProvider = alternatives.clone().into();
                model.selection_menu.frp.resize.emit(Vector2::new(150.0,20.0 * item_count as f32));
                model.selection_menu.frp.set_entries.emit(alternatives);
            });

            resiz_menu <- all(model.selection_menu.size,frp.input.set_icon_size);
            eval resiz_menu (((menu_size,icon_size)) {
                // Align the top of the menu to the bottom of the icon.
                model.selection_menu.set_position_y(-menu_size.y/2.0-icon_size.y/2.0);
                // Align the right of the menu to the right of the icon.
                model.selection_menu.set_position_x(-menu_size.x/2.0+icon_size.x/2.0);
            });

           eval model.selection_menu.chosen_entry([frp,model,selection_menu_visible](entry_id) {
                if let Some(entry_id) = entry_id {
                    let paths = model.visualization_alternatives.borrow_mut().clone().unwrap();
                    let visualization_path = paths.content.get(*entry_id).cloned();
                    frp.source.selected_visualization.emit(visualization_path);
                }
                model.hide_selection_menu();
                selection_menu_visible.emit(false);
                frp.source.menu_open.emit(false);
                frp.source.menu_closed.emit(());

            });

           eval_ model.icon_overlay.events.mouse_down ([model,selection_menu_visible,frp]{
              if !selection_menu_visible_sampler.value() {
                    model.show_selection_menu();
                    selection_menu_visible.emit(true);
                    frp.source.menu_open.emit(true);
                } else {
                    model.hide_selection_menu();
                    selection_menu_visible.emit(false);
                    frp.source.menu_open.emit(false);
                    frp.source.menu_closed.emit(());
                }
           });

           mouse_down <- mouse.down.constant(());
           mouse_down_remote <- mouse_down.gate_not(&icon_hovered);
           hide_menu <- any(&frp.hide_selection_menu,&mouse_down_remote);
           eval_ hide_menu ([model,selection_menu_visible,frp] {
                model.hide_selection_menu();
                selection_menu_visible.emit(false);
                frp.source.menu_open.emit(false);
                 frp.source.menu_closed.emit(());
           });





        }

        self
    }
}

impl display::Object for VisualisationChooser {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object
    }
}
