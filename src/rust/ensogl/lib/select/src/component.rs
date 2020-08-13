//! Select List Component
use crate::prelude::*;

use crate::entry;

use ensogl::gui::component;
use ensogl::{display, application};
use ensogl::application::Application;
use ensogl::display::shape::*;
use ensogl::data::color;
use enso_frp as frp;
use ensogl::gui::component::Animation;
use ensogl::display::object::Instance;


pub const DEFAULT_WIDTH_PX:f32  = 100.0;
pub const DEFAULT_HEIGHT_PX:f32 = 150.0;

mod selection {
    use super::*;

    pub const CORNER_RADIUS_PX:f32 = 4.0;

    ensogl::define_shape_system! {
        (style:Style,_unchecked:f32) {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let color  = style.get("select.selection.color").color().unwrap_or_else(|| color::Rgba::new(1.0,0.0,0.0,1.0).into());
            let height = Var::from(entry::ENTRY_HEIGHT.px());
            let rect   = Rect((&width,&height)).corners_radius(CORNER_RADIUS_PX.px());
            let shape  = rect.fill(color::Rgba::from(color));
            shape.into()
        }
    }
}

#[derive(Clone,CloneRef,Debug)]
struct Model {
    entries        : entry::EntryList,
    selection      : component::ShapeView<selection::Shape>,
    scrolled_area  : display::object::Instance,
    display_object : display::object::Instance,
}

impl Model {
    fn new(app:&Application) -> Self {
        let scene          = app.display.scene();
        let logger         = Logger::new("SelectionContainer");
        let display_object = display::object::Instance::new(&logger);
        let scrolled_area  = display::object::Instance::new(&logger);
        let entries        = entry::EntryList::new(&logger,app);
        let selection      = component::ShapeView::<selection::Shape>::new(&logger,scene);
        display_object.add_child(&scrolled_area);
        scrolled_area.add_child(&entries);
        scrolled_area.add_child(&selection);
        Model{entries,selection,display_object,scrolled_area}
    }

    fn update_entries_after_window_move(&self, window_y:f32, window_size:&Vector2<f32>) {
        let visible_entries = self.visible_entries(window_y,window_size);
        self.entries.update_entries(visible_entries);
        self.selection.shape.sprite.size.set(Vector2(window_size.x, entry::ENTRY_HEIGHT));
        self.selection.mod_position(|pos| pos.x = window_size.x/2.0);
    }

    fn update_entries
    (&self, provider:entry::AnyModelProvider, window_y:f32, window_size:&Vector2<f32>) {
        let visible_entries = self.visible_entries(window_y,window_size);
        self.entries.update_entries_new_provider(provider,visible_entries);
    }

    fn visible_entries(&self, window_y:f32, window_size:&Vector2<f32>) -> Range<entry::Id> {
        let first = (-window_y.min(0.0)/entry::ENTRY_HEIGHT) as entry::Id;
        let last  = (-(window_y - window_size.y).min(0.0)/entry::ENTRY_HEIGHT) as entry::Id;
        first..(last + 1)
    }
}



// ===========
// === FRP ===
// ===========

ensogl::def_command_api! { Commands
    /// Move selection one position up.
    move_selection_up,
    /// Move selection one position down.
    move_selection_down,
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
        let frp     = &self.frp;
        let network = &frp.network;
        let model   = &self.model;

        let window_size = Animation::<Vector2<f32>>::new(&network);
        let window_y    = Animation::<f32>         ::new(&network);
        let selection_y = Animation::<f32>         ::new(&network);

        frp::extend!{ network
            // === Selection ===

            selected_entry_move_up <- frp.move_selection_up.map2(&frp.selected_entry, |(),id| {
                id.and_then(|id| id.checked_sub(1))
            });
            selected_entry_move_down <- frp.move_selection_down.map2(&frp.selected_entry, |(),id| {
                Some(id.map_or(0, |id| id+1))
            });
            frp.source.selected_entry <+ selected_entry_move_up;
            frp.source.selected_entry <+ selected_entry_move_down;


            // === Selection Position ===

            selection_x        <- window_size.value.map(|size| size.x/2.0);
            target_selection_y <- frp.selected_entry.map(|id| {
                match id {
                    Some(id) => -(*id as f32 * entry::ENTRY_HEIGHT),
                    None     => entry::ENTRY_HEIGHT,
                }
            });
            selection_pos <- all_with(&selection_x,&selection_y.value,|x,y| Vector2(*x,*y));
            //TODO[ao] can animation target be an frp input?
            eval target_selection_y ((y) selection_y.set_target_value(*y));
            eval selection_pos      ((position) model.selection.set_position_xy(*position));


            // === Resize and Scrolling ===

            eval frp.resize      ((size)     window_size.set_target_value(*size));
            eval frp.scroll_jump ((scroll_y) window_y.set_target_value(*scroll_y));
            window_info <- all_with(&window_y.value,&window_size.value, |window_y,window_size|(*window_y,*window_size));
            eval window_info (((window_y,window_size)) {
                model.scrolled_area.set_position_y(-window_y);
                model.update_entries_after_window_move(*window_y,window_size);
            });

            new_entries_with_window <- frp.set_entries.map2(&window_info, f!((entries,(window_y,window_size)) {
                model.scrolled_area.set_position_y(-window_y);
                model.update_entries(entries.clone_ref(),*window_y,window_size);
            }));


        }

        self
    }
}

impl display::Object for Select {
    fn display_object(&self) -> &Instance { &self.model.display_object }
}
