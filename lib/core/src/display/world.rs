use crate::prelude::*;

use crate::data::opt_vec::OptVec;
use crate::display::workspace;
use crate::system::web;
use crate::system::web::fmt;
use crate::system::web::group;
use crate::system::web::Logger;
use wasm_bindgen::prelude::Closure;
use crate::closure;
use crate::dirty;
use crate::data::function::callback::*;

use std::collections::HashMap;

pub use crate::display::workspace::MeshID;


// ===========
// === Add ===
// ===========

pub trait Add<T> {
    type Result = ();
    fn add(&mut self, component: T) -> Self::Result;
}

type AddResult<T,S> = <T as Add<S>>::Result;

// ========================
// === CallbackRegistry ===
// ========================

// === Types ===

pub trait DynCallback = FnMut() + 'static;

// === Handle ===

#[derive(Derivative)]
#[derivative(Debug)]
pub struct CallbackHandle {
    pub id       : usize,
    pub registry : CallbackRegistry
}

impl Drop for CallbackHandle {
    fn drop(&mut self) {
        self.registry.raw.borrow_mut().remove(self.id);
    }
}

// === Definition ===

#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct CallbackRegistry {
    #[derivative(Debug="ignore")]
    pub raw: Rc<RefCell<OptVec<Box<dyn FnMut()>>>>
}

impl CallbackRegistry {
    pub fn clone_ref(&self) -> Self {
        let raw = Rc::clone(&self.raw);
        Self { raw }
    }

    pub fn add<F: DynCallback>(&self, callback: F) -> CallbackHandle {
        let callback = Box::new(callback) as Box<dyn FnMut()>;
        let registry = self.clone_ref();
        let id       = self.raw.borrow_mut().insert(callback);
        CallbackHandle { id, registry }
    }

    pub fn run_all(&self) {
        self.raw.borrow_mut().iter_mut().for_each(|f| f());
    }
}

// =================
// === EventLoop ===
// =================

// === Definition === 

#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct EventLoop {
    pub rc: Rc<RefCell<EventLoopData>>,
}

#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct EventLoopData {
    pub main      : Option<Closure<dyn FnMut()>>,
    pub main_id   : i32,
    pub callbacks : CallbackRegistry,
}

impl EventLoop {
    pub fn new() -> Self {
        let event_loop      = Self::default();
        let event_loop_weak = Rc::downgrade(&event_loop.rc);
        event_loop.set_main(move || {
            event_loop_weak.upgrade().map(|t| t.borrow_mut().run());
        });
        event_loop.rc.borrow_mut().run();
        event_loop
    }

    pub fn add_callback<F: DynCallback>(&self, callback: F) -> CallbackHandle {
        self.rc.borrow().callbacks.add(callback)
    }

    pub fn clone_ref(&self) -> Self {
        let rc = Rc::clone(&self.rc);
        Self { rc }
    }

    fn set_main<F: DynCallback>(&self, main: F) {
        let main = Closure::wrap(Box::new(main) as Box<dyn FnMut()>);
        self.rc.borrow_mut().main = Some(main);
    }
}

impl EventLoopData {
    pub fn run(&mut self) {
        let callback_id = self.main.as_ref().map_or(default(), |main| {
            self.callbacks.run_all();
            web::request_animation_frame(main).unwrap()
        });
        self.main_id = callback_id;
    }
}

impl Drop for EventLoopData {
    fn drop(&mut self) {
        web::cancel_animation_frame(self.main_id);
    }
}



// =============
// === World ===
// =============

// fn testme() {
//     let a:HashMap<fn()> = default();
// }

/// World is the top-level structure managing several instances of [Workspace].
/// It is responsible for updating the system on every animation frame.
#[derive(Debug)]
pub struct World {
    pub data           : Rc<RefCell<WorldData>>,
    pub event_loop     : EventLoop,
    pub refresh_handle : CallbackHandle
}

impl Default for World {
    fn default() -> Self {
        let data           = Rc::new(RefCell::new(WorldData::new()));
        let mut event_loop = EventLoop::new();
        let local_data     = Rc::clone(&data);
        let refresh_handle = event_loop.add_callback(move || {
            local_data.borrow().refresh();
        });
        Self { data, event_loop, refresh_handle }
    }
}

impl World {
    pub fn new() -> Self {
        let out: Self = Default::default();
        // out.start();
        out
    }

    pub fn started(&self) -> bool {
        self.data.borrow().started
    }

    // pub fn start(&self) {
    //     if !self.started() {
    //         self.data.borrow_mut().started = true;
    //         self.event_loop.run();
    //     }
    // }

    // pub fn stop(&self) {
    //     self.data.borrow_mut().started = false;
    // }

    pub fn add_workspace(&self, name: &str) -> WorkspaceID {
        self.data.borrow_mut().add_workspace(name)
    }

    // pub fn drop_workspace(&self, id: workspace::ID) {
    //     self.data.borrow_mut().drop_workspace(id)
    // }

    // pub fn refresh(&self) {
    //     self.data.borrow().refresh()
    // }

    // fn request_callback(callback: &EventLoop) {
    //     callback.main.borrow().as_ref().iter().for_each(|f| {
    //         web::request_animation_frame(f).unwrap();
    //     });
    // }
}

impl<T> Add<T> 
for World where WorldData: Add<T> {
    type Result = AddResult<WorldData,T>;
    fn add(&mut self, t: T) -> Self::Result {
        self.data.borrow_mut().add(t)
    }
}

// =================
// === WorldData ===
// =================

// === Definition === 

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct WorldData {
    pub workspaces      : OptVec<Workspace>,
    pub workspace_dirty : WorkspaceDirty,
    pub logger          : Logger,
    pub started         : bool,
    // pub on_frame_fns    : HashMap<usize, Box<dyn FnMut()>>
}

impl Default for WorldData {
    fn default() -> Self {
        let workspaces       = default();
        let logger           = Logger::new("world");
        let workspace_logger = logger.sub("workspace_dirty");
        let workspace_dirty  = WorkspaceDirty::new((), workspace_logger);
        let started          = false;
        // let on_frame_fns     = default();
        Self { workspaces, workspace_dirty, logger, started}//, on_frame_fns }
    }
}

// === Types ===

pub type WorkspaceID    = usize;
pub type WorkspaceDirty = dirty::SharedSet<WorkspaceID, ()>;

pub type AttributeIndex <T> = workspace::AttributeIndex<T, Closure_workspace_on_change_handler>;
pub type Mesh           = workspace::Mesh           <Closure_workspace_on_change_handler>;
pub type Geometry       = workspace::Geometry       <Closure_workspace_on_change_handler>;
pub type Scopes         = workspace::Scopes         <Closure_workspace_on_change_handler>;
pub type AttributeScope = workspace::AttributeScope <Closure_workspace_on_change_handler>;
pub type UniformScope   = workspace::UniformScope   <Closure_workspace_on_change_handler>;
pub type GlobalScope    = workspace::GlobalScope    <Closure_workspace_on_change_handler>;
pub type Attribute  <T> = workspace::Attribute   <T, Closure_workspace_on_change_handler>;
pub type Workspace      = workspace::Workspace      <Closure_workspace_on_change_handler>;

// === Callbacks ===

closure!(workspace_on_change_handler<>
    (dirty: WorkspaceDirty, ix: WorkspaceID) || { dirty.set(ix) });

// === Implementation ===

impl WorldData {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn is_dirty(&self) -> bool {
        self.workspace_dirty.is_set()
    }

    pub fn add_workspace(&mut self, name: &str) -> WorkspaceID {
        let logger = &self.logger;
        let dirty  = &self.workspace_dirty;
        self.workspaces.insert_with_ix(|ix| {
            group!(logger, format!("Adding workspace {} ({}).", ix, name), {
                let on_change = workspace_on_change_handler(dirty.clone(), ix);
                let wspace_logger = logger.sub(ix.to_string());
                Workspace::new(name, wspace_logger, on_change).unwrap() // FIXME
            })
        })
        
    }

    pub fn drop_workspace(&mut self, id: WorkspaceID) {
        let logger = &self.logger;
        let item   = self.workspaces.remove(id);
        match item {
            None => logger.warning("Trying to delete non-existing workspace."),
            Some(item) => logger.group(fmt!("Dropping workspace {}.", id), || {
                let _destruct_it_here = item;
            }),
        }
    }

    pub fn refresh(&self) {
        if self.is_dirty() {
            group!(self.logger, "Refresh.", {
                self.workspace_dirty.unset();
                self.workspaces.iter().for_each(|t| t.refresh());
            });
        }
    }

}


impl Add<workspace::WorkspaceBuilder> for WorldData {
    type Result = WorkspaceID;
    fn add(&mut self, bldr: workspace::WorkspaceBuilder) -> Self::Result {
        let name   = bldr.name;
        let logger = &self.logger;
        let dirty  = &self.workspace_dirty;
        self.workspaces.insert_with_ix(|ix| {
            group!(logger, format!("Adding workspace {} ({}).", ix, name), {
                let on_change = workspace_on_change_handler(dirty.clone(), ix);
                let wspace_logger = logger.sub(ix.to_string());
                Workspace::new(name, wspace_logger, on_change).unwrap() // FIXME
            })
        })
    }
}

impl Index<usize> for WorldData {
    type Output = Workspace;
    fn index(&self, ix: usize) -> &Self::Output {
        self.workspaces.index(ix)
    }
}

impl IndexMut<usize> for WorldData {
    fn index_mut(&mut self, ix: usize) -> &mut Self::Output {
        self.workspaces.index_mut(ix)
    }
}

