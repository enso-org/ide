//! ListView EnsoGL Component.
//!
//! ListView a displayed list of entries with possibility of selecting one and "choosing" by
//! clicking or pressing enter - similar to the HTML `<select>`.

pub mod entry;

use crate::prelude::*;
use crate::shadow;

use enso_frp as frp;
use ensogl_core::application;
use ensogl_core::application::Application;
use ensogl_core::application::shortcut;
use ensogl_core::display;
use ensogl_core::display::shape::*;
use ensogl_core::DEPRECATED_Animation;
use ensogl_theme as theme;
use ensogl_core::data::color;
use crate::scroll_area::ScrollArea;
use crate::selector;


#[derive(Debug,Copy,Clone)]
pub enum SelectionMethod { Hover, Click }

impl Default for SelectionMethod {
    fn default() -> Self {
        SelectionMethod::Hover
    }
}



// ==========================
// === Shapes Definitions ===
// ==========================

// === Constants ===

/// The size of shadow under element. It is not counted in the component width and height.
pub const SHADOW_PX:f32 = 10.0;
const SHAPE_PADDING:f32 = 5.0;
pub const CORNER_RADIUS_PX:f32 = 6.0;


// === Selection ===

pub mod selection {
    use super::*;

    ensogl_core::define_shape_system! {
        (style:Style, color_rgba:Vector4) {
            let color = Var::<color::Rgba>::from(color_rgba);
            let sprite_width  : Var<Pixels> = "input_size.x".into();
            let sprite_height : Var<Pixels> = "input_size.y".into();
            let padding_inner_x = style.get_number(ensogl_theme::application::searcher::selection::padding::horizontal);
            let padding_inner_y = style.get_number(ensogl_theme::application::searcher::selection::padding::vertical);
            let width         = sprite_width - 2.0.px() * SHAPE_PADDING + 2.0.px() * padding_inner_x;
            let height        = sprite_height + 2.0.px() * padding_inner_y;
            let rect          = Rect((&width,&height)).corners_radius(CORNER_RADIUS_PX.px());
            let shape         = rect.fill(color);
            shape.into()
        }
    }
}


// === Background ===

mod background {
    use super::*;

    ensogl_core::define_shape_system! {
        below = [selection];
        (style:Style) {
            let sprite_width  : Var<Pixels> = "input_size.x".into();
            let sprite_height : Var<Pixels> = "input_size.y".into();
            let width         = sprite_width - SHADOW_PX.px() * 2.0 - SHAPE_PADDING.px() * 2.0;
            let height        = sprite_height - SHADOW_PX.px() * 2.0 - SHAPE_PADDING.px() * 2.0;
            let color         = style.get_color(theme::widget::list_view::background);
            let rect          = Rect((&width,&height)).corners_radius(CORNER_RADIUS_PX.px());
            let shape         = rect.fill(color);

            let shadow  = shadow::from_shape(rect.into(),style);

            (shadow + shape).into()
        }
    }
}


// === IO Rect ===

/// Utility shape that is invisible but provides mouse input. Fills the whole sprite.
pub mod io_rect {
    use super::*;

    ensogl_core::define_shape_system! {
        () {
            let sprite_width  : Var<Pixels> = "input_size.x".into();
            let sprite_height : Var<Pixels> = "input_size.y".into();

            let rect  = Rect((&sprite_width,&sprite_height));
            let shape = rect.corners_radius(CORNER_RADIUS_PX.px()).fill(HOVER_COLOR);

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

impl View {
    fn y_range(&self) -> Range<f32> {
        let start = self.position_y - self.size.y;
        let end   = self.position_y;
        start..end
    }
}

/// The Model of Select Component.
#[derive(Clone,CloneRef,Debug)]
struct Model {
    app            : Application,
    entries        : entry::List,
    selection      : selection::View,
    background     : background::View,
    scroll_area    : ScrollArea,
    io_rect        : io_rect::View,
    display_object : display::object::Instance,
}

impl Model {
    fn new(app:&Application) -> Self {
        let app            = app.clone_ref();
        let scene          = app.display.scene();
        let logger         = Logger::new("SelectionContainer");
        let display_object = display::object::Instance::new(&logger);
        let scroll_area    = ScrollArea::new(&app);
        let entries        = entry::List::new(&logger,&app);
        let background     = background::View::new(&logger);
        let selection      = selection::View::new(&logger);
        let io_rect        = io_rect::View::new(&logger);
        scene.layers.add_shapes_order_dependency::<io_rect::View,selector::shape::background::View>();
        // scene.layers.add_shapes_order_dependency::<selection::View,io_rect::View>();
        // display_object.add_child(&background);
        display_object.add_child(&scroll_area);
        display_object.add_child(&io_rect);
        scroll_area.content.add_child(&entries);
        scroll_area.content.add_child(&selection);
        Model{app,entries,selection,background,scroll_area,io_rect,display_object}
    }

    fn padding(&self) -> f32 {
        // FIXME : StyleWatch is unsuitable here, as it was designed as an internal tool for shape
        // system (#795)
        let styles = StyleWatch::new(&self.app.display.scene().style_sheet);
        styles.get_number(ensogl_theme::application::searcher::padding)
    }

    /// Update the displayed entries list when _view_ has changed - the list was scrolled or
    /// resized.
    fn update_after_view_change(&self, view:&View) {
        let padding_px      = self.padding();
        let padding         = 2.0 * padding_px + SHAPE_PADDING;
        let padding         = Vector2(padding, padding);
        let shadow          = Vector2(2.0 * SHADOW_PX,  2.0 * SHADOW_PX);
        self.background.size.set(view.size + padding + shadow);
        self.selection.set_position_x(view.size.x/2.0);
        self.entries.set_entry_width(view.size.x - self.padding());
        self.scroll_area.resize(view.size);
        self.scroll_area.set_position_x(-view.size.x/2.0);
        self.scroll_area.set_position_y(view.size.y/2.0);
        self.io_rect.size.set(view.size);

        self.entries.set_visible_range(view.y_range());
    }

    fn set_entries(&self, provider:entry::AnyEntryProvider) {
        self.scroll_area.set_content_height(entry::List::total_height(provider.entry_count()));
        self.entries.set_provider(provider);
    }

    fn selected_entry_after_jump
    (&self, current_entry:Option<entry::Id>, jump:isize) -> Option<entry::Id> {
        if jump < 0 {
            let current_entry = current_entry?;
            if current_entry == 0 { None                                    }
            else                  { Some(current_entry.saturating_sub(-jump as usize)) }
        } else {
            let max_entry = self.entries.entry_count().checked_sub(1)?;
            Some(current_entry.map_or(0, |id| id+(jump as usize)).min(max_entry))
        }
    }
}



// ===========
// === FRP ===
// ===========

ensogl_core::define_endpoints! {
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

        resize               (Vector2<f32>),
        scroll_jump          (f32),
        set_entries          (entry::AnyEntryProvider),
        set_selection_method (SelectionMethod),
        select_entry         (entry::Id),
        chose_entry          (entry::Id),

        click        (),
        double_click (),
    }

    Output {
        selected_entry  (Option<entry::Id>),
        chosen_entry    (Option<entry::Id>),
        size            (Vector2<f32>),
        scroll_position (f32),
    }
}



// ========================
// === Select Component ===
// ========================

/// Select Component.
///
/// Select is a displayed list of entries with possibility of selecting one and "chosing" by
/// clicking or pressing enter.
#[allow(missing_docs)]
#[derive(Clone,CloneRef,Debug)]
pub struct ListView {
    model   : Model,
    pub frp : Frp,
}

impl Deref for ListView {
    type Target = Frp;
    fn deref(&self) -> &Self::Target { &self.frp }
}

impl ListView {
    /// Constructor.
    pub fn new(app:&Application) -> Self {
        let frp   = Frp::new();
        let model = Model::new(app);
        ListView {model,frp}.init(app)
    }

    fn init(self, app:&Application) -> Self {
        const MOUSE_MOVE_THRESHOLD:f32 = std::f32::EPSILON;

        let frp              = &self.frp;
        let network          = &frp.network;
        let model            = &self.model;
        let scene            = app.display.scene();
        let mouse            = &scene.mouse.frp;
        let selection_y      = DEPRECATED_Animation::<f32>::new(&network);
        let selection_height = DEPRECATED_Animation::<f32>::new(&network);

        let style = StyleWatchFrp::new(&scene.style_sheet);
        let selection_color           = color::Animation::new(&network);
        let focused_selection_color   = style.get_color(ensogl_theme::widget::list_view::selection::focused);
        let unfocused_selection_color = style.get_color(ensogl_theme::widget::list_view::selection::unfocused);

        frp::extend! { network
            init_selection_color <- source::<()>();
            focused_selection_color <- all(&focused_selection_color,&init_selection_color)._0();
            unfocused_selection_color <- all(&unfocused_selection_color,&init_selection_color)._0();
            selection_color_target_rgba <- frp.focused.switch(&unfocused_selection_color,&focused_selection_color);
            selection_color.target <+ selection_color_target_rgba.map(|c| color::Lcha::from(c));
            eval selection_color.value((c) model.selection.color_rgba.set(color::Rgba::from(c).into()));
            init_selection_color.emit(());

            // eval frp.focused([](focused) warning!(Logger::new(""), "{*focused:?}"));


            // === Mouse Position ===

            mouse_in <- bool(&model.io_rect.events.mouse_out,&model.io_rect.events.mouse_over);
            mouse_moved       <- mouse.distance.map(|dist| *dist > MOUSE_MOVE_THRESHOLD );
            mouse_y_in_scroll <- mouse.position.map(f!([model,scene](pos) {
                scene.screen_to_object_space(&model.scroll_area.content,*pos).y
            }));
            mouse_pointed_entry <- mouse_y_in_scroll.map(f!([model](y)
                entry::List::entry_at_y_position(*y,model.entries.entry_count()).entry()
            ));


            // === Selected Entry ===

            frp.source.selected_entry <+ frp.select_entry.map(|id| Some(*id));

            selection_jump_on_one_up  <- frp.move_selection_up.constant(-1);
            selection_jump_on_page_up <- frp.move_selection_page_up.map(f_!([model]
                -(model.entries.visible_entry_count() as isize)
            ));
            selection_jump_on_one_down  <- frp.move_selection_down.constant(1);
            selection_jump_on_page_down <- frp.move_selection_page_down.map(f_!(
                model.entries.visible_entry_count() as isize
            ));
            selection_jump_up   <- any(selection_jump_on_one_up,selection_jump_on_page_up);
            selection_jump_down <- any(selection_jump_on_one_down,selection_jump_on_page_down);
            selected_entry_after_jump_up <- selection_jump_up.map2(&frp.selected_entry,
                f!((jump,id) model.selected_entry_after_jump(*id,*jump))
            );
            selected_entry_after_moving_first <- frp.move_selection_to_first.map(f!([model](())
                (model.entries.entry_count() > 0).and_option(Some(0))
            ));
            selected_entry_after_moving_last  <- frp.move_selection_to_last.map(f!([model] (())
                model.entries.entry_count().checked_sub(1)
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
            hover_selected_entry <- mouse_pointed_entry.gate(&mouse_in).gate(&mouse_moved).gate_not(&mouse.is_down_primary);

            select_on_hover <- frp.set_selection_method.map(|&method|
                matches!(method, SelectionMethod::Hover));
            select_on_click <- frp.set_selection_method.map(|&method|
                matches!(method, SelectionMethod::Click));

            frp.source.selected_entry <+ selected_entry_after_move;
            frp.source.selected_entry <+ hover_selected_entry.gate(&select_on_hover);
            frp.source.selected_entry <+ frp.chosen_entry.gate(&select_on_click);
            frp.source.selected_entry <+ frp.deselect_entries.constant(None);
            frp.source.selected_entry <+ frp.set_entries.constant(None);

            eval frp.selected_entry((&selection) model.entries.set_selection(selection));


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
                id.map_or(0.0,entry::List::position_y_of_entry)
            );
            target_selection_height <- frp.selected_entry.map(f!([](id)
                if id.is_some() {entry::HEIGHT} else {0.0}
            ));
            eval target_selection_y      ((y) selection_y.set_target_value(*y));
            eval target_selection_height ((h) selection_height.set_target_value(*h));
            eval frp.set_entries         ([selection_y,selection_height](_) {
                selection_y.skip();
                selection_height.skip();
            });
            selectin_sprite_y <- all_with(&selection_y.value,&selection_height.value,
                |y,h| y + (entry::HEIGHT - h) / 2.0
            );
            eval selectin_sprite_y ((y) model.selection.set_position_y(*y));
            selection_size <- all_with(&frp.size,&selection_height.value,f!([](size,height) {
                let width = size.x;
                Vector2(width,*height)
            }));
            eval selection_size ((size) model.selection.size.set(*size));
        }

        frp::extend!{ network

            // === Scrolling ===

            selection_top_after_move_up <- selected_entry_after_move_up.map(|id|
                id.map(|id| entry::List::y_range_of_entry(id).end)
            );
            max_scroll_after_move_up <- selection_top_after_move_up.map(|top|
                -top.unwrap_or(0.0)
            );
            scroll_after_move_up <- max_scroll_after_move_up.map2(&frp.scroll_position,|max,current|
                current.min(*max)
            );
            selection_bottom_after_move_down <- selected_entry_after_move_down.map(|id|
                id.map(|id| entry::List::y_range_of_entry(id).start)
            );
            min_scroll_after_move_down <- selection_bottom_after_move_down.map2(&frp.size,
                |y,size| -y.map_or(0.0, |y| y + size.y)
            );
            scroll_after_move_down <- min_scroll_after_move_down.map2(&frp.scroll_position,
                |min_scroll,current| current.max(*min_scroll)
            );
            model.scroll_area.scroll_to_y <+ scroll_after_move_up;
            model.scroll_area.scroll_to_y <+ scroll_after_move_down;
            model.scroll_area.scroll_to_y <+ frp.scroll_jump;
            model.scroll_area.jump_to_y   <+ frp.set_entries.constant(0.0);
            frp.source.scroll_position    <+ model.scroll_area.scroll_position_y;


            // === Resize ===
            frp.source.size <+ frp.resize;
            // .map(f!([model](size)
            //     size)
            //     //- Vector2(model.padding(),model.padding()))
            // );


            // === Update Entries ===

            view_info <- all_with(&model.scroll_area.scroll_position_y,&frp.size, |y,size|
                View{position_y:-*y,size:*size}
            );
            // This should go before handling mouse events to have proper checking of
            eval view_info ((view) model.update_after_view_change(view));
            _new_entries <- frp.set_entries.map(f!((entries)
                model.set_entries(entries.clone_ref())
            ));
        }

        frp.set_selection_method(SelectionMethod::Hover);

        self
    }
}

impl display::Object for ListView {
    fn display_object(&self) -> &display::object::Instance { &self.model.display_object }
}

impl application::command::FrpNetworkProvider for ListView {
    fn network(&self) -> &frp::Network { &self.frp.network }
}

impl application::View for ListView {
    fn label() -> &'static str { "ListView" }
    fn new(app:&Application) -> Self { ListView::new(app) }
    fn app(&self) -> &Application { &self.model.app }
    fn default_shortcuts() -> Vec<shortcut::Shortcut> {
        use shortcut::ActionType::*;
        (&[ (PressAndRepeat , "up"                , "move_selection_up"        , "focused")
          , (PressAndRepeat , "down"              , "move_selection_down"      , "focused")
          , (Press          , "page-up"           , "move_selection_page_up"   , "focused")
          , (Press          , "page-down"         , "move_selection_page_down" , "focused")
          , (Press          , "home"              , "move_selection_to_first"  , "focused")
          , (Press          , "end"               , "move_selection_to_last"   , "focused")
          , (Press          , "enter"             , "chose_selected_entry"     , "focused")
          , (Press          , "left-mouse-button" , "click"                    , "")
          , (DoublePress    , "left-mouse-button" , "double_click"             , "")
          ]).iter().map(|(a,b,c,d)|Self::self_shortcut_when(*a,*b,*c,*d)).collect()
    }
}
