use crate::prelude::*;

use crate::backend::webgl;
use crate::backend::webgl::Context;
use crate::closure;
use crate::data::function::callback::*;
use crate::dirty;
use crate::dirty::traits::*;
use crate::display::symbol::geometry;
use crate::promote;
use crate::promote_all;
use crate::promote_geometry_types;
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
    pub geometry_dirty : GeometryDirty <OnDirty>,
    pub logger         : Logger,
    context            : Context
}

// === Types ===

pub type GeometryDirty<Callback> = dirty::SharedBool<Callback>;
promote_geometry_types!{ [OnGeometryChange] geometry }

#[macro_export]
macro_rules! promote_mesh_types { ($($args:tt)*) => {
    crate::promote_geometry_types! {$($args)*}
    promote! {$($args)* [Mesh]}
};}

// === Callbacks ===

closure! {
fn geometry_on_change<C:Callback0>(dirty:GeometryDirty<C>) ->
    OnGeometryChange { || dirty.set() }
}

// === Implementation ===

impl<OnDirty:Callback0> Mesh<OnDirty> {
    /// Create new instance with the provided on-dirty callback.
    pub fn new(context:&Context, logger:Logger, on_dirty:OnDirty) -> Self {
        let geometry_logger = logger.sub("geometry_dirty");
        let geometry_dirty  = GeometryDirty::new(geometry_logger,on_dirty);
        let geo_on_change   = geometry_on_change(geometry_dirty.clone_rc());
        let context         = context.clone();
        let geometry        = group!(logger, "Initializing.", {
            Geometry::new(&context,logger.sub("geometry"),geo_on_change)
        });
        Mesh {geometry,geometry_dirty,logger,context}
    }
    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self) {
        group!(self.logger, "Updating.", {
            if self.geometry_dirty.check_and_unset() {
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

            println!("-----------");

            // === Rendering ==

            self.context.use_program(Some(&program));

            self.context.enable_vertex_attrib_array(pos_loc);
            self.geometry.scopes.point.buffers[0].bind(webgl::Context::ARRAY_BUFFER);

            // hidden part: binds ARRAY_BUFFER to the attribute
            self.context.vertex_attrib_pointer_with_i32(
                pos_loc,
                3,                     // size - 3 components per iteration
                webgl::Context::FLOAT, // type
                false,                 // normalize
                0,                     // stride
                0,                     // offset
            );

            self.context.draw_arrays(webgl::Context::TRIANGLES, 0, (3) as i32);

            println!("{:?}",&self.geometry.scopes.point.buffers[0]);
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

impl<OnDirty:Callback0> SharedMesh<OnDirty> {
    /// Create new instance with the provided on-dirty callback.
    pub fn new(context:&Context, logger:Logger, on_dirty:OnDirty) -> Self {
        let raw = RefCell::new(Mesh::new(context,logger, on_dirty));
        Self { raw }
    }
}