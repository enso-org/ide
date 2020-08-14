//! Select List Component
use crate::prelude::*;

use crate::entry;

use ensogl_core::gui::component;
use ensogl_core::{display, application};
use ensogl_core::application::{Application, shortcut};
use ensogl_core::display::shape::*;
use ensogl_core::data::color;
use enso_frp as frp;
use ensogl_core::gui::component::Animation;
use ensogl_core::display::object::Instance;
use enso_frp::io::keyboard::Key;


pub const DEFAULT_WIDTH_PX:f32  = 100.0;
pub const DEFAULT_HEIGHT_PX:f32 = 150.0;

#[derive(Copy,Clone,Debug,Default)]
struct WindowInfo {
    position_y : f32,
    size       : Vector2<f32>,
}

mod selection {
    use super::*;

    pub const CORNER_RADIUS_PX:f32 = entry::PADDING * 2.0;

    ensogl_core::define_shape_system! {
        (style:Style) {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let color  = style.get("select.selection.color").color().unwrap_or_else(|| color::Rgba::new(1.0,0.0,0.0,1.0).into());
            let rect   = Rect((&width,&height)).corners_radius(CORNER_RADIUS_PX.px());
            let shape  = rect.fill(color::Rgba::from(color));
            shape.into()
        }
    }
}

mod background {
    use super::*;

    ensogl_core::define_shape_system! {
        (style:Style) {
            let color  = style.get("select.background.color").color().unwrap_or_else(|| color::Rgba::new(0.4,0.4,0.4,1.0).into());
            let shape = Plane().fill(color::Rgba::from(color));
            shape.into()
        }
    }
}

#[derive(Clone,CloneRef,Debug)]
struct Model {
    entries        : entry::List,
    selection      : component::ShapeView<selection::Shape>,
    background     : component::ShapeView<background::Shape>,
    scrolled_area  : display::object::Instance,
    display_object : display::object::Instance,
}

impl Model {
    fn new(app:&Application) -> Self {
        let scene          = app.display.scene();
        let logger         = Logger::new("SelectionContainer");
        let display_object = display::object::Instance::new(&logger);
        let scrolled_area  = display::object::Instance::new(&logger);
        let entries        = entry::List::new(&logger, app);
        let background     = component::ShapeView::<background::Shape>::new(&logger,scene);
        let selection      = component::ShapeView::<selection::Shape>::new(&logger,scene);
        display_object.add_child(&background);
        display_object.add_child(&scrolled_area);
        scrolled_area.add_child(&entries);
        scrolled_area.add_child(&selection);
        Model{entries,selection,background,display_object,scrolled_area}
    }

    fn update_after_window_change(&self, window:&WindowInfo) {
        let visible_entries = self.visible_entries(window);
        self.entries.update_entries(visible_entries);
        self.entries.set_position_x(-window.size.x / 2.0);
        self.background.shape.sprite.size.set(window.size);
        self.scrolled_area.set_position_y(window.size.y / 2.0 - window.position_y);
    }

    fn update_entries
    (&self, provider:entry::AnyModelProvider, window:&WindowInfo) {
        let visible_entries = self.visible_entries(window);
        self.entries.update_entries_new_provider(provider,visible_entries);
    }

    fn visible_entries(&self, WindowInfo{position_y,size}:&WindowInfo) -> Range<entry::Id> {
        let first = (-position_y.min(0.0)/entry::HEIGHT - 0.5 + entry::PADDING/entry::HEIGHT) as entry::Id;
        let last  = (-(position_y - size.y).min(0.0)/entry::HEIGHT + 0.5 - entry::PADDING/entry::HEIGHT) as entry::Id;
        first..(last + 1)
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
    /// Move selection one position down.
    move_selection_down,
    /// Move selection page down (jump over all visible entries).
    move_selection_page_down,
}

impl application::command::CommandApi for Select {
    fn command_api_docs() -> Vec<application::command::EndpointDocs> {
        Commands::command_api_docs()
    }

    fn command_api(&self) -> Vec<application::command::CommandEndpoint> {
        self.frp.input.command.command_api()
    }
}

// TODO[ao] move to another crate.
ensogl_text::define_endpoints! {
    Commands { Commands }
    Input {
        resize           (Vector2<f32>),
        scroll_jump      (f32),
        set_entries      (entry::AnyModelProvider),
        select_entry     (entry::Id),
        deselect_entries (),
    }
    Output {
        selected_entry (Option<entry::Id>),

    }
}

pub struct Select {
    model   : Model,
    pub frp : Frp,
}

impl Deref for Select {
    type Target = Frp;
    fn deref(&self) -> &Self::Target { &self.frp }
}

impl Select {
    pub fn new(app:&Application) -> Self {
        // TODO[ao] seems like typical setup. Perhaps it should be a generated function.
        let network        = frp::Network::new();
        let frp_inputs     = FrpInputs::new(&network);
        let frp_endpoints  = FrpEndpoints::new(&network,frp_inputs.clone_ref());
        let frp            = Frp::new(network,frp_endpoints);
        let model          = Model::new(app);
        Select{frp,model}.init()
    }

    fn init(self) -> Self {
        const MAX_SCROLL:f32 = entry::HEIGHT/2.0;

        let frp              = &self.frp;
        let network          = &frp.network;
        let model            = &self.model;
        let window_y         = Animation::<f32>::new(&network);
        let selection_y      = Animation::<f32>::new(&network);
        let selection_height = Animation::<f32>::new(&network);
        window_y.set_value(MAX_SCROLL);
        window_y.set_target_value(MAX_SCROLL);

        frp::extend!{ network
            window_info <- all_with(&window_y.value,&frp.resize, |y,size| WindowInfo{position_y:*y,size:*size});
            min_scroll  <- all_with(&frp.resize,&frp.set_entries,|size,entries| (entries.entry_count() as f32 * -entry::HEIGHT - size.y - MAX_SCROLL).min(MAX_SCROLL));


            // === Selection ===

            selection_jump_on_move_up   <- frp.move_selection_up.constant(-1);
            selection_jump_on_page_up   <- frp.move_selection_page_up.map2(&window_info, f!([model]((),window) -(model.visible_entries(&window).len() as isize)));
            selection_jump_on_move_down <- frp.move_selection_down.constant(1);
            selection_jump_on_page_down   <- frp.move_selection_page_down.map2(&window_info, f!([model]((),window) (model.visible_entries(&window).len() as isize)));
            selection_jump_up           <- any(selection_jump_on_move_up,selection_jump_on_page_up);
            selection_jump_down         <- any(selection_jump_on_move_down,selection_jump_on_page_down);

            selected_entry_after_jump_up <- selection_jump_up.map2(&frp.selected_entry, |jump,id| {
                id.and_then(|id| id.checked_sub(-jump as usize))
            });
            selected_entry_after_jump_down <- selection_jump_down.map3(&frp.selected_entry, &frp.set_entries, |jump,id,entries| {
                entries.entry_count().checked_sub(1).map_or(None, |max_entry| Some(id.map_or(0, |id| id+(*jump as usize)).min(max_entry)))
            });
            selected_entry_after_move <- any(&selected_entry_after_jump_up,&selected_entry_after_jump_down);
            frp.source.selected_entry <+ selected_entry_after_move;


            // === Selection Position ===

            target_selection_y <- frp.selected_entry.map(|id| id.map_or(0.0, |id| id as f32 * -entry::HEIGHT));
            target_selection_height <- frp.selected_entry.map(|id| if id.is_some() {entry::HEIGHT} else {0.0});
            //TODO[ao] can animation target be an frp input?
            eval target_selection_y      ((y) selection_y.set_target_value(*y));
            eval target_selection_height ((h) selection_height.set_target_value(*h));
            eval selection_y.value       ((y) model.selection.set_position_y(*y));
            selection_size <- all_with(&frp.resize,&selection_height.value, |window,height| Vector2(window.x,*height));
            eval selection_size  ((size) model.selection.shape.sprite.size.set(*size));

            // === Resize and Scrolling ===

            target_scroll <- any(...);
            selection_top_after_move_up      <- selected_entry_after_jump_up.map(|id| id.map(|id| entry::List::position_y_range_of_entry(id).end));
            min_scroll_after_move_up         <- selection_top_after_move_up.map(|top| top.unwrap_or(MAX_SCROLL));
            scroll_after_move_up             <- min_scroll_after_move_up.map2(&target_scroll, |min_scroll,current:&f32| current.max(*min_scroll));
            selection_bottom_after_move_down <- selected_entry_after_jump_down.map(|id| id.map(|id| entry::List::position_y_range_of_entry(id).start));
            max_scroll_after_move_down       <- selection_bottom_after_move_down.map2(&frp.resize, |id,window_size| id.map_or(MAX_SCROLL, |id| id + window_size.y));
            scroll_after_move_down           <- max_scroll_after_move_down.map2(&target_scroll, |max_scroll,current| current.min(*max_scroll));

            target_scroll <+ scroll_after_move_up;
            target_scroll <+ scroll_after_move_down;
            target_scroll <+ frp.scroll_jump;

            eval target_scroll ((scroll_y) window_y.set_target_value(*scroll_y));
            eval window_info ((window) {
                model.update_after_window_change(window);
            });

            _new_entries_with_window <- frp.set_entries.map2(&window_info, f!((entries,window) {
                model.update_entries(entries.clone_ref(),window);
            }));


        }

        self
    }
}

impl display::Object for Select {
    fn display_object(&self) -> &Instance { &self.model.display_object }
}

impl application::command::FrpNetworkProvider for Select {
    fn network(&self) -> &frp::Network { &self.frp.network }
}

impl application::command::Provider for Select {
    fn label() -> &'static str { "Select" }
}

impl application::View for Select {
    fn new(app:&Application) -> Self { Select::new(app) }
}

impl application::shortcut::DefaultShortcutProvider for Select {
    fn default_shortcuts() -> Vec<shortcut::Shortcut> {
        vec!
        [ Self::self_shortcut(shortcut::Action::press   (&[Key::ArrowUp]  , shortcut::Pattern::Any) , "move_selection_up")
        , Self::self_shortcut(shortcut::Action::press   (&[Key::ArrowDown], shortcut::Pattern::Any) , "move_selection_down")
        , Self::self_shortcut(shortcut::Action::press   (&[Key::PageUp]   , shortcut::Pattern::Any) , "move_selection_page_up")
        , Self::self_shortcut(shortcut::Action::press   (&[Key::PageDown] , shortcut::Pattern::Any) , "move_selection_page_down")
        ]
    }
}
