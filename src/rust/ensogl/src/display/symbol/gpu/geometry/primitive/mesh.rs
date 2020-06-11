//! This module defines a [polygon mesh](https://en.wikipedia.org/wiki/Polygon_mesh).

use crate::prelude::*;

use crate::control::callback::CallbackFn;
use crate::data::dirty::traits::*;
use crate::data::dirty;
use crate::debug::stats::Stats;
use crate::system::gpu::shader::Context;

use num_enum::IntoPrimitive;

use shapely::shared;




// ===============
// === Exports ===
// ===============

/// Common data types.
pub mod types {
    pub use crate::system::gpu::types::*;
    pub use super::Mesh;
}
pub use types::*;


// --------------------------------------------------

/// Container for all scopes owned by a mesh.
#[derive(Debug)]
pub struct Scopes {
    /// Point Scope. A point is simply a point in space. Points are often assigned with such
    /// variables as 'position' or 'color'.
    pub point: AttributeScope,

    /// Vertex Scope. A vertex is a reference to a point. Primitives use vertices to reference
    /// points. For example, the corners of a polygon, the center of a sphere, or a control vertex
    /// of a spline curve. Primitives can share points, while vertices are unique to a primitive.
    pub vertex: AttributeScope,

    /// Primitive Scope. Primitives refer to a unit of geometry, lower-level than an object but
    /// above points. There are several different types of primitives, including polygon faces or
    /// Bezier/NURBS surfaces.
    pub primitive: AttributeScope,

    /// Instance Scope. Instances are virtual copies of the same geometry. They share point, vertex,
    /// and primitive variables.
    pub instance: AttributeScope,
}

/// A singleton for each of scope types.
#[derive(Copy,Clone,Debug,Display,IntoPrimitive,PartialEq)]
#[allow(missing_docs)]
#[repr(u8)]
pub enum ScopeType {Point,Vertex,Primitive,Instance}

impl From<ScopeType> for usize {
    fn from(t: ScopeType) -> Self {
        Into::<u8>::into(t).into()
    }
}


// === Types ===

/// Dirty flag remembering which scopes were mutated.
pub type ScopesDirty = dirty::SharedEnum<u8,ScopeType,Box<dyn Fn()>>;


// === Implementation ===

macro_rules! update_scopes {
    ($self:ident . {$($name:ident),*} {$($uname:ident),*}) => {$(
        if $self.scopes_dirty.check(&ScopeType::$uname) {
            $self.scopes.$name.update()
        }
    )*}
}


// ============
// === Mesh ===
// ============

// === Definition ===

shared! { Mesh
/// A polygon mesh is a collection of vertices, edges and faces that defines the shape of a
/// polyhedral object. Mesh describes the shape of the display element. It consist of several
/// scopes containing sets of variables. See the documentation of `Scopes` to learn more.
///
/// Please note, that there are other, higher-level scopes defined by other structures, including:
///
///   - Symbol Scope
///     Object refers to the whole geometry with all of its instances.
///
///   - Global Scope
///     Global scope is shared by all objects and it contains some universal global variables, like
///     the current 'time' counter.
///
/// Each scope can contain named attributes which can be accessed from within materials. If the same
/// name was defined in various scopes, it gets resolved to the var defined in the most specific
/// scope. For example, if var 'color' was defined in both 'instance' and 'point' scope, the 'point'
/// definition overlapps the other one.
#[derive(Debug)]
pub struct MeshData {
    scopes       : Scopes,
    scopes_dirty : ScopesDirty,
    logger       : Logger,
    context      : Context,
    stats        : Stats,
}

impl {
    /// Creates new mesh with attached dirty callback.
    pub fn new<OnMut:CallbackFn>
    (logger:Logger, stats:&Stats, context:&Context,on_mut:OnMut) -> Self {
        stats.inc_mesh_count();
        let stats         = stats.clone();
        let scopes_logger = Logger::sub(&logger,"scopes_dirty");
        let scopes_dirty  = ScopesDirty::new(scopes_logger,Box::new(on_mut));
        let context       = context.clone();
        let scopes        = group!(logger, "Initializing.", {
            macro_rules! new_scope { ({ $($name:ident),* } { $($uname:ident),* } ) => {$(
                let sub_logger = Logger::sub(&logger,stringify!($name));
                let status_mod = ScopeType::$uname;
                let scs_dirty  = scopes_dirty.clone_ref();
                let callback   = move || {scs_dirty.set(status_mod)};
                let $name      = AttributeScope::new(sub_logger,&stats,&context,callback);
            )*}}
            new_scope! ({point,vertex,primitive,instance}{Point,Vertex,Primitive,Instance});
            Scopes {point,vertex,primitive,instance}
        });
        Self {context,scopes,scopes_dirty,logger,stats}
    }

    /// Point scope accessor.
    pub fn point_scope(&self) -> AttributeScope {
        self.scopes.point.clone_ref()
    }

    /// Vertex scope accessor.
    pub fn vertex_scope(&self) -> AttributeScope {
        self.scopes.vertex.clone_ref()
    }

    /// Primitive scope accessor.
    pub fn primitive_scope(&self) -> AttributeScope {
        self.scopes.primitive.clone_ref()
    }

    /// Instance scope accessor.
    pub fn instance_scope(&self) -> AttributeScope {
        self.scopes.instance.clone_ref()
    }

    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self) {
        group!(self.logger, "Updating.", {
            if self.scopes_dirty.check_all() {
                update_scopes!{
                    self.{point,vertex,primitive,instance}{Point,Vertex,Primitive,Instance}
                }
                self.scopes_dirty.unset_all()
            }
        })
    }

    /// Browses all scopes and finds where a variable was defined. Scopes are browsed in a
    /// hierarchical order. To learn more about the ordering see the documentation of `Mesh`.
    pub fn lookup_variable<S:Str>(&self, name:S) -> Option<ScopeType> {
        let name = name.as_ref();
        if      self.scopes.point     . contains(name) { Some(ScopeType::Point)     }
        else if self.scopes.vertex    . contains(name) { Some(ScopeType::Vertex)    }
        else if self.scopes.primitive . contains(name) { Some(ScopeType::Primitive) }
        else if self.scopes.instance  . contains(name) { Some(ScopeType::Instance)  }
        else {None}
    }

    /// Gets reference to scope based on the scope type.
    pub fn scope_by_type(&self, scope_type:ScopeType) -> AttributeScope {
        match scope_type {
            ScopeType::Point     => &self.scopes.point,
            ScopeType::Vertex    => &self.scopes.vertex,
            ScopeType::Primitive => &self.scopes.primitive,
            ScopeType::Instance  => &self.scopes.instance,
        }.clone_ref()
    }
}}

impl Drop for MeshData {
    fn drop(&mut self) {
        self.stats.dec_mesh_count();
    }
}
