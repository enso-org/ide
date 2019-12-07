use crate::prelude::*;

use crate::backend::webgl;
use crate::backend::webgl::Context;
use crate::closure;
use crate::data::function::callback::*;
use crate::dirty;
use crate::dirty::traits::*;
use crate::display::symbol::geometry;
use crate::display::symbol::material;
use crate::promote;
use crate::promote_all;
use crate::promote_geometry_types;
use crate::promote_material_types;
use crate::system::web::Logger;
use crate::system::web::group;
use eval_tt::*;

use crate::display::symbol::buffer::IsBuffer;

// ============
// === Mesh ===
// ============

// === Definition ===

/// Mesh is a `Geometry` with attached `Material`.
#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Mesh<OnDirty> {
    #[shrinkwrap(main_field)]
    pub geometry       : Geometry      <OnDirty>,
    pub material       : Material      <OnDirty>,
    pub geometry_dirty : GeometryDirty <OnDirty>,
    pub material_dirty : MaterialDirty <OnDirty>,
    pub logger         : Logger,
    context            : Context
}

// === Types ===

pub type GeometryDirty<Callback> = dirty::SharedBool<Callback>;
pub type MaterialDirty<Callback> = dirty::SharedBool<Callback>;
promote_geometry_types!{ [OnGeometryChange] geometry }
promote_material_types!{ [OnGeometryChange] material }

#[macro_export]
macro_rules! promote_mesh_types { ($($args:tt)*) => {
    crate::promote_geometry_types! {$($args)*}
    crate::promote_material_types! {$($args)*}
    promote! {$($args)* [Mesh]}
};}

// === Callbacks ===

closure! {
fn geometry_on_change<C:Callback0>(dirty:GeometryDirty<C>) ->
    OnGeometryChange { || dirty.set() }
}

closure! {
fn material_on_change<C:Callback0>(dirty:MaterialDirty<C>) ->
    OnMaterialChange { || dirty.set() }
}

// === Implementation ===

impl<OnDirty:Callback0+Clone> Mesh<OnDirty> {
    /// Create new instance with the provided on-dirty callback.
    pub fn new(ctx:&Context, logger:Logger, on_dirty:OnDirty) -> Self {
        let init_logger = logger.clone();
        group!(init_logger, "Initializing.", {
            let context         = ctx.clone();
            let on_dirty2       = on_dirty.clone();
            let geo_logger      = logger.sub("geometry");
            let mat_logger      = logger.sub("material");
            let geo_dirt_logger = logger.sub("geometry_dirty");
            let mat_dirt_logger = logger.sub("material_dirty");
            let geometry_dirty  = GeometryDirty::new(geo_dirt_logger,on_dirty2);
            let material_dirty  = MaterialDirty::new(mat_dirt_logger,on_dirty);
            let geo_on_change   = geometry_on_change(geometry_dirty.clone_rc());
            let mat_on_change   = material_on_change(material_dirty.clone_rc());
            let material        = Material::new(ctx,mat_logger,mat_on_change);
            let geometry        = Geometry::new(ctx,geo_logger,geo_on_change);
            Self{geometry,material,geometry_dirty,material_dirty,logger,context}
        })
    }
    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self) {
        group!(self.logger, "Updating.", {
            if self.geometry_dirty.check_and_unset_all() {
                self.geometry.update()
            }
        })
    }

    pub fn render(&self) {
        group!(self.logger, "Rendering.", {
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

            let pos_loc = self.context.get_attrib_location(&program, "position");
            let pos_loc = pos_loc as u32;

//            println!("-----------");

            // === Rendering ==

            self.context.use_program(Some(&program));

            self.context.enable_vertex_attrib_array(pos_loc);
            let buffer = &self.geometry.scopes.point.buffers[0];
            buffer.bind(webgl::Context::ARRAY_BUFFER);
            buffer.vertex_attrib_pointer(pos_loc);

            self.context.draw_arrays(webgl::Context::TRIANGLES, 0, buffer.len() as i32);

//            println!("{:?}",&self.geometry.scopes.point.buffers[0]);
        })
    }
}

// ==================
// === SharedMesh ===
// ==================

/// A shared version of `Mesh`.
#[derive(Shrinkwrap)]
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct SharedMesh<OnDirty> {
    pub raw: RefCell<Mesh<OnDirty>>
}

impl<OnDirty:Callback0+Clone> SharedMesh<OnDirty> {
    /// Create new instance with the provided on-dirty callback.
    pub fn new(context:&Context, logger:Logger, on_dirty:OnDirty) -> Self {
        let raw = RefCell::new(Mesh::new(context,logger, on_dirty));
        Self { raw }
    }
}