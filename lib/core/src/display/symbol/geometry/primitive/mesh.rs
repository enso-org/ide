#![allow(missing_docs)]

pub use crate::system::gpu::data;


pub mod types {
    use super::*;
    pub use crate::system::gpu::data::types::*;
    pub use crate::system::gpu::data::attribute::types::*;
    pub use super::Mesh;
}
pub use types::*;

use types::Buffer;
use types::Attribute;



use crate::closure;
use crate::data::dirty::traits::*;
use crate::data::dirty;
use crate::data::function::callback::*;
use crate::debug::stats::Stats;
use crate::display::render::webgl::Context;
use crate::prelude::*;
//use crate::promote;
//use crate::promote_all;
//use crate::promote_scope_types;
use crate::system::web::group;
use crate::system::web::Logger;
use eval_tt::*;
use num_enum::IntoPrimitive;





// ============
// === Mesh ===
// ============

// === Definition ===

/// A polygon mesh is a collection of vertices, edges and faces that defines the shape of a
/// polyhedral object. Mesh describes the shape of the display element. It consist of several
/// scopes containing sets of variables.
///
///   - Point Scope
///     A point is simply a point in space. Points are often assigned with such variables as
///     'position' or 'color'.
///
///   - Vertex Scope
///     A vertex is a reference to a point. Primitives use vertices to reference points. For
///     example, the corners of a polygon, the center of a sphere, or a control vertex of a spline
///     curve. Primitives can share points, while vertices are unique to a primitive.
///
///   - Primitive Scope
///     Primitives refer to a unit of geometry, lower-level than an object but above points. There
///     are several different types of primitives, including polygon faces or Bezier/NURBS surfaces.
///
///   - Instance Scope
///     Instances are virtual copies of the same geometry. They share point, vertex, and primitive
///     variables.
///
///   - Object Scope
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
#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Mesh {
    #[shrinkwrap(main_field)]
    pub scopes       : Scopes,
    pub scopes_dirty : ScopesDirty,
    pub logger       : Logger,
    context          : Context,
    stats            : Stats,
}

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Scopes {
    pub point     : AttributeScope,
    pub vertex    : AttributeScope,
    pub primitive : AttributeScope,
    pub instance  : AttributeScope,
}

pub type PointId     = usize;
pub type VertexId    = usize;
pub type PrimitiveId = usize;
pub type InstanceId  = usize;

#[derive(Copy,Clone,Debug,IntoPrimitive,PartialEq)]
#[repr(u8)]
pub enum ScopeType {
    Point, Vertex, Primitive, Instance
}

impl From<ScopeType> for usize {
    fn from(t: ScopeType) -> Self {
        Into::<u8>::into(t).into()
    }
}

impl Display for ScopeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{:?}",self)
    }
}


// === Types ===

pub type ScopesDirty = dirty::SharedEnum<u8,ScopeType,Box<dyn Fn()>>;
//promote_scope_types!{ [ScopeOnChange] data }

//#[macro_export]
///// Promote relevant types to parent scope. See `promote!` macro for more information.
//macro_rules! promote_mesh_types { ($($args:tt)*) => {
////    crate::promote_scope_types! { $($args)* }
//    promote! {$($args)* [Mesh,Scopes]}
//};}



// === Callbacks ===

closure! {
fn scope_on_change(dirty:ScopesDirty, item:ScopeType) -> ScopeOnChange {
    || dirty.set(item)
}}


// === Implementation ===

macro_rules! update_scopes { ($self:ident . {$($name:ident),*} {$($uname:ident),*}) => {$(
    if $self.scopes_dirty.check(&ScopeType::$uname) {
        $self.scopes.$name.update()
    }
)*}}

impl Mesh {

    /// Creates new mesh with attached dirty callback.
    pub fn new<OnMut:Fn()+'static>(logger:Logger, stats:&Stats, context:&Context,on_mut:OnMut) -> Self {
        stats.inc_mesh_count();
        let stats         = stats.clone();
        let scopes_logger = logger.sub("scopes_dirty");
        let scopes_dirty  = ScopesDirty::new(scopes_logger,Box::new(on_mut));
        let context       = context.clone();
        let scopes        = group!(logger, "Initializing.", {
            macro_rules! new_scope { ($cls:ident { $($name:ident),* } { $($uname:ident),* } ) => {$(
                let sub_logger = logger.sub(stringify!($name));
                let status_mod = ScopeType::$uname;
                let scs_dirty  = scopes_dirty.clone_ref();
                let callback   = move || {scs_dirty.set(status_mod)};
                let $name      = $cls::new(sub_logger,&stats,&context,callback);
            )*}}
            new_scope!(AttributeScope {point,vertex,primitive,instance}{Point,Vertex,Primitive,Instance});
            Scopes {point,vertex,primitive,instance}
        });
        Self {context,scopes,scopes_dirty,logger,stats}
    }

    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self) {
        group!(self.logger, "Updating.", {
            if self.scopes_dirty.check_all() {
                update_scopes!(self.{point,vertex,primitive,instance}
                                    {Point,Vertex,Primitive,Instance});
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
    pub fn scope_by_type(&self, scope_type:ScopeType) -> &AttributeScope {
        match scope_type {
            ScopeType::Point     => &self.scopes.point,
            ScopeType::Vertex    => &self.scopes.vertex,
            ScopeType::Primitive => &self.scopes.primitive,
            ScopeType::Instance  => &self.scopes.instance,
        }
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        self.stats.dec_mesh_count();
    }
}
