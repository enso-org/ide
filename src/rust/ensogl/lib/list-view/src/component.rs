//! Select List Component
use crate::prelude::*;

use crate::entry;

use enso_frp as frp;
use ensogl_core::application;
use ensogl_core::application::Application;
use ensogl_core::application::shortcut;
use ensogl_core::data::color;
use ensogl_core::display;
use ensogl_core::display::Scene;
use ensogl_core::display::shape::*;
use ensogl_core::gui::component;
use ensogl_core::gui::component::Animation;
use ensogl_theme;
use enso_frp::io::keyboard::Key;



// ==========================
// === Shapes Definitions ===
// ==========================

// === Constants ===

/// The distance between sprite and displayed component edge, needed to proper antialiasing.
pub const PADDING_PX:f32 = 1.0;


// === Selection ===

mod selection {
    use super::*;

    pub const CORNER_RADIUS_PX:f32 = entry::PADDING * 2.0;

    ensogl_core::define_shape_system! {
        (style:Style) {
            let sprite_width  : Var<Pixels> = "input_size.x".into();
            let sprite_height : Var<Pixels> = "input_size.y".into();
            let width         = sprite_width  - 2.0.px() * PADDING_PX;
            let height        = sprite_height - 2.0.px() * PADDING_PX;
            let color         = style.get_color(ensogl_theme::vars::widget::list_view::highlight::color);
            let rect          = Rect((&width,&height)).corners_radius(CORNER_RADIUS_PX.px());
            let shape         = rect.fill(color::Rgba::from(color));
            shape.into()
        }
    }
}


// === Background ===

mod background {
    use super::*;

    pub const CORNER_RADIUS_PX:f32 = selection::CORNER_RADIUS_PX;

    ensogl_core::define_shape_system! {
        (style:Style) {
            let sprite_width  : Var<Pixels> = "input_size.x".into();
            let sprite_height : Var<Pixels> = "input_size.y".into();
            let width         = sprite_width  - PADDING_PX.px() * 2.0;
            let height        = sprite_height - PADDING_PX.px() * 2.0;
            let color         = style.get_color(ensogl_theme::vars::widget::list_view::background::color);
            let rect          = Rect((&width,&height)).corners_radius(CORNER_RADIUS_PX.px());
            let shape         = rect.fill(color::Rgba::from(color));
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


/// The Model of Select Component.
#[derive(Clone,CloneRef,Debug)]
struct Model {
    scene          : Scene,
    entries        : entry::List,
    selection      : component::ShapeView<selection::Shape>,
    background     : component::ShapeView<background::Shape>,
    scrolled_area  : display::object::Instance,
    display_object : display::object::Instance,
}

impl Model {
    fn new(app:&Application) -> Self {
        let scene          = app.display.scene().clone_ref();
        let logger         = Logger::new("SelectionContainer");
        let display_object = display::object::Instance::new(&logger);
        let scrolled_area  = display::object::Instance::new(&logger);
        let entries        = entry::List::new(&logger, app);
        let background     = component::ShapeView::<background::Shape>::new(&logger,&scene);
        let selection      = component::ShapeView::<selection::Shape>::new(&logger,&scene);
        display_object.add_child(&background);
        display_object.add_child(&scrolled_area);
        scrolled_area.add_child(&entries);
        scrolled_area.add_child(&selection);
        Model{scene,entries,selection,background,display_object,scrolled_area}
    }

    /// Update the displayed entries list when _view_ has changed - the list was scrolled or
    /// resized.
    fn update_after_view_change(&self, view:&View) {
        let visible_entries = Self::visible_entries(view,self.entries.entry_count());
        let padding         = Vector2(2.0 * PADDING_PX, 2.0 * PADDING_PX);
        self.entries.set_position_x(-view.size.x / 2.0);
        self.background.shape.sprite.size.set(view.size + padding);
        self.scrolled_area.set_position_y(view.size.y / 2.0 - view.position_y);
        self.entries.update_entries(visible_entries);
    }

    fn set_entries(&self, provider:entry::AnyModelProvider, view:&View) {
        let visible_entries = Self::visible_entries(view,provider.entry_count());
        self.entries.update_entries_new_provider(provider,visible_entries);
    }

    fn visible_entries(View {position_y,size}:&View, entry_count:usize) -> Range<entry::Id> {
        if entry_count == 0 {
            0..0
        } else {
            let entry_at_y_saturating = |y:f32| {
                match entry::List::entry_at_y_position(y,entry_count) {
                    entry::IdAtYPosition::AboveFirst => 0,
                    entry::IdAtYPosition::UnderLast  => entry_count - 1,
                    entry::IdAtYPosition::Entry(id)  => id,
                }
            };
            let first = entry_at_y_saturating(*position_y);
            let last  = entry_at_y_saturating(position_y - size.y) + 1;
            first..last
        }
    }

    /// Check if the `point` is inside component assuming that it have given `size`.
    fn is_inside(&self, point:Vector2<f32>, size:Vector2<f32>) -> bool {
        let pos_obj_space = self.scene.screen_to_object_space(&self.background,point);
        let x_range       = (-size.x / 2.0)..=(size.x / 2.0);
        let y_range       = (-size.y / 2.0)..=(size.y / 2.0);
        x_range.contains(&pos_obj_space.x) && y_range.contains(&pos_obj_space.y)
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

ensogl_core::def_command_api! { Commands
    /// Move selection one position up.
    move_selection_up,
    /// Move selection page up (jump over all visible entries).
    move_selection_page_up,
    /// Move selection to the first argument.
    move_selection_to_first,
    /// Move selection one position down.
    move_selection_down,
    /// Move selection page down (jump over all visible entries).
    move_selection_page_down,
    /// Move selection to the last argument.
    move_selection_to_last,
    /// Chose the currently selected entry.
    chose_selected_entry,
    /// Deselect all entries.
    deselect_entries,
}

impl application::command::CommandApi for ListView {
    fn command_api_docs() -> Vec<application::command::EndpointDocs> {
        Commands::command_api_docs()
    }

    fn command_api(&self) -> Vec<application::command::CommandEndpoint> {
        self.frp.input.command.command_api()
    }
}

ensogl_text::define_endpoints! {
    Commands { Commands }
    Input {
        resize           (Vector2<f32>),
        scroll_jump      (f32),
        set_entries      (entry::AnyModelProvider),
        select_entry     (entry::Id),
        chose_entry      (entry::Id),
        deselect_entries (),
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
        let frp           = Frp::new_network();
        let model         = Model::new(app);
        ListView {frp,model}.init(app)
    }

    fn init(self, app:&Application) -> Self {
        const MAX_SCROLL:f32           = entry::HEIGHT/2.0;
        const MOUSE_MOVE_THRESHOLD:f32 = std::f32::EPSILON;

        let frp              = &self.frp;
        let network          = &frp.network;
        let model            = &self.model;
        let scene            = app.display.scene();
        let mouse            = &scene.mouse.frp;
        let view_y           = Animation::<f32>::new(&network);
        let selection_y      = Animation::<f32>::new(&network);
        let selection_height = Animation::<f32>::new(&network);

        frp::extend!{ network

            // === Mouse Position ===

            mouse_in <- all_with(&mouse.position,&frp.size,f!((pos,size)
                model.is_inside(*pos,*size)
            ));
            mouse_moved       <- mouse.distance.map(|dist| *dist > MOUSE_MOVE_THRESHOLD );
            mouse_y_in_scroll <- mouse.position.map(f!([model,scene](pos) {
                scene.screen_to_object_space(&model.scrolled_area,*pos).y
            }));
            mouse_pointed_entry <- mouse_y_in_scroll.map(f!([model](y)
                entry::List::entry_at_y_position(*y,model.entries.entry_count()).entry()
            ));

            // === Mouse Events ===

            frp.source.mouse_out  <+ mouse_in.gate_not(&mouse_in).constant(());
            frp.source.mouse_over <+ mouse_in.gate(&mouse_in).constant(());

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
            mouse_selected_entry <- mouse_pointed_entry.gate(&mouse_in).gate(&mouse_moved);

            frp.source.selected_entry <+ selected_entry_after_move;
            frp.source.selected_entry <+ mouse_selected_entry;
            frp.source.selected_entry <+ frp.deselect_entries.constant(None);
            frp.source.selected_entry <+ frp.set_entries.constant(None);


            // === Chosen Entry ===

            any_entry_selected        <- frp.selected_entry.map(|e| e.is_some());
            any_entry_pointed         <- mouse_pointed_entry.map(|e| e.is_some());
            opt_selected_entry_chosen <- frp.selected_entry.sample(&frp.chose_selected_entry);
            opt_pointed_entry_chosen  <- mouse_pointed_entry.sample(&mouse.down_0);
            frp.source.chosen_entry   <+ opt_pointed_entry_chosen.gate(&any_entry_pointed);
            frp.source.chosen_entry   <+ frp.chose_entry.map(|id| Some(*id));
            frp.source.chosen_entry   <+ opt_selected_entry_chosen.gate(&any_entry_selected);


            // === Selection Size and Position ===

            target_selection_y <- frp.selected_entry.map(|id|
                id.map_or(0.0,entry::List::position_y_of_entry)
            );
            target_selection_height <- frp.selected_entry.map(|id|
                if id.is_some() {entry::HEIGHT + 2.0 * PADDING_PX} else {0.0}
            );
            eval target_selection_y      ((y) selection_y.set_target_value(*y));
            eval target_selection_height ((h) selection_height.set_target_value(*h));
            eval frp.set_entries         ([selection_y,selection_height](_) {
                selection_y.skip();
                selection_height.skip();
            });
            eval selection_y.value       ((y) model.selection.set_position_y(*y));
            selection_size <- all_with(&frp.size,&selection_height.value,|size,height| {
                let width = size.x + 2.0 * PADDING_PX;
                Vector2(width,*height)
            });
            eval selection_size  ((size) model.selection.shape.sprite.size.set(*size));


            // === Scrolling ===

            selection_top_after_move_up <- selected_entry_after_move_up.map(|id|
                id.map(|id| entry::List::y_range_of_entry(id).end)
            );
            min_scroll_after_move_up <- selection_top_after_move_up.map(|top|
                top.unwrap_or(MAX_SCROLL)
            );
            scroll_after_move_up <- min_scroll_after_move_up.map2(&frp.scroll_position,|min,current|
                current.max(*min)
            );
            selection_bottom_after_move_down <- selected_entry_after_move_down.map(|id|
                id.map(|id| entry::List::y_range_of_entry(id).start)
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
            frp.source.scroll_position <+ frp.set_entries.constant(MAX_SCROLL);
            eval frp.scroll_position ((scroll_y) view_y.set_target_value(*scroll_y));
            eval frp.set_entries     ((_) {
                view_y.set_target_value(MAX_SCROLL);
                view_y.skip();
            });


            // === Resize ===
            frp.source.size <+ frp.resize;


            // === Update Entries ===

            view_info <- all_with(&view_y.value,&frp.size, |y,size|
                View{position_y:*y,size:*size}
            );
            // This should go before handling mouse events to have proper checking of
            eval view_info ((view) model.update_after_view_change(view));
            _new_entries <- frp.set_entries.map2(&view_info, f!((entries,view)
                model.set_entries(entries.clone_ref(),view)
            ));
        }

        view_y.set_target_value(MAX_SCROLL);
        view_y.skip();
        frp.scroll_jump(MAX_SCROLL);

        self
    }
}

impl display::Object for ListView {
    fn display_object(&self) -> &display::object::Instance { &self.model.display_object }
}

impl application::command::FrpNetworkProvider for ListView {
    fn network(&self) -> &frp::Network { &self.frp.network }
}

impl application::command::Provider for ListView {
    fn label() -> &'static str { "ListView" }
}

impl application::View for ListView {
    fn new(app:&Application) -> Self { ListView::new(app) }
}

impl application::shortcut::DefaultShortcutProvider for ListView {
    fn default_shortcuts() -> Vec<shortcut::Shortcut> {
        vec!
        [ Self::self_shortcut(shortcut::Action::press   (&[Key::ArrowUp]  , shortcut::Pattern::Any) , "move_selection_up")
        , Self::self_shortcut(shortcut::Action::press   (&[Key::ArrowDown], shortcut::Pattern::Any) , "move_selection_down")
        , Self::self_shortcut(shortcut::Action::press   (&[Key::PageUp]   , shortcut::Pattern::Any) , "move_selection_page_up")
        , Self::self_shortcut(shortcut::Action::press   (&[Key::PageDown] , shortcut::Pattern::Any) , "move_selection_page_down")
        , Self::self_shortcut(shortcut::Action::press   (&[Key::Home]     , shortcut::Pattern::Any) , "move_selection_to_first")
        , Self::self_shortcut(shortcut::Action::press   (&[Key::End]      , shortcut::Pattern::Any) , "move_selection_to_last")
        , Self::self_shortcut(shortcut::Action::press   (&[Key::Enter]    , shortcut::Pattern::Any) , "chose_selected_entry")
        ]
    }
}
