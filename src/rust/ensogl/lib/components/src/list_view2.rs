//! ListView EnsoGL Component.
//!
//! ListView a displayed list of entries with possibility of selecting one and "choosing" by
//! clicking or pressing enter - similar to the HTML `<select>`.



use enso_frp as frp;
use ensogl_core::application;
use ensogl_core::application::Application;
use ensogl_core::application::shortcut;
use ensogl_core::Animation;
use ensogl_core::display;
use ensogl_core::display::scene::layer::LayerId;
use ensogl_core::display::shape::*;
use ensogl_core::display::layout::alignment::Alignment;
use ensogl_theme as theme;
use ensogl_core::data::color;
pub use crate::list_view::entry;
pub use crate::list_view::entry::Entry;

use crate::prelude::*;
use crate::shadow;



// ==============
// === Shapes ===
// ==============

// === Constants ===

/// The size of shadow under element. It is not counted in the component width and height.
pub const SHADOW_PX:f32 = 10.0;
const SHAPE_PADDING:f32 = 5.0;


// === Selection ===

/// The selection rectangle shape.
pub mod selection {
    use ensogl_theme::application::searcher::selection::padding;

    use super::*;

    /// The corner radius in pixels.
    pub const CORNER_RADIUS_PX:f32 = 12.0;

    ensogl_core::define_shape_system! {
        (style:Style) {
            let sprite_width  : Var<Pixels> = "input_size.x".into();
            let sprite_height : Var<Pixels> = "input_size.y".into();
            let padding_in_x = style.get_number(padding::horizontal);
            let padding_in_y = style.get_number(padding::vertical);
            let width        = sprite_width  - 2.0.px() * SHAPE_PADDING + 2.0.px() * padding_in_x;
            let height       = sprite_height - 2.0.px() * SHAPE_PADDING + 2.0.px() * padding_in_y;
            let color        = style.get_color(ensogl_theme::widget::list_view::highlight);
            let rect         = Rect((&width,&height)).corners_radius(CORNER_RADIUS_PX.px());
            let shape        = rect.fill(color);
            shape.into()
        }
    }
}


// === Background ===

/// The default list view background.
pub mod background {
    use super::*;

    /// The corner radius in pixels.
    pub const CORNER_RADIUS_PX:f32 = selection::CORNER_RADIUS_PX;

    ensogl_core::define_shape_system! {
        below = [selection];
        (style:Style) {
            let sprite_width  : Var<Pixels> = "input_size.x".into();
            let sprite_height : Var<Pixels> = "input_size.y".into();
            let width         = sprite_width  - SHADOW_PX.px() * 2.0 - SHAPE_PADDING.px() * 2.0;
            let height        = sprite_height - SHADOW_PX.px() * 2.0 - SHAPE_PADDING.px() * 2.0;
            let color         = style.get_color(theme::widget::list_view::background);
            let rect          = Rect((&width,&height)).corners_radius(CORNER_RADIUS_PX.px());
            let shape         = rect.fill(color);
            let shadow        = shadow::from_shape(rect.into(),style);
            (shadow + shape).into()
        }
    }
}


// === Placeholder ===

pub mod placeholder {
    use super::*;

    /// The corner radius in pixels.
    pub const CORNER_RADIUS_PX:f32 = selection::CORNER_RADIUS_PX;

    ensogl_core::define_shape_system! {
        below = [selection];
        (style:Style) {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let rect  = Rect((&width,&height));
            let shape = rect.fill(color::Rgba::red());
            shape.into()
        }
    }
}



// =============
// === Model ===
// =============

/// Information about displayed fragment of entries list.
#[derive(Copy,Clone,Debug,Default)]
struct View {
    position_y : f32,
    size       : Vector2<f32>,
}


#[derive(Clone,Copy,Debug)]
pub enum Length {
    ElementCount(usize),
    Constant(f32)
}

impl Default for Length {
    fn default() -> Self {
        Self::ElementCount(10)
    }
}

#[derive(Clone,Copy,Debug)]
pub struct Size {
    pub length : Length,
    pub width  : f32,
}

impl Default for Size {
    fn default() -> Self {
        let length = default();
        let width  = 100.0;
        Self {length,width}
    }
}


// ===========
// === FRP ===
// ===========

ensogl_core::define_endpoints! {
    <E>
    Input {
        /// Move selection one position up.
        move_selection_up(),
        /// Move selection page up (jump over all visible entries).
        move_selection_page_up(),
        /// Move selection to the first argument.
        move_selection_to_first(),
        /// Move selection one position down.
        move_selection_down(),
        /// Move selection page down (jump over all visible entries).
        move_selection_page_down(),
        /// Move selection to the last argument.
        move_selection_to_last(),
        /// Chose the currently selected entry.
        chose_selected_entry(),
        /// Deselect all entries.
        deselect_entries(),

        set_size     (Size),
        scroll_jump  (f32),
        // set_entries  (entry::provider::Any<E>),
        select_entry (entry::Id),
        chose_entry  (entry::Id),

        set_entry ((entry::Id,Rc<Option<E>>)),
    }

    Output {
        selected_entry  (Option<entry::Id>),
        chosen_entry    (Option<entry::Id>),
        size            (Vector2<f32>),
        scroll_position (f32),
        get_entries     (Vec<entry::Id>),
    }
}


#[derive(Clone,CloneRef,Debug)]
pub struct Placeholder {
    display_object : display::object::Instance,
    view           : placeholder::View
}

impl Placeholder {
    pub fn new(logger:impl AnyLogger) -> Self {
        let logger         = Logger::new_sub(logger,"placeholder");
        let display_object = display::object::Instance::new(&logger);
        let view           = placeholder::View::new(&logger);
        display_object.add_child(&view);
        view.size.set(Vector2(28.0,28.0));
        view.set_position_xy(Vector2(14.0,14.0));
        Self {display_object,view}
    }
}

impl display::Object for Placeholder {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}


// ================
// === ListView ===
// ================

/// ListView Component.
///
/// This is a displayed list of entries (of any type `E`) with possibility of selecting one and
/// "choosing" by clicking or pressing enter. The basic entry types are defined in [`entry`] module.
#[allow(missing_docs)]
#[derive(Clone,CloneRef,Debug)]
pub struct ListView<E:Entry> {
    model   : Model<E>,
    pub frp : Frp<E::Model>,
}

#[derive(Clone,Copy,Debug)]
pub enum Lazy<T> {
    Requested,
    Known(T)
}


#[derive(Clone,CloneRef,Debug)]
#[clone_ref(bound="E:CloneRef")]
pub enum Slot<E> {
    Placeholder(Placeholder),
    Entry(E)
}



/// The Model of Select Component.
#[derive(Clone,CloneRef,Debug)]
struct Model<E:Entry> {
    logger         : Logger,
    app            : Application,
    selection      : selection::View,
    background     : background::View,
    scroll_area  : display::object::Instance,
    display_object : display::object::Instance,
    entry_model_registry        : Rc<RefCell<HashMap<entry::Id,Lazy<E::Model>>>>,
    slots              : Rc<RefCell<Vec<Slot<E>>>>,
    entry_default_len : Rc<Cell<f32>>,
}

impl<E:Entry> Deref for ListView<E> {
    type Target = Frp<E::Model>;
    fn deref(&self) -> &Self::Target { &self.frp }
}


impl<E:Entry> Model<E> {
    fn new(app:&Application) -> Self {
        let app            = app.clone_ref();
        let logger         = Logger::new("SelectionContainer");
        let display_object = display::object::Instance::new(&logger);
        let scroll_area  = display::object::Instance::new(&logger);
        let background     = background::View::new(&logger);
        let selection      = selection::View::new(&logger);
        let entry_model_registry        = default();
        let entry_default_len = Rc::new(Cell::new(30.0));
        let slots = default();
        display_object.add_child(&background);
        display_object.add_child(&scroll_area);
        scroll_area.add_child(&selection);
        Model{logger,app,selection,background,scroll_area,display_object,entry_model_registry,entry_default_len,slots}
    }

    fn entries_to_be_requested(&self, size:Vector2<f32>) -> Vec<entry::Id> {
        DEBUG!("entries_to_be_requested: {size:?}");
        let length = size.y;

        let mut index  = 0;
        let mut offset = 0.0;

        let mut to_be_requested = Vec::<entry::Id>::new();

        let entry_model_registry : &mut HashMap<entry::Id,Lazy<E::Model>> = &mut *self.entry_model_registry.borrow_mut();

        while offset < length {
            let slot = entry_model_registry.entry(index).or_insert_with(|| {
                to_be_requested.push(index);
                offset += self.entry_default_len.get();
                Lazy::Requested
            });
            match slot {
                Lazy::Requested => {}
                Lazy::Known(entry) => {
                    offset += self.entry_default_len.get();
                }
            }
            index += 1;
        }

        to_be_requested
    }

    fn update_view(&self, size:Vector2<f32>) {
        DEBUG!("update_view: {size:?}");

        let length = size.y;

        let mut index  = 0;
        let mut offset = 0.0;

        let mut new_entries = Vec::new();

        let entry_model_registry : &mut HashMap<entry::Id,Lazy<E::Model>> = &mut *self.entry_model_registry.borrow_mut();

        while offset < length {
            let slot = entry_model_registry.get(&index);
            match slot {
                None => {
                    DEBUG!("set unknown #{index}, offset {offset}.");
                    let entry = Placeholder::new(&self.logger);
                    offset += self.entry_default_len.get();
                    self.scroll_area.add_child(&entry);
                    entry.set_position_y(-offset);
                    new_entries.push(Slot::Placeholder(entry));
                }
                Some(Lazy::Requested) => {
                    DEBUG!("set requested #{index}, offset {offset}.");
                    let entry = Placeholder::new(&self.logger);
                    offset += self.entry_default_len.get();
                    self.scroll_area.add_child(&entry);
                    entry.set_position_y(-offset);
                    new_entries.push(Slot::Placeholder(entry));
                }

                Some(Lazy::Known(model)) => {
                    DEBUG!("set known #{index}, offset {offset}.");
                    let entry = E::new(&self.app);
                    entry.set_model(model);
                    entry.set_label_layer(&self.app.display.scene().layers.label);
                    offset += self.entry_default_len.get();
                    self.scroll_area.add_child(&entry);
                    entry.set_position_y(-offset + 15.0); // FIXME: label center
                    new_entries.push(Slot::Entry(entry));
                }
            }
            index += 1;
        }

        *self.slots.borrow_mut() = new_entries;
        // to_be_requested
    }

    fn set_display_size(&self, size:Vector2<f32>) {
        // let padding_px = 100.0;
        // let padding         = 2.0 * padding_px + SHAPE_PADDING;
        let padding_offset         = Vector2(SHAPE_PADDING,-SHAPE_PADDING);
        let shadow_offset          = Vector2(SHADOW_PX,-SHADOW_PX);
        let shape_offset           = Vector2(size.x/2.0, -size.y/2.0);
        let position               = shape_offset - shadow_offset - padding_offset;
        // self.background.size.set(size + padding + shadow);
        self.background.size.set(size);
        self.background.set_position_xy(position);
        // self.scroll_area.set_position_x(10.0); // TODO: padding
    }

    fn set_entry(&self, index:entry::Id, entry:Option<&E::Model>) {
        match entry {
            None => {
                self.entry_model_registry.borrow_mut().remove(&index);
            }
            Some(e) => {
                self.entry_model_registry.borrow_mut().insert(index,Lazy::Known(e.clone()));
            },
        }
    }
}

impl<E:Entry> ListView<E>
where E::Model : Default {
    /// Constructor.
    pub fn new(app:&Application) -> Self {
        let frp   = Frp::new();
        let model = Model::new(app);
        ListView {model,frp}.init(app)
    }

    fn init(self, app:&Application) -> Self {
        const MAX_SCROLL:f32           = entry::HEIGHT/2.0;
        const MOUSE_MOVE_THRESHOLD:f32 = std::f32::EPSILON;


        let frp              = &self.frp;
        let network          = &frp.network;

        let size_anim : Animation<Vector2<f32>> = Animation::new(network);

        network.store(&size_anim);


        let model            = &self.model;
        let scene            = app.display.scene();
        let mouse            = &scene.mouse.frp;

        frp::extend! { network
            new_size         <- frp.set_size.map(|size| Vector2(size.width,100.0));
            missing_entries  <- new_size.map(f!((size) model.entries_to_be_requested(*size)));

            eval new_size ((size) model.update_view(*size));

            frp.source.get_entries <+ missing_entries;

            eval size_anim.value((size) model.set_display_size(*size));
            size_anim.target <+ new_size;

            trace missing_entries;
            trace frp.set_entry;

            eval frp.set_entry (((id,entry)) model.set_entry(*id,(**entry).as_ref()));

        }

        self
    }

    // /// Sets the scene layer where the labels will be placed.
    // pub fn set_label_layer(&self, layer:LayerId) {
    //     self.model.entry_model_registry.set_label_layer(layer);
    // }
}

impl<E:Entry> display::Object for ListView<E> {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object
    }
}

impl<E:Entry> application::command::FrpNetworkProvider for ListView<E> {
    fn network(&self) -> &frp::Network {
        &self.frp.network
    }
}

impl<E:Entry> application::View for ListView<E> {
    fn label() -> &'static str { "ListView" }
    fn new(app:&Application) -> Self { ListView::new(app) }
    fn app(&self) -> &Application { &self.model.app }
    fn default_shortcuts() -> Vec<shortcut::Shortcut> {
        use shortcut::ActionType::*;
        (&[ (PressAndRepeat , "up"        , "move_selection_up")
          , (PressAndRepeat , "down"      , "move_selection_down")
          , (Press          , "page-up"   , "move_selection_page_up")
          , (Press          , "page-down" , "move_selection_page_down")
          , (Press          , "home"      , "move_selection_to_first")
          , (Press          , "end"       , "move_selection_to_last")
          , (Press          , "enter"     , "chose_selected_entry")
          ]).iter().map(|(a,b,c)|Self::self_shortcut(*a,*b,*c)).collect()
    }
}
