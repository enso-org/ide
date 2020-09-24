//! Drop Down Menu Component.
use ensogl_core::prelude::*;

use enso_frp as frp;
use enso_frp;
use ensogl_core::application::Application;
use ensogl_core::data::color;
use ensogl_core::display::shape::*;
use ensogl_core::display;
use ensogl_core::gui::component;
use ensogl_gui_list_view as list_view;
use ensogl_gui_list_view::entry::ModelProvider;
use ensogl_text as text;



// =================
// === Constants ===
// =================

/// Invisible dummy color to catch hover events.
const HOVER_COLOR : color::Rgba = color::Rgba::new(1.0,0.0,0.0,0.000_001);
/// The width of the visualisation selection menu.
const MENU_WIDTH  : f32         = 180.0;



// ==============
// === Shapes ===
// ==============

/// Icon that indicates the drop down menu.
pub mod icon {
    use super::*;

    ensogl_core::define_shape_system! {
        (style:Style) {
            let width            = Var::<Pixels>::from("input_size.x");
            let height           = Var::<Pixels>::from("input_size.y");
            let triangle         = Triangle(width,height);
            let triangle_down    = triangle.rotate(Var::<f32>::from(std::f32::consts::PI));
            let color_path       = ensogl_theme::vars::graph_editor::visualization::action_bar::icon::color;
            let icon_color       = style.get_color(color_path);
            let triangle_colored = triangle_down.fill(color::Rgba::from(icon_color));

            triangle_colored.into()
        }
    }
}

/// Invisible rectangular area around the icon
pub mod chooser_hover_area {
    use super::*;

    ensogl_core::define_shape_system! {
        () {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let background           = Rect((&width,&height));
            let background           = background.fill(HOVER_COLOR);
            background.into()
        }
    }
}



// ===========
// === FRP ===
// ===========

ensogl_text::define_endpoints! {
    Input {
        set_entries         (list_view::entry::AnyModelProvider),
        set_icon_size       (Vector2),
        set_icon_padding    (Vector2),
        hide_selection_menu (),
        set_selected        (Option<list_view::entry::Id>),
    }
    Output {
        menu_visible    (bool),
        menu_closed     (),
        chosen_entry    (Option<list_view::entry::Id>),
        icon_mouse_over (),
        icon_mouse_out  (),
    }
}



// =============
// === Model ===
// =============

#[derive(Clone,Debug)]
struct Model {
    logger          : Logger,
    app             : Application,
    display_object  : display::object::Instance,

    icon            : component::ShapeView<icon::Shape>,
    icon_overlay    : component::ShapeView<chooser_hover_area::Shape>,

    label           : text::Area,
    selection_menu  : list_view::ListView,

    content         : RefCell<Option<list_view::entry::SingleMaskedProvider>>,
}

impl Model {
    fn new(app:&Application) -> Self {
        let logger         = Logger::new("visualization_chooser::Model");
        let scene          = app.display.scene().clone_ref();
        let app            = app.clone_ref();
        let display_object = display::object::Instance::new(&logger);
        let icon           = component::ShapeView::new(&logger,&scene);
        let icon_overlay   = component::ShapeView::new(&logger,&scene);
        let selection_menu = list_view::ListView::new(&app);
        let label          = app.new_view::<text::Area>();
        let content        = default();

        Self{logger,app,display_object,icon,
            icon_overlay,selection_menu,label,content}.init()
    }

    fn init(self) -> Self {
        self.add_child(&self.icon);
        self.add_child(&self.icon_overlay);
        self.add_child(&self.label);

        // FIXME: Use a string from some settings/i10n source.
        self.set_label("None");

        // Clear default parent and hide again.
        self.show_selection_menu();
        self.hide_selection_menu();

        self
    }

    fn set_label(&self, label:&str) {
        self.label.set_cursor(&default());
        self.label.select_all();
        self.label.insert(label);
        self.label.remove_all_cursors();
    }

    fn show_selection_menu(&self) {
        self.add_child(&self.selection_menu);
    }

    fn hide_selection_menu(&self) {
        self.selection_menu.unset_parent()
    }

    fn get_content_item(&self, id:Option<list_view::entry::Id>) -> Option<list_view::entry::Model> {
        self.content.borrow().as_ref()?.get(id?)
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

/// UI entity that shows a button that opens a list of visualisations that can be selected from.
#[allow(missing_docs)]
#[derive(Clone,CloneRef,Debug)]
pub struct DropDownMenu {
        model : Rc<Model>,
    pub frp   : Frp,
}

impl DropDownMenu {
    /// Constructor.
    pub fn new(app:&Application) -> Self {
        let frp   = Frp::new_network();
        let model = Rc::new(Model::new(app));
        Self {frp,model}.init(app)
    }

    fn init(self, app:&Application) -> Self {
        let network = &self.frp.network;
        let frp     = &self.frp;
        let model   = &self.model;

        let scene   = app.display.scene();
        let mouse   = &scene.mouse.frp;

        frp::extend! { network


            // === Simple Input Processing ===

            eval  frp.input.set_entries ([model](entries) {
                let entries:list_view::entry::SingleMaskedProvider=entries.clone().into();
                model.content.set(entries.clone());
                let item_count = entries.entry_count();

                let line_height = list_view::entry::HEIGHT;
                let menu_size   = Vector2::new(MENU_WIDTH,line_height * item_count as f32);
                model.selection_menu.frp.resize.emit(menu_size);

                 let entries:list_view::entry::AnyModelProvider=entries.into();
                 model.selection_menu.frp.set_entries.emit(entries);
            });


            // === Layouting ===

            icon_size <- all(frp.input.set_icon_size,frp.input.set_icon_padding);
            eval icon_size (((size,padding)) {
                model.icon.shape.sprite.size.set(size-2.0*padding);
            });

            resiz_menu <- all(model.selection_menu.size,frp.input.set_icon_size);
            eval resiz_menu (((menu_size,icon_size)) {
                // Align the top of the menu to the bottom of the icon.
                model.selection_menu.set_position_y(-menu_size.y/2.0-icon_size.y/2.0);
                // Align the right of the menu to the right of the icon.
                model.selection_menu.set_position_x(-menu_size.x/2.0+icon_size.x/2.0);
            });

            label_position <- all(model.label.frp.width,frp.input.set_icon_size);
            eval label_position (((text_width,icon_size)) {
                model.label.set_position_x(-text_width-icon_size.x/2.0);
                // Adjust for text offset, so this appears more centered.
                model.label.set_position_y(0.25 * icon_size.y);
            });

            overlay_size <- all(model.label.frp.width,frp.input.set_icon_size);
            eval overlay_size ([model]((text_width,icon_size)) {
                let size = Vector2::new(text_width + icon_size.x,icon_size.y);
                model.icon_overlay.shape.sprite.size.set(size);
                model.icon_overlay.set_position_x(-text_width/2.0);
            });


             // === Menu State ===

            selection_menu_visible         <- source::<bool>();
            selection_menu_visible_sampler <- selection_menu_visible.sampler();

            hide_menu <- source::<()>();
            show_menu <- source::<()>();

            eval_ hide_menu ([frp,model,selection_menu_visible]{
                model.hide_selection_menu();
                selection_menu_visible.emit(false);
                frp.source.menu_visible.emit(false);
                frp.source.menu_closed.emit(());
            });

             eval_ show_menu ([frp,model,selection_menu_visible]{
                model.show_selection_menu();
                selection_menu_visible.emit(true);
                frp.source.menu_visible.emit(true);
            });


            // === Selection ===

            eval model.selection_menu.chosen_entry([frp,hide_menu,model](entry_id) {
                hide_menu.emit(());
                if let Some(entry_id) = entry_id {
                    let unmasked_id = model.content.borrow().as_ref().map(|content| {
                        content.unmasked_index(*entry_id)
                    });
                    if let Some(unmasked_id) = unmasked_id {
                        frp.source.chosen_entry.emit(unmasked_id);
                          frp.input.set_selected(unmasked_id);
                    };
                }
            });

            eval frp.input.set_selected([model](entry_id) {
                if let Some(entry_id) = entry_id {
                    if let Some(content) = model.content.borrow().as_ref() {
                        content.clear_mask();
                        if let Some(item) = model.get_content_item(Some(*entry_id)) {
                            model.set_label(&item.label)
                        };
                        content.set_mask(*entry_id);
                        let entries:list_view::entry::AnyModelProvider=content.clone().into();
                        model.selection_menu.frp.set_entries.emit(entries);
                    };
                };
            });

            // === Menu Toggle Through Mouse Interaction ===

            icon_hovered <- source::<bool>();
            eval_ model.icon_overlay.events.mouse_over ( icon_hovered.emit(true) );
            eval_ model.icon_overlay.events.mouse_out ( icon_hovered.emit(false) );

            frp.source.icon_mouse_over <+ model.icon_overlay.events.mouse_over;
            frp.source.icon_mouse_out  <+ model.icon_overlay.events.mouse_out;

            eval_ model.icon_overlay.events.mouse_down ([show_menu,hide_menu]{
                if !selection_menu_visible_sampler.value() {
                    show_menu.emit(());
                } else {
                    hide_menu.emit(());
                }
           });


           // === Close Menu ===

           mouse_down        <- mouse.down.constant(());
           mouse_down_remote <- mouse_down.gate_not(&icon_hovered);
           dismiss_menu      <- any(&frp.hide_selection_menu,&mouse_down_remote);
           eval_ dismiss_menu ([hide_menu] {
               hide_menu.emit(());
           });
        }

        self
    }
}

impl display::Object for DropDownMenu {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object
    }
}

