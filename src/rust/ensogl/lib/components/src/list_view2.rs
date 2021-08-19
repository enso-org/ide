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
use crate::list_view::list::IdAtOffset;



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

ensogl_core::define_endpoints! { [TRACE_ALL]
    <E:(Debug+'static)>
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
        set_scroll (f32),
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
        view.set_position_xy(Vector2(14.0,0.0));
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

impl<E:display::Object> display::Object for Slot<E> {
    fn display_object(&self) -> &display::object::Instance {
        match self {
            Self::Placeholder(t) => &t.display_object,
            Self::Entry(entry)   => entry.display_object()
        }
    }
}

#[derive(Clone,Copy,Debug,Default)]
pub struct SlotRange {
    pub start : usize,
    pub end   : usize
}

impl SlotRange {
    pub fn new(start:usize, end:usize) -> Self {
        Self {start,end}
    }

    pub fn contains(self, index:usize) -> bool {
        index >= self.start && index < self.end
    }
}



/// The Model of Select Component.
#[derive(Clone,CloneRef,Debug)]
struct Model<E:Entry> {
    logger         : Logger,
    app            : Application,
    selection      : selection::View,
    background     : background::View,
    scroll_area    : display::object::Instance,
    display_object : display::object::Instance,
    data           : Rc<RefCell<ModelData<E>>>,
}

#[derive(Debug)]
pub struct ModelData<E:Entry> {
    logger               : Logger,
    app                  : Application,
    scroll_area          : display::object::Instance,
    entry_model_registry : HashMap<entry::Id,Lazy<E::Model>>,
    slots                : Vec<Slot<E>>,
    entry_pool           : Vec<E>,
    placeholder_pool     : Vec<Placeholder>,
    slot_range2          : SlotRange,
    entry_default_len    : f32,
    length               : f32,
    last_entry_id        : Option<entry::Id>,
}

impl<E:Entry> Deref for ListView<E> {
    type Target = Frp<E::Model>;
    fn deref(&self) -> &Self::Target { &self.frp }
}

impl<E:Entry> ModelData<E> {
    fn new(logger:&Logger, app:&Application, scroll_area:&display::object::Instance) -> Self {
        let logger = logger.clone_ref();
        let app = app.clone_ref();
        let scroll_area = scroll_area.clone_ref();
        let entry_model_registry        = default();
        let slots = default();
        let entry_pool = default();
        let placeholder_pool = default();
        let slot_range2 = default();
        let entry_default_len = 30.0;
        let length = default();
        let last_entry_id = default();
        Self {logger,app,scroll_area,entry_model_registry,entry_default_len,slots,entry_pool
            ,placeholder_pool,slot_range2,length,last_entry_id}
    }

    fn entries_to_be_requested(&mut self, size:Vector2<f32>) -> Vec<entry::Id> {
        DEBUG!("entries_to_be_requested: {size:?}");
        let length = size.y;

        let mut index  = 0;
        let mut offset = 0.0;

        let mut to_be_requested = Vec::<entry::Id>::new();
        let entry_default_len = self.entry_default_len;

        while offset < length {
            let slot = self.entry_model_registry.entry(index).or_insert_with(|| {
                to_be_requested.push(index);
                offset += entry_default_len;
                Lazy::Requested
            });
            match slot {
                Lazy::Requested => {}
                Lazy::Known(entry) => {
                    offset += entry_default_len;
                }
            }
            index += 1;
        }

        to_be_requested
    }

    fn set_size(&mut self, size:Vector2<f32>) {
        self.length = size.y;
    }

    fn move_slot_to_pool(&mut self, slot:Slot<E>) {
        slot.unset_parent();
        match slot {
            Slot::Placeholder (t) => self.placeholder_pool.push(t),
            Slot::Entry       (t) => self.entry_pool.push(t),
        }
    }

    fn update_view(&mut self) {

        let scroll = self.scroll_area.position().y;
        let index = (scroll / 30.0).floor() as usize;

        let start = self.slot_range2.start;
        let end   = index.min(self.slot_range2.end);

        for ix in start .. end {
            let slot = self.slots.remove(0);
            self.move_slot_to_pool(slot);
        }

        self.slot_range2.start = index;



        let length = self.length;
        let offset1 =  scroll - (index as f32) * 30.0;
        let mut offset = ((self.slot_range2.end - self.slot_range2.start) as f32 * 30.0 - offset1);
        let mut index = self.slot_range2.end;

        DEBUG!("offset: {offset}");

        while offset < length {
            let slot = self.entry_model_registry.get(&index);
            match slot {
                None => {
                    DEBUG!("set unknown #{index}, offset {offset}.");
                    let entry = Placeholder::new(&self.logger);
                    offset += self.entry_default_len;
                    self.scroll_area.add_child(&entry);
                    entry.set_position_y(-((index + 1) as f32) * 30.0);
                    self.slots.push(Slot::Placeholder(entry));
                }
                Some(Lazy::Requested) => {
                    DEBUG!("set requested #{index}, offset {offset}.");
                    let entry = Placeholder::new(&self.logger);
                    offset += self.entry_default_len;
                    self.scroll_area.add_child(&entry);
                    entry.set_position_y(-((index + 1) as f32) * 30.0);
                    self.slots.push(Slot::Placeholder(entry));
                }

                Some(Lazy::Known(model)) => {
                    DEBUG!("set known #{index}, offset {offset}.");
                    let entry = E::new(&self.app);
                    entry.set_model(model);
                    entry.set_label_layer(&self.app.display.scene().layers.label);
                    offset += self.entry_default_len;
                    self.scroll_area.add_child(&entry);
                    entry.set_position_y(-((index + 1) as f32) * 30.0);
                    // entry.set_position_y(-offset + 15.0); // FIXME: label center
                    self.slots.push(Slot::Entry(entry));
                }
            }
            index += 1;
        }

        self.slot_range2.end = index;
        DEBUG!("{self.slot_range2:?}");

    }

    fn new_entry(&mut self) -> E {
        let entry = self.entry_pool.pop().unwrap_or_else(|| E::new(&self.app));
        entry.set_label_layer(&self.app.display.scene().layers.label);
        entry
    }

    fn new_placeholder(&mut self) -> Placeholder {
        self.placeholder_pool.pop().unwrap_or_else(|| Placeholder::new(&self.logger))
    }

    fn set_entry(&mut self, index:entry::Id, new_entry:Option<&E::Model>) {

        if self.slot_range2.contains(index) {
            DEBUG!("REFRESH THE VIEW!");
            let slot = self.slots[index].clone_ref();
            match slot {
                Slot::Placeholder(placeholder) => {
                    match new_entry {
                        None => {},
                        Some(model) => {
                            let entry = self.new_entry();
                            entry.set_model(model);
                            entry.set_position_y(placeholder.position().y);
                            self.scroll_area.add_child(&entry);
                            self.slots[index] = Slot::Entry(entry);
                            self.move_slot_to_pool(Slot::Placeholder(placeholder));
                        }
                    }
                }
                Slot::Entry(entry) => {
                    match new_entry {
                        None => {
                            let placeholder = self.new_placeholder();
                            placeholder.set_position_y(entry.position().y);
                            self.scroll_area.add_child(&placeholder);
                            self.slots[index] = Slot::Placeholder(placeholder);
                            self.move_slot_to_pool(Slot::Entry(entry));
                        }
                        Some(model) => {
                            entry.set_model(model);
                        }
                    }
                }
            }
        }

        match new_entry {
            None => {
                self.entry_model_registry.remove(&index);
            }
            Some(e) => {
                self.entry_model_registry.insert(index,Lazy::Known(e.clone()));
            },
        }

        self.last_entry_id = Some(self.last_entry_id.unwrap_or(entry::Id::MIN).max(index));
    }

    fn set_scroll(&mut self, scroll:f32) {
        self.scroll_area.set_position_y(scroll);
        self.update_view();
    }
}

impl<E:Entry> Model<E> {
    fn new(app:&Application) -> Self {
        let app            = app.clone_ref();
        let logger         = Logger::new("SelectionContainer");
        let display_object = display::object::Instance::new(&logger);
        let scroll_area  = display::object::Instance::new(&logger);
        let background     = background::View::new(&logger);
        let selection      = selection::View::new(&logger);
        let data = Rc::new(RefCell::new(ModelData::new(&logger,&app,&scroll_area)));
        display_object.add_child(&background);
        display_object.add_child(&scroll_area);
        scroll_area.add_child(&selection);
        Self {logger,app,selection,background,scroll_area,display_object,data}
    }

    fn entries_to_be_requested(&self, size:Vector2<f32>) -> Vec<entry::Id> {
        self.data.borrow_mut().entries_to_be_requested(size)
    }

    fn update_view(&self) {
        self.data.borrow_mut().update_view()
    }

    fn set_size(&self, size:Vector2<f32>) {
        self.data.borrow_mut().set_size(size)
    }

    fn set_display_size(&self, size:Vector2<f32>) {
        // let padding_px = 100.0;
        // let padding         = 2.0 * padding_px + SHAPE_PADDING;
        let padding                = 2.0 * Vector2(SHAPE_PADDING,SHAPE_PADDING);
        let shadow_padding         = 2.0 * Vector2(SHADOW_PX,SHADOW_PX);
        let sprite_size            = size + padding + shadow_padding;
        let padding_offset         = Vector2(SHAPE_PADDING,-SHAPE_PADDING);
        let shadow_offset          = Vector2(SHADOW_PX,-SHADOW_PX);
        let shape_offset           = Vector2(sprite_size.x/2.0, -sprite_size.y/2.0);
        let position               = shape_offset - shadow_offset - padding_offset;
        // self.background.size.set(size + padding + shadow);
        self.background.size.set(sprite_size);
        self.background.set_position_xy(position);
        self.selection.set_position_x(position.x);
        // self.scroll_area.set_position_x(10.0); // TODO: padding
    }

    fn set_entry(&self, index:entry::Id, entry:Option<&E::Model>) {
        self.data.borrow_mut().set_entry(index,entry)
    }

    fn set_scroll(&self, scroll:f32) {
        self.data.borrow_mut().set_scroll(scroll)
    }

    /// Check if the `point` is inside component assuming that it have given `size`.
    fn is_inside(&self, point:Vector2<f32>, size:Vector2<f32>) -> bool {
        let pos_obj_space = self.app.display.scene().screen_to_object_space(&self.background,point);
        let x_range       = (-size.x / 2.0)..=(size.x / 2.0);
        let y_range       = (-size.y / 2.0)..=(size.y / 2.0);
        x_range.contains(&pos_obj_space.x) && y_range.contains(&pos_obj_space.y)
    }

    fn selected_entry_after_jump
    (&self, current_entry:Option<entry::Id>, jump:isize) -> Option<entry::Id> {
        if jump < 0 {
            let current_entry = current_entry?;
            if current_entry == 0 { None }
            else                  { Some(current_entry.saturating_sub(-jump as usize)) }
        } else {
            let max_entry = self.last_entry_id()?;
            Some(current_entry.map_or(0, |id| id+(jump as usize)).min(max_entry))
        }
    }

    fn last_entry_id(&self) -> Option<entry::Id> {
        (*self.data).borrow().last_entry_id
    }

    fn visible_entries_count(&self) -> usize {
        (*self.data).borrow().slots.len()
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

        let scroll_y         = Animation::<f32>::new(network);
        let selection_y      = Animation::<f32>::new(network);
        let selection_height = Animation::<f32>::new(network);


        frp::extend! { network
            new_size         <- frp.set_size.map(|size| Vector2(size.width,80.0));
            missing_entries  <- new_size.map(f!((size) model.entries_to_be_requested(*size)));
            frp.source.size  <+ new_size;

            eval new_size ((size) model.set_size(*size));
            eval new_size ((size) model.update_view());

            frp.source.get_entries <+ missing_entries;

            eval size_anim.value((size) model.set_display_size(*size));
            size_anim.target <+ new_size;

            trace missing_entries;
            trace frp.set_entry;

            eval frp.set_entry (((id,entry)) model.set_entry(*id,(**entry).as_ref()));

            eval frp.set_scroll ((t) model.set_scroll(*t));


            // === Mouse Position ===

            mouse_in <- all_with(&mouse.position,&frp.size,f!((pos,size)
                model.is_inside(*pos,*size)
            ));
            mouse_moved       <- mouse.distance.map(|dist| *dist > MOUSE_MOVE_THRESHOLD );
            mouse_y_in_scroll <- mouse.position.map(f!([model,scene](pos) {
                scene.screen_to_object_space(&model.scroll_area,*pos).y
            }));
            mouse_pointed_entry <- mouse_y_in_scroll.map(f!([model](y)
                Self::entry_at_offset(*y,model.last_entry_id()).entry()
            ));


            // === Selected Entry ===

            frp.source.selected_entry <+ frp.select_entry.map(|id| Some(*id));

            selection_jump_on_one_up  <- frp.move_selection_up.constant(-1);
            selection_jump_on_page_up <- frp.move_selection_page_up.map(f_!([model]
                -(model.visible_entries_count() as isize)
            ));
            selection_jump_on_one_down  <- frp.move_selection_down.constant(1);
            selection_jump_on_page_down <- frp.move_selection_page_down.map(f_!(
                model.visible_entries_count() as isize
            ));
            selection_jump_up   <- any(selection_jump_on_one_up,selection_jump_on_page_up);
            selection_jump_down <- any(selection_jump_on_one_down,selection_jump_on_page_down);
            selected_entry_after_jump_up <- selection_jump_up.map2(&frp.selected_entry,
                f!((jump,id) model.selected_entry_after_jump(*id,*jump))
            );
            selected_entry_after_moving_first <- frp.move_selection_to_first.map(f!([model](())
                model.last_entry_id().is_some().and_option(Some(1))
            ));
            selected_entry_after_moving_last  <- frp.move_selection_to_last.map(f!([model] (())
                model.last_entry_id()
            ));
            selected_entry_after_jump_down <- selection_jump_down.map2(&frp.selected_entry,
                f!((jump,id) model.selected_entry_after_jump(*id,*jump))
            );
            selected_entry_after_move_up <-
                any(selected_entry_after_jump_up,selected_entry_after_moving_first);
            selected_entry_after_move_down <-
                any(selected_entry_after_jump_down,selected_entry_after_moving_last);
            selected_entry_after_move <-
                any(&selected_entry_after_move_up,&selected_entry_after_move_down);
            mouse_selected_entry <- mouse_pointed_entry.gate(&mouse_in).gate(&mouse_moved);

            frp.source.selected_entry <+ selected_entry_after_move;
            frp.source.selected_entry <+ mouse_selected_entry;
            frp.source.selected_entry <+ frp.deselect_entries.constant(None);
            // frp.source.selected_entry <+ frp.set_entries.constant(None);


            // === Chosen Entry ===

            any_entry_selected        <- frp.selected_entry.map(|e| e.is_some());
            any_entry_pointed         <- mouse_pointed_entry.map(|e| e.is_some());
            opt_selected_entry_chosen <- frp.selected_entry.sample(&frp.chose_selected_entry);
            opt_pointed_entry_chosen  <- mouse_pointed_entry.sample(&mouse.down_0).gate(&mouse_in);
            frp.source.chosen_entry   <+ opt_pointed_entry_chosen.gate(&any_entry_pointed);
            frp.source.chosen_entry   <+ frp.chose_entry.map(|id| Some(*id));
            frp.source.chosen_entry   <+ opt_selected_entry_chosen.gate(&any_entry_selected);


            // === Selection Size and Position ===

            target_selection_y <- frp.selected_entry.map(|id|
                id.map_or(0.0,Self::offset_of_entry)
            );
            target_selection_height <- frp.selected_entry.map(f!([](id)
                if id.is_some() {entry::HEIGHT} else {0.0}
            ));
            selection_y.target      <+ target_selection_y;
            selection_height.target <+ target_selection_height;

            selection_sprite_y <- all_with(&selection_y.value,&selection_height.value,
                |y,h| y + (entry::HEIGHT - h) / 2.0
            );
            trace selection_sprite_y;
            eval selection_sprite_y ((y) model.selection.set_position_y(*y));
            selection_size <- all_with(&frp.size,&selection_height.value,f!([](size,height) {
                let width = size.x;
                Vector2(width,*height)
            }));
            trace selection_size;
            eval selection_size ((size) model.selection.size.set(*size));


            // === Scrolling ===

            selection_top_after_move_up <- selected_entry_after_move_up.map(|id|
                id.map(|id| Self::y_range_of_entry(id).end)
            );
            min_scroll_after_move_up <- selection_top_after_move_up.map(|top|
                top.unwrap_or(MAX_SCROLL)
            );
            scroll_after_move_up <- min_scroll_after_move_up.map2(&frp.scroll_position,|min,current|
                current.max(*min)
            );
            selection_bottom_after_move_down <- selected_entry_after_move_down.map(|id|
                id.map(|id| Self::y_range_of_entry(id).start)
            );
            max_scroll_after_move_down <- selection_bottom_after_move_down.map2(&frp.size,
                |y,size| y.map_or(MAX_SCROLL, |y| y + size.y)
            );
            scroll_after_move_down <- max_scroll_after_move_down.map2(&frp.scroll_position,
                |max_scroll,current| current.min(*max_scroll)
            );
            frp.source.scroll_position <+ scroll_after_move_up;
            frp.source.scroll_position <+ scroll_after_move_down;
            frp.source.scroll_position <+ frp.scroll_jump;
            // frp.source.scroll_position <+ frp.set_entries.constant(MAX_SCROLL);
            scroll_y.target            <+ frp.scroll_position;
            eval scroll_y.value ((y) model.set_scroll(-*y));
            // eval frp.set_entries     ((_) {
            //     view_y.set_target_value(MAX_SCROLL);
            //     view_y.skip();
            // });

        }

        self
    }

    // /// Sets the scene layer where the labels will be placed.
    // pub fn set_label_layer(&self, layer:LayerId) {
    //     self.model.entry_model_registry.set_label_layer(layer);
    // }

    /// Y position of entry with given id, relative to scroll area.
    pub fn offset_of_entry(id:entry::Id) -> f32 { -((id + 1) as f32) * 30.0 }

    /// Y range of entry with given id, relative to Entry List position.
    pub fn y_range_of_entry(id:entry::Id) -> Range<f32> {
        let position = Self::offset_of_entry(id);
        (position - entry::HEIGHT / 2.0)..(position + entry::HEIGHT / 2.0)
    }

    /// Get the entry id which lays on given y coordinate.
    pub fn entry_at_offset(y:f32, last_entry:Option<entry::Id>) -> IdAtOffset {
        use IdAtOffset::*;
        let all_entries_start = Self::y_range_of_all_entries(last_entry).start;
        if      y > entry::HEIGHT/2.0 { AboveFirst                                   }
        else if y < all_entries_start { UnderLast                                    }
        else                          { Entry((-y/entry::HEIGHT + 0.5) as entry::Id) }
    }

    /// Y range of all entries in this list, including not displayed.
    pub fn y_range_of_all_entries(last_entry:Option<entry::Id>) -> Range<f32> {
        let start = if let Some(last_entry) = last_entry {
            Self::offset_of_entry(last_entry) - entry::HEIGHT / 2.0
        } else {
            entry::HEIGHT / 2.0
        };
        let end = entry::HEIGHT / 2.0;
        start..end
    }
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
