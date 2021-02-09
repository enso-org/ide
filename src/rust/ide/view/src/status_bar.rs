//! A module containing IDE status bar component definitions (frp, model, view, etc.)
use crate::prelude::*;

use ensogl::application::Application;
use ensogl::display;
use ensogl::display::Scene;
use ensogl_text as text;



// =================
// === Constants ===
// =================

pub const DEFAULT_WIDTH : f32 = 1024.0;
pub const HEIGHT        : f32 = 40.0;
pub const PADDING       : f32 = 12.0;



// =============
// === Event ===
// =============

pub mod event {
    use crate::prelude::*;

    /// An id of some event displayed in a status bar.
    #[derive(Clone,CloneRef,Copy,Debug,Default,Eq,From,Hash,Into,PartialEq)]
    pub struct Id(pub usize);

    im_string_newtype! {
        /// A label assigned to some event displayed in a status bar.
        Label
    }
}



// ===============
// === Process ===
// ===============

pub mod process {
    use crate::prelude::*;

    /// An id of some process displayed in a status bar.
    #[derive(Clone,CloneRef,Copy,Debug,Default,Eq,From,Hash,Into,PartialEq)]
    pub struct Id(pub u64);

    impl Id {
        pub fn next(&self) -> Id {
            Id(self.0 + 1)
        }
    }

    im_string_newtype! {
        /// A label assigned to some process displayed in a status bar.
        Label
    }

    #[derive(Debug)]
    pub struct Guard {
        pub(crate) id                      : Id,
        pub(crate) finish_process_endpoint : enso_frp::Any<Id>,
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            self.finish_process_endpoint.emit(self.id);
        }
    }
}



// ==============
// === Shapes ===
// ==============

mod background {
    // TODO
}



// ===========
// === FRP ===
// ===========

ensogl::define_endpoints! {
    Input {
        add_event      (event::Label),
        add_process    (process::Label),
        finish_process (process::Id),
    }
    Output {
        width             (f32),
        last_event        (event::Id),
        last_process      (process::Id),
        displayed_event   (Option<event::Id>),
        displayed_process (Option<process::Id>),
    }
}



// =============
// === Model ===
// =============

/// An internal model of Status Bar component
#[derive(Clone,CloneRef,Debug)]
struct Model {
    logger          : Logger,
    display_object  : display::object::Instance,
    label           : text::Area,
    events          : Rc<RefCell<Vec<event::Label>>>,
    processes       : Rc<RefCell<HashMap<process::Id,process::Label>>>,
    next_process_id : Rc<RefCell<process::Id>>,
}

impl Model {
    fn new(app:&Application) -> Self {
        let logger          = Logger::new("StatusBar");
        let display_object  = display::object::Instance::new(&logger);
        let label           = text::Area::new(app);
        let events          = default();
        let processes       = default();
        let next_process_id = default();
        display_object.add_child(&label);
        Self {logger,display_object,label,events,processes,next_process_id}
    }

    fn set_width(&self, width:f32) {
        self.label.set_position_x(-width/2.0 + PADDING);
    }

    fn add_event(&self, label:&event::Label) -> event::Id {
        let mut events = self.events.borrow_mut();
        let new_id     = event::Id(events.len());
        events.push(label.clone_ref());
        new_id
    }

    fn add_process(&self, label:&process::Label) -> process::Id {
        let mut processes       = self.processes.borrow_mut();
        let mut next_process_id = self.next_process_id.borrow_mut();
        let new_id              = *next_process_id;
        *next_process_id        = next_process_id.next();
        processes.insert(new_id,label.clone_ref());
        new_id
    }

    /// Returns true if there was process with given id.
    fn finish_process(&self, id:process::Id) -> bool {
        self.processes.borrow_mut().remove(&id).is_some()
    }
}



// ============
// === View ===
// ============

/// The StatusBar component view.
///
/// The status bar gathers information about events and processes occurring in the Application.
// TODO: add notion about number of processes running.
#[derive(Clone,CloneRef,Debug)]
pub struct View {
    frp   : Frp,
    model : Model,
}

impl View {
    /// Create new StatusBar view.
    pub fn new(app:&Application) -> Self {
        let frp         = Frp::new();
        let model       = Model::new(&app);
        let network     = &frp.network;
        let scene       = app.display.scene();
        let scene_shape = app.display.scene().shape().clone_ref();

        enso_frp::extend! { network
            event_added      <- frp.add_event.map(f!((label) model.add_event(label)));
            process_added    <- frp.add_process.map(f!((label) model.add_process(label)));
            process_finished <- frp.finish_process.filter_map(f!([model](id)
                model.finish_process(*id).as_some(*id)
            ));
            displayed_process_finished <- frp.finish_process.all(&frp.output.displayed_process).filter(|(fin,dis)| dis.contains(fin));

            label_after_adding_event <- frp.add_event.map(|label| AsRef::<ImString>::as_ref(label).clone_ref());
            label_after_adding_process <- frp.add_process.map(|label| AsRef::<ImString>::as_ref(label).clone_ref());
            label_after_finishing_process <- displayed_process_finished.constant(ImString::default());

            label <- any(label_after_adding_event,label_after_adding_process,label_after_finishing_process);
            eval label ((label) model.label.set_content(label.to_string()));

            frp.source.last_event   <+ event_added;
            frp.source.last_process <+ process_added;

            frp.source.displayed_event <+ event_added.map(|id| Some(*id));
            frp.source.displayed_event <+ process_added.constant(None);
            frp.source.displayed_process <+ process_added.map(|id| Some(*id));
            frp.source.displayed_process <+ event_added.constant(None);
            frp.source.displayed_process <+ displayed_process_finished.constant(None);

            width <- scene_shape.map(|scene_shape| scene_shape.width);
            eval width ((width) model.set_width(*width));
            eval scene_shape ((scene_shape)
                model.display_object.set_position_y(-scene_shape.height / 2.0 + HEIGHT / 2.0);
            );
            frp.source.width <+ width;
        }

        model.set_width(scene_shape.value().width);

        Self {frp,model}
    }

    pub fn add_process(&self, label:process::Label) -> process::Guard {
        self.frp.add_process(label);
        let id = self.frp.last_process.value();
        let finish_process_endpoint = self.frp.finish_process.clone_ref().into();
        process::Guard {id,finish_process_endpoint}
    }
}

impl display::Object for View {
    fn display_object(&self) -> &display::object::Instance<Scene> { &self.model.display_object }
}

impl Deref for View {
    type Target = Frp;

    fn deref(&self) -> &Self::Target { &self.frp }
}