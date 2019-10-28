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

// =============
// === Types ===
// =============

type Callback = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

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
    pub data     : Rc<RefCell<WorldData>>,
    pub on_frame : Callback,
}

impl Default for World {
    fn default() -> Self {
        let data           = Rc::new(RefCell::new(WorldData::new()));
        let on_frame       = Rc::new(RefCell::new(None));
        let data_local     = data.clone();
        let on_frame_local = on_frame.clone();
        *on_frame.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let data_local = data_local.borrow();
            if data_local.started {
                // data_local.
                data_local.refresh();
                Self::request_callback(&on_frame_local);
            }
        }) as Box<dyn FnMut()>));
        Self { data, on_frame }
    }
}

impl World {
    pub fn new() -> Self {
        let out: Self = Default::default();
        out.start();
        out
    }

    pub fn started(&self) -> bool {
        self.data.borrow().started
    }

    pub fn start(&self) {
        if !self.started() {
            self.data.borrow_mut().started = true;
            Self::request_callback(&self.on_frame);
        }
    }

    pub fn stop(&self) {
        self.data.borrow_mut().started = false;
    }

    pub fn add_workspace(&self, name: &str) -> WorkspaceID {
        self.data.borrow_mut().add_workspace(name)
    }

    // pub fn drop_workspace(&self, id: workspace::ID) {
    //     self.data.borrow_mut().drop_workspace(id)
    // }

    // pub fn refresh(&self) {
    //     self.data.borrow().refresh()
    // }

    fn request_callback(callback: &Callback) {
        callback.borrow().as_ref().iter().for_each(|f| {
            web::request_animation_frame(f).unwrap();
        });
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

    fn xindex_mut(&mut self, ix: usize) {
        let w: &mut OptVec<Workspace> = &mut self.workspaces;
        // let a: &mut Option<Workspace> = self.workspaces.index_mut(ix);
        // self.workspaces.index_mut(ix).as_mut().unwrap()
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
