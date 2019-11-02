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

pub trait Callback = FnMut() + 'static;

// === Handle ===

#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct CallbackHandle (Rc<()>);

impl CallbackHandle {
    pub fn new() -> Self {
        default()
    }

    pub fn guard(&self) -> Guard {
        Guard(Rc::downgrade(&self.0))
    }

    pub fn forget(self) {
        std::mem::forget(self)
    }
}

pub struct Guard (Weak<()>);

impl Guard {
    pub fn exists(&self) -> bool {
        self.0.upgrade().is_some()
    }
}

// === Definition ===

#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct CallbackRegistry {
    #[derivative(Debug="ignore")]
    pub list: Vec<(Guard, Box<dyn FnMut()>)>
}

impl CallbackRegistry {
    pub fn add<F: Callback>(&mut self, callback: F) -> CallbackHandle {
        let callback = Box::new(callback) as Box<dyn FnMut()>;
        let handle   = CallbackHandle::new();
        let guard    = handle.guard();
        self.list.push((guard, callback));
        handle
    }

    pub fn run_all(&mut self) {
        self.list.retain(|(guard,_)| guard.exists());
        self.list.iter_mut().for_each(|(_, callback)| callback());
    }
}

// =================
// === EventLoop ===
// =================

// === Definition === 

#[derive(Shrinkwrap)]
#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct EventLoop {
    pub rc: Rc<RefCell<EventLoopData>>,
}

impl EventLoop {
    pub fn new() -> Self {
        Self::default().init()
    }

    fn init(self) -> Self {
        let data = Rc::downgrade(&self.rc);
        let main = move || { data.upgrade().map(|t| t.borrow_mut().run()); };
        with(self.borrow_mut(), |mut data| {
            data.main = Some(Closure::new(main));
            data.run();
        });
        self
    }

    pub fn add_callback<F: Callback>(&self, callback: F) -> CallbackHandle {
        self.borrow_mut().callbacks.add(callback)
    }

    pub fn clone_ref(&self) -> Self {
        let rc = Rc::clone(&self.rc);
        Self { rc }
    }
}

impl EventLoopData {
    pub fn run(&mut self) {
        let callbacks   = &mut self.callbacks;
        let callback_id = self.main.as_ref().map_or(default(), |main| {
            callbacks.run_all();
            web::request_animation_frame(main).unwrap()
        });
        self.main_id = callback_id;
    }
}

// === EventLoopData ===

#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct EventLoopData {
    pub main      : Option<Closure<dyn FnMut()>>,
    pub main_id   : i32,
    pub callbacks : CallbackRegistry,
}

impl Drop for EventLoopData {
    fn drop(&mut self) {
        web::cancel_animation_frame(self.main_id).ok();
    }
}





// =============
// === World ===
// =============

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

// === Definition === 

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct World {
    pub workspaces      : OptVec<Workspace>,
    pub workspace_dirty : WorkspaceDirty,
    pub logger          : Logger,
    pub event_loop      : EventLoop,
    pub update_handle  : Option<CallbackHandle>,
    pub self_reference  : Option<WorldRef>
}

// === Implementation ===

impl World {
    pub fn new() -> WorldRef {
        let data       = Rc::new(RefCell::new(Self::new_uninitialized()));
        let world_ref  = WorldRef { data };
        let world_ref2 = world_ref.clone_ref();
        let world_ref3 = world_ref.clone_ref();
        with(world_ref.borrow_mut(), |mut data| {
            let update         = move || world_ref2.borrow().update();
            let update_handle  = data.event_loop.add_callback(update);
            data.update_handle = Some(update_handle);
            data.self_reference = Some(world_ref3);
        });
        world_ref
    }

    pub fn new_uninitialized() -> Self {
        let workspaces       = default();
        let logger           = Logger::new("world");
        let workspace_logger = logger.sub("workspace_dirty");
        let workspace_dirty  = WorkspaceDirty::new((), workspace_logger);
        let event_loop       = EventLoop::new();
        let update_handle   = None;
        let self_reference   = None;
        Self { workspaces, workspace_dirty, logger, event_loop, update_handle, self_reference}
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

    pub fn on_frame<F: FnMut(&mut World) + 'static>(&mut self, mut callback: F) -> CallbackHandle { 
        let this = self.self_reference.as_ref().unwrap().clone_ref();
        let func = move || callback(&mut this.borrow_mut());
        self.event_loop.add_callback(func)
    }

    pub fn update(&self) {
        if self.is_dirty() {
            group!(self.logger, "Update.", {
                self.workspace_dirty.unset();
                self.workspaces.iter().for_each(|t| t.update());
            });
        }
    }

    pub fn dispose(&mut self) {
        self.update_handle = None;
    }
}


impl Add<workspace::WorkspaceBuilder> for World {
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

impl Index<usize> for World {
    type Output = Workspace;
    fn index(&self, ix: usize) -> &Self::Output {
        self.workspaces.index(ix)
    }
}

impl IndexMut<usize> for World {
    fn index_mut(&mut self, ix: usize) -> &mut Self::Output {
        self.workspaces.index_mut(ix)
    }
}

impl Drop for World {
    fn drop(&mut self) {
        self.logger.info("Dropping.");
    }
}

// ================
// === WorldRef ===
// ================

/// World is the top-level structure managing several instances of [Workspace].
/// It is responsible for updating the system on every animation frame.
#[derive(Shrinkwrap)]
#[derive(Debug)]
pub struct WorldRef {
    pub data: Rc<RefCell<World>>,
}

impl WorldRef {
    pub fn clone_ref(&self) -> Self {
        let data = Rc::clone(&self.data);
        Self { data }
    }

    pub fn dispose(&self) {
        self.borrow_mut().dispose()
    }
}

impl<T> Add<T> for WorldRef where World: Add<T> {
    type Result = AddResult<World,T>;
    fn add(&mut self, t: T) -> Self::Result {
        self.borrow_mut().add(t)
    }
}