use crate::prelude::*;

use crate::dirty;
use crate::backend::webgl;
use crate::system::web;
use crate::system::web::group;
use crate::system::web::fmt;
use crate::system::web::resize_observer::ResizeObserver;
use crate::system::web::Logger;
use wasm_bindgen::prelude::Closure;
use web_sys::WebGlRenderingContext;
use crate::closure;
use crate::data::function::callback::*;
use crate::display::mesh_registry;
use crate::dirty::traits::*;
pub use crate::display::mesh_registry::MeshID;

// =============
// === Error ===
// =============

#[derive(Debug, Fail, From)]
pub enum Error {
    #[fail(display = "Web Platform error: {}", error)]
    WebError { error: web::Error },
}

// =============
// === Types ===
// =============

pub type ID = usize;

// =================
// === Workspace ===
// =================

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Workspace<OnDirty> {
    pub canvas              : web_sys::HtmlCanvasElement,
    pub context             : WebGlRenderingContext,
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

pub type AttributeIndex <T, Callback> = mesh_registry::AttributeIndex<T, Closure_mesh_registry_on_dirty<Callback>>;
pub type Mesh           <Callback> = mesh_registry::Mesh           <Closure_mesh_registry_on_dirty<Callback>>;
pub type Geometry       <Callback> = mesh_registry::Geometry       <Closure_mesh_registry_on_dirty<Callback>>;
pub type Scopes         <Callback> = mesh_registry::Scopes         <Closure_mesh_registry_on_dirty<Callback>>;
pub type AttributeScope <Callback> = mesh_registry::AttributeScope <Closure_mesh_registry_on_dirty<Callback>>;
pub type UniformScope   <Callback> = mesh_registry::UniformScope   <Closure_mesh_registry_on_dirty<Callback>>;
pub type GlobalScope    <Callback> = mesh_registry::GlobalScope    <Closure_mesh_registry_on_dirty<Callback>>;
pub type Attribute   <T, Callback> = mesh_registry::Attribute   <T, Closure_mesh_registry_on_dirty<Callback>>;
pub type MeshRegistry   <Callback> = mesh_registry::MeshRegistry   <Closure_mesh_registry_on_dirty<Callback>>;

// === Callbacks ===

closure!(mesh_registry_on_dirty<Callback: Callback0>
    (dirty: MeshRegistryDirty<Callback>) || { dirty.set() });

// === Implementation ===

#[derive(Debug)]
pub struct Listeners {
    resize: ResizeObserver,
}

impl<OnDirty: Clone + Callback0 + 'static> Workspace<OnDirty> {
    pub fn new <Dom: Str>
    (dom: Dom, logger: Logger, on_dirty: OnDirty) -> Result<Self, Error> {
        logger.trace("Initializing.");
        let dom     = dom.as_ref();
        let canvas  = web::get_canvas(dom)?;
        let context = web::get_webgl_context(&canvas, 1)?;

        let shape_dirty_logger = logger.sub("shape_dirty");
        let shape_dirty        = ShapeDirty::new(shape_dirty_logger, on_dirty.clone());

        let mesh_registry_dirty_logger = logger.sub("mesh_registry_dirty");
        let mesh_registry_dirty = MeshRegistryDirty::new(mesh_registry_dirty_logger, on_dirty);

        let mesh_registry_on_dirty = mesh_registry_on_dirty(mesh_registry_dirty.clone());
        let mesh_registry_logger = logger.sub("mesh_registry");
        let mesh_registry        = MeshRegistry::new(mesh_registry_logger, mesh_registry_on_dirty);

        let shape     = default();
        let listeners = Self::new_listeners(&canvas, &shape, &shape_dirty);
        Ok(Self { canvas, context, mesh_registry, mesh_registry_dirty, shape, shape_dirty, logger, listeners })
    }

    pub fn build<Name: Into<String>> (name: Name) -> WorkspaceBuilder {
        let name = name.into();
        WorkspaceBuilder { name }
    }

    pub fn new_listeners(canvas: &web_sys::HtmlCanvasElement, shape: &Rc<RefCell<WorkspaceShape>>, dirty: &ShapeDirty<OnDirty>) -> Listeners {
        let shape = shape.clone();
        let dirty = dirty.clone();
        let on_resize = Closure::new(move |width, height| {
            *shape.borrow_mut() = WorkspaceShape { width, height };
            dirty.set();
        });
        let resize = ResizeObserver::new(canvas, on_resize);
        Listeners { resize }
    }

    pub fn new_mesh(&mut self) -> MeshID {
        self.mesh_registry.new_mesh()
    }

    pub fn is_dirty(&self) -> bool {
        self.shape_dirty.check() || self.mesh_registry_dirty.check()
    }

    fn resize_canvas(&self, shape: &WorkspaceShape) {
        let width  = shape.width;
        let height = shape.height;
        self.logger.group(fmt!("Resized to {}px x {}px.", width, height), || {
            self.canvas.set_attribute("width", &width.to_string()).unwrap();
            self.canvas.set_attribute("height", &height.to_string()).unwrap();
            self.context.viewport(0, 0, width, height);
        });
    }

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
        
            let vert_shader = webgl::compile_shader(
                &self.context,
                webgl::Context::VERTEX_SHADER,
                r#"
    attribute vec4 position;
    void main() {
        gl_Position = position;
    }
"#,
            )
            .unwrap();
            let frag_shader = webgl::compile_shader(
                &self.context,
                webgl::Context::FRAGMENT_SHADER,
                r#"
    void main() {
        gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
    }
"#,
            )
            .unwrap();
            let program =
                webgl::link_program(&self.context, &vert_shader, &frag_shader).unwrap();
            self.context.use_program(Some(&program));

            let vertices: [f32; 9] = [-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0];

            let buffer = self.context.create_buffer().ok_or("failed to create buffer").unwrap();
            self.context.bind_buffer(webgl::Context::ARRAY_BUFFER, Some(&buffer));

            // Note that `Float32Array::view` is somewhat dangerous (hence the
            // `unsafe`!). This is creating a raw view into our module's
            // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
            // (aka do a memory allocation in Rust) it'll cause the buffer to change,
            // causing the `Float32Array` to be invalid.
            //
            // As a result, after `Float32Array::view` we have to be very careful not to
            // do any memory allocations before it's dropped.
            unsafe {
                let vert_array = js_sys::Float32Array::view(&vertices);

                self.context.buffer_data_with_array_buffer_view(
                    webgl::Context::ARRAY_BUFFER,
                    &vert_array,
                    webgl::Context::STATIC_DRAW,
                );
            }

            self.context.vertex_attrib_pointer_with_i32(
                0,
                3,
                webgl::Context::FLOAT,
                false,
                0,
                0,
            );
            self.context.enable_vertex_attrib_array(0);

            self.context.clear_color(0.0, 0.0, 0.0, 1.0);
            self.context.clear(webgl::Context::COLOR_BUFFER_BIT);

            self.context.draw_arrays(webgl::Context::TRIANGLES, 0, (vertices.len() / 3) as i32);
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

// // =====================
// // === WorkspaceData ===
// // =====================

// // === Definition ===

// #[derive(Debug)]
// pub struct WorkspaceData<OnDirty> {
//     pub canvas:  web_sys::HtmlCanvasElement,
//     pub context: WebGlRenderingContext,
//     pub shape_dirty : ShapeDirty<OnDirty>,
//     pub logger:  Logger,
//     pub dirty:   SharedSimple,
// }

// #[derive(Default)]
// pub struct WorkspaceShape {
//     pub width  : f32,
//     pub height : f32,
// }

// pub type WorkspaceShapeDirtyState = WorkspaceShape;

// // === Types ===

// pub type ShapeDirty <Callback> = dirty::SharedCustom<WorkspaceShapeDirtyState, Callback>;

// // === Callbacks ===

// // closure!(shape_on_change<Callback: Callback0>
// //     (dirty: ShapeDirty<Callback>, action: fn(&mut WorkspaceShapeDirtyState)) 
// //         || { dirty.set(action) });

// // === Implementation ===

// impl<OnDirty> WorkspaceData<OnDirty> {
//     pub fn new(
//         dom: &str,
//         logger: Logger,
//         sup_dirty: &SharedSimple,
//         on_dirty: OnDirty,
//     ) -> Result<Self, Error>
//     {
//         logger.trace("Initializing.");
//         let canvas = web::get_canvas(dom)?;
//         let context = web::get_webgl_context(&canvas, 1)?;
//         let dirty = sup_dirty.new_child(&logger);

//         let shape_dirty_logger = logger.sub("shape_dirty");
//         let shape_dirty        = ShapeDirty::new(on_dirty, shape_dirty_logger);
//         Ok(Self { canvas, context, shape_dirty, logger, dirty })
//     }

//     pub fn resize(&self, width: i32, height: i32) {
//         self.logger.group(fmt!("Resized to {}px x {}px.", width, height), || {
//             self.canvas.set_attribute("width", &width.to_string()).unwrap();
//             self.canvas.set_attribute("height", &height.to_string()).unwrap();
//             self.context.viewport(0, 0, width, height);
//             self.dirty.set();
//         });
//     }
// }
