pub mod event_loop;
pub mod scene;
pub mod workspace;

use crate::prelude::*;

pub use crate::data::container::*;
pub use crate::display::world::workspace::SymbolId;

use crate::closure;
use crate::control::callback::CallbackHandle;
use crate::data::opt_vec::OptVec;
use crate::data::dirty;
use crate::data::dirty::traits::*;
use crate::promote_all;
use crate::promote_workspace_types;
use crate::promote;
use crate::system::web::group;
use crate::system::web::Logger;
use crate::display::shape::text::font::Fonts;

use event_loop::EventLoop;
use eval_tt::*;



// =============
// === World ===
// =============

// === Definition ===

/// Shared reference to the `World` object.
#[derive(Clone,Debug)]
pub struct World {
    pub rc: Rc<RefCell<WorldData>>,
}

impl World {
    /// Create new shared reference.
    pub fn new(world: WorldData) -> Self {
        let rc = Rc::new(RefCell::new(world));
        Self {rc}
    }

    /// Cheap clone of the world reference.
    pub fn clone_ref(&self) -> Self {
        self.clone()
    }

    /// Dispose the world object, cancel all handlers and events.
    pub fn dispose(&self) {
        self.rc.borrow_mut().dispose()
    }

    /// Run the provided callback on every frame. Returns a `CallbackHandle`,
    /// which when dropped will cancel the callback. If you want the function
    /// to run forever, you can use the `forget` method in the handle.
    pub fn on_frame<F:FnMut(&World)+'static>
    (&self, mut callback:F) -> CallbackHandle {
        let this = self.clone_ref();
        let func = move || callback(&this);
        self.rc.borrow_mut().event_loop.add_callback(func)
    }
}

impl<T> Add<T> for World where WorldData: Add<T> {
    type Result = AddResult<WorldData,T>;
    fn add(&mut self, t:T) -> Self::Result {
        self.rc.borrow_mut().add(t)
    }
}

impl Deref for World {
    type Target = Rc<RefCell<WorldData>>;

    fn deref(&self) -> &Self::Target {
        &self.rc
    }
}



// =================
// === WorldData ===
// =================

// === Definition === 

/// World is the top-level structure managing several instances of `Workspace`.
/// It is responsible for updating the system on every animation frame.
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct WorldData {
    pub workspace       : Workspace,
    pub workspace_dirty : WorkspaceDirty,
    pub logger          : Logger,
    pub event_loop      : EventLoop,
    pub fonts           : Fonts,
    pub update_handle   : Option<CallbackHandle>,
}


// === Types ===

pub type WorkspaceID    = usize;
pub type WorkspaceDirty = dirty::SharedBool;
promote_workspace_types!{ [[WorkspaceOnChange]] workspace }


// === Callbacks ===

closure! {
fn workspace_on_change(dirty:WorkspaceDirty) -> WorkspaceOnChange {
    || dirty.set()
}}


// === Implementation ===

impl WorldData {
    /// Create and initialize new world instance.
    #[allow(clippy::new_ret_no_self)]
    pub fn new<Dom:Str>(dom:Dom) -> World {
        println!("NOTICE! When profiling in Chrome check 'Disable JavaScript Samples' under the \
                  gear icon in the 'Performance' tab. It can drastically slow the rendering.");
        let world     = World::new(Self::new_uninitialized(dom));
        let world_ref = world.clone_ref();
        with(world.borrow_mut(), |mut data| {
            let update          = move || world_ref.borrow_mut().run();
            let update_handle   = data.event_loop.add_callback(update);
            data.update_handle  = Some(update_handle);
        });
        world
    }

    /// Create new uninitialized world instance. You should rather not need to
    /// call this function directly.
    fn new_uninitialized<Dom:Str>(dom:Dom) -> Self {
        let logger                 = Logger::new("world");
        let workspace_logger       = logger.sub("workspace");
        let workspace_dirty_logger = logger.sub("workspace_dirty");
        let workspace_dirty        = WorkspaceDirty::new(workspace_dirty_logger,());
        let on_change              = workspace_on_change(workspace_dirty.clone_ref());
        let workspace              = Workspace::new(dom,workspace_logger,on_change).unwrap(); // fixme unwrap
        let fonts                  = Fonts::new();
        let event_loop             = EventLoop::new();
        let update_handle          = default();
        Self {workspace,workspace_dirty,logger,event_loop,fonts,update_handle}
    }

    pub fn run(&mut self) {
        self.update();
    }

    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self) {
        if self.workspace_dirty.check_all() {
            group!(self.logger, "Updating.", {
        // FIXME render only needed workspaces.
        self.workspace_dirty.unset_all();
        let fonts = &mut self.fonts;
        self.workspace.update(fonts);
            });
        }
    }

    /// Dispose the world object, cancel all handlers and events.
    pub fn dispose(&mut self) {
        self.update_handle = None;
    }
}

impl Drop for WorldData {
    fn drop(&mut self) {
        self.logger.info("Dropping.");
    }
}



