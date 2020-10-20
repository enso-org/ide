//! Drop Down Menu Component.
use ensogl_core::prelude::*;

use enso_frp as frp;
use enso_frp;
use ensogl_core::gui::component::Animation;
use ensogl_core::application::Application;
use ensogl_core::data::color;
use ensogl_core::display::shape::*;
use ensogl_core::display::shape::primitive::StyleWatch;
use ensogl_core::display;
use ensogl_core::gui::component;
use ensogl_gui_list_view as list_view;
use ensogl_gui_list_view::entry::ModelProvider;
use ensogl_text as text;
use ensogl_theme as theme;


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

/// Arrow icon that indicates the drop down menu.
pub mod arrow {
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

ensogl_core::define_endpoints! {
    Input {
        set_entries         (list_view::entry::AnyModelProvider),
        set_icon_size       (Vector2),
        set_icon_padding    (Vector2),
        hide_selection_menu (),
        set_selected        (Option<list_view::entry::Id>),
        set_menu_offset_y   (f32),
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

    icon            : component::ShapeView<arrow::Shape>,
    icon_overlay    : component::ShapeView<chooser_hover_area::Shape>,

    label           : text::Area,
    selection_menu  : list_view::ListView,

    // `SingleMaskedProvider` allows us to hide the selected element.
    content         : RefCell<Option<list_view::entry::SingleMaskedProvider>>,
}

impl Model {
    fn new(app:&Application) -> Self {
        let logger         = Logger::new("visualization_chooser::Model");
        let scene          = app.display.scene();
        let app            = app.clone_ref();
        let display_object = display::object::Instance::new(&logger);
        let icon           = component::ShapeView::new(&logger,scene);
        let icon_overlay   = component::ShapeView::new(&logger,scene);
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

        // Clear default parent and hide again.
        self.show_selection_menu();

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

    fn get_content_item_count(&self) -> usize {
        match self.content.borrow().as_ref() {
            Some(content) => content.entry_count(),
            None          => 0,
        }
    }

    /// Transform index of an element visible in the menu, to the index of the all the objects,
    /// accounting for the removal of the selected item.
    ///
    /// Example:
    /// Widget state: Selected [B], menu content [A, C]
    /// Item list                [A, B,  C]
    /// Unmasked index           [0, 1,  2]
    /// Masked indices           [0, na, 1]
    fn get_unmasked_index(&self, ix:Option<usize>) -> Option<usize> {
        Some(self.content.borrow().as_ref()?.unmasked_index(ix?))
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


            // === Input Processing ===

            eval frp.input.set_entries ([model](entries) {
                let entries:list_view::entry::SingleMaskedProvider = entries.clone_ref().into();
                model.content.set(entries.clone());
                let entries:list_view::entry::AnyModelProvider = entries.into();
                model.selection_menu.frp.set_entries.emit(entries);
            });


            // === Layouting ===

            let menu_height = Animation::<f32>::new(&network);

            eval menu_height.value ([model](height) {
                model.selection_menu.frp.resize.emit(Vector2::new(MENU_WIDTH,*height));
                if *height <= 0.0 {
                    model.hide_selection_menu();
                }
            });

            icon_size <- all(frp.input.set_icon_size,frp.input.set_icon_padding);
            eval icon_size (((size,padding)) {
                model.icon.shape.sprite.size.set(size-2.0*padding);
            });

            resize_menu <- all(model.selection_menu.size,frp.input.set_icon_size,frp.input.set_menu_offset_y);
            eval resize_menu (((menu_size,icon_size,menu_offset_y)) {
                // Align the top of the menu to the bottom of the icon.
                model.selection_menu.set_position_y(-menu_size.y/2.0-icon_size.y/2.0-menu_offset_y);
                // Align the right of the menu to the right of the icon.
                let offfset_y = -menu_size.x/2.0+icon_size.x/2.0-list_view::component::SHADOW_PX/2.0;
                model.selection_menu.set_position_x(offfset_y);
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

            hide_menu <- source::<()>();
            show_menu <- source::<()>();

            eval_ hide_menu ([model,frp,menu_height]{
                model.selection_menu.deselect_entries.emit(());
                frp.source.menu_visible.emit(false);
                frp.source.menu_closed.emit(());
                /// The following line is a workaround for #815.
                /// If we end at 0.0 the `ListView` will still display the first
                /// content item. This avoids the slowdown close to 0.0, so we can
                /// manually remove the `ListView` from the scene at 0.0.
                /// See #815
                menu_height.set_target_value(-20.0);
            });

             eval_ show_menu ([frp,model,menu_height]{
                let item_count    = model.get_content_item_count();
                let line_height   = list_view::entry::HEIGHT;
                let target_height = line_height * item_count as f32;
                model.show_selection_menu();
                menu_height.set_target_value(target_height);
                frp.source.menu_visible.emit(true);
            });


            // === Selection ===

            eval model.selection_menu.chosen_entry([frp,hide_menu,model](entry_id) {
                hide_menu.emit(());
                let unmasked_id = model.get_unmasked_index(*entry_id);
                if let Some(unmasked_id) = unmasked_id {
                    frp.source.chosen_entry.emit(unmasked_id);
                    frp.input.set_selected(unmasked_id);
                };
            });

            eval frp.input.set_selected([model](entry_id) {
                if let Some(entry_id) = entry_id {
                    if let Some(content) = model.content.borrow().as_ref() {
                        // We get an external item index, so we operate on all items, thus we
                        // clear the mask.
                        content.clear_mask();
                        if let Some(item) = model.get_content_item(Some(*entry_id)) {
                            model.set_label(&item.label)
                        };
                        // Remove selected item from menu list
                        content.set_mask(*entry_id);
                        // Update menu content.
                        let entries:list_view::entry::AnyModelProvider = content.clone().into();
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


            let icon_mouse_down = model.icon_overlay.events.mouse_down.clone_ref();
            visibility_on_mouse_down <- frp.source.menu_visible.sample(&icon_mouse_down) ;

            eval visibility_on_mouse_down ([show_menu,hide_menu](is_visible){
                if !is_visible {
                    show_menu.emit(());
                } else {
                    hide_menu.emit(());
                }
            });


           // === Close Menu ===

           mouse_down        <- mouse.down.constant(());
           mouse_down_remote <- mouse_down.gate_not(&icon_hovered);
           dismiss_menu      <- any(&frp.hide_selection_menu,&mouse_down_remote);
           eval_ dismiss_menu ( hide_menu.emit(()) );
        }

        // FIXME : StyleWatch is unsuitable here, as it was designed as an internal tool for
        // shape system (#795)
        let styles     = StyleWatch::new(&app.display.scene().style_sheet);
        let text_color = styles.get_color(theme::vars::widget::list_view::text::color);
        model.label.set_default_color(color::Rgba::from(text_color));

        self
    }
}

impl display::Object for DropDownMenu {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object
    }
}
