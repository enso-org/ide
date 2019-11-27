use crate::prelude::*;

pub use crate::display::mesh_registry::MeshID;

use crate::backend::webgl;
use crate::closure;
use crate::data::function::callback::*;
use crate::dirty;
use crate::dirty::traits::*;
use crate::display::mesh_registry;
use crate::promote_all;
use crate::promote_mesh_registry_types;
use crate::promote; 
use crate::system::web;
use crate::system::web::fmt;
use crate::system::web::group;
use crate::system::web::Logger;
use crate::system::web::resize_observer::ResizeObserver;
use eval_tt::*;
use wasm_bindgen::prelude::Closure;


// =============
// === Error ===
// =============

#[derive(Debug, Fail, From)]
pub enum Error {
    #[fail(display = "Web Platform error: {}", error)]
    WebError { error: web::Error },
}

// =================
// === Workspace ===
// =================

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Workspace<OnDirty> {
    pub canvas              : web_sys::HtmlCanvasElement,
    pub context             : webgl::Context,
    pub pixel_ratio         : f64,
    pub mesh_registry       : MeshRegistry<OnDirty>,
    pub mesh_registry_dirty : MeshRegistryDirty<OnDirty>,
    pub shape               : Rc<RefCell<WorkspaceShape>>,
    pub shape_dirty         : ShapeDirty<OnDirty>,
    pub logger              : Logger,
    pub listeners           : Listeners,
}

#[derive(Default)]
#[derive(Debug)]
pub struct WorkspaceShape {
    pub width  : i32,
    pub height : i32,
}

pub type WorkspaceShapeDirtyState = WorkspaceShape;

// === Types ===

pub type ShapeDirty        <Callback> = dirty::SharedBool<Callback>;
pub type MeshRegistryDirty <Callback> = dirty::SharedBool<Callback>;
promote_mesh_registry_types!{ [OnMeshRegistryChange] mesh_registry }

#[macro_export]
macro_rules! promote_workspace_types { ($($args:tt)*) => {
    crate::promote_mesh_registry_types! { $($args)* }
    promote! { $($args)* [Workspace] }
};}

// === Callbacks ===

closure! {
fn mesh_registry_on_change<C:Callback0> (dirty:MeshRegistryDirty<C>) -> 
    OnMeshRegistryChange { || dirty.set() }
}

// === Implementation ===

#[derive(Debug)]
pub struct Listeners {
    resize: ResizeObserver,
}

impl<OnDirty: Clone + Callback0 + 'static> Workspace<OnDirty> {
    /// Create new instance with the provided on-dirty callback.
    pub fn new<Dom: Str>
    (dom:Dom, logger:Logger, on_dirty:OnDirty) -> Result<Self, Error> {
        logger.trace("Initializing.");
        let dom           = dom.as_ref();
        let canvas        = web::get_canvas(dom)?;
        let context       = web::get_webgl_context(&canvas,1)?;
        let pixel_ratio   = web::device_pixel_ratio()?;
        let sub_logger    = logger.sub("shape_dirty");
        let shape_dirty   = ShapeDirty::new(sub_logger,on_dirty.clone());
        let sub_logger    = logger.sub("mesh_registry_dirty");
        let dirty_flag    = MeshRegistryDirty::new(sub_logger, on_dirty);
        let on_change     = mesh_registry_on_change(dirty_flag.clone_rc());
        let sub_logger    = logger.sub("mesh_registry");
        let mesh_registry = MeshRegistry::new(&context,sub_logger, on_change);
        let shape         = default();
        let listeners     = Self::init_listeners
                            (&logger,&canvas,&shape,&shape_dirty);
        let mesh_registry_dirty = dirty_flag;
        let this = Self
            {canvas,context,pixel_ratio,mesh_registry,mesh_registry_dirty
            ,shape,shape_dirty,logger,listeners};
        Ok(this)
    }
    /// Initialize all listeners and attach them to DOM elements.
    fn init_listeners
    ( logger : &Logger
    , canvas : &web_sys::HtmlCanvasElement
    , shape  : &Rc<RefCell<WorkspaceShape>>
    , dirty  : &ShapeDirty<OnDirty>
    ) -> Listeners {
        let logger = logger.clone();
        let shape  = shape.clone();
        let dirty  = dirty.clone();
        let on_resize = Closure::new(move |width, height| {
            group!(logger, "Resize observer event.", {
                *shape.borrow_mut() = WorkspaceShape {width,height};
                dirty.set();
            })
        });
        let resize = ResizeObserver::new(canvas,on_resize);
        Listeners {resize}
    }
    /// Build new instance with the provided builder object.
    pub fn build<Name:Into<String>> (name:Name) -> WorkspaceBuilder {
        let name = name.into();
        WorkspaceBuilder {name}
    }
    /// Create a new mesh instance.
    pub fn new_mesh(&mut self) -> MeshID {
        self.mesh_registry.new_mesh()
    }
    /// Resize the underlying canvas. This function should rather not be called
    /// directly. If you want to change the canvas size, modify the `shape` and
    /// set the dirty flag.
    fn resize_canvas(&self, shape:&WorkspaceShape) {
        let ratio  = self.pixel_ratio.floor() as i32;
        let width  = ratio * shape.width;
        let height = ratio * shape.height;
        self.logger.group(fmt!("Resized to {}px x {}px.", width, height), || {
            self.canvas.set_attribute("width",  &width.to_string()).unwrap();
            self.canvas.set_attribute("height", &height.to_string()).unwrap();
            self.context.viewport(0, 0, width, height);
        });
    }
    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self) {
        group!(self.logger, "Updating.", {
            if self.shape_dirty.check() {
                self.resize_canvas(&self.shape.borrow());
                self.shape_dirty.unset();
            }
            if self.mesh_registry_dirty.check() {
                self.mesh_registry.update();
                self.mesh_registry_dirty.unset();
            }

            self.logger.info("Clearing the scene.");
            self.context.clear_color(0.0, 0.0, 0.0, 1.0);
            self.context.clear(webgl::Context::COLOR_BUFFER_BIT);
            self.logger.info("Rendering meshes.");
            self.mesh_registry.render();
        })
    }
}


impl<OnDirty> Index<usize> for Workspace<OnDirty> {
    type Output = Mesh<OnDirty>;
    fn index(&self, ix: usize) -> &Self::Output {
        self.mesh_registry.index(ix)
    }
}

impl<OnDirty> IndexMut<usize> for Workspace<OnDirty> {
    fn index_mut(&mut self, ix: usize) -> &mut Self::Output {
        self.mesh_registry.index_mut(ix)
    }
}


// ========================
// === WorkspaceBuilder ===
// ========================

pub struct WorkspaceBuilder {
    pub name: String 
}

