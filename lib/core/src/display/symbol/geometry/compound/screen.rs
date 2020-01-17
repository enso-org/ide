//! This module defines a geometry which always covers the whole screen. An example use case is
//! render pass implementation - rendering to framebuffer and then using the result with some
//! post-processing effect by applying the previous output to a screen covering geometry.


use crate::prelude::*;

use crate::display::symbol::material::Material;
use crate::system::gpu::data::AttributeInstanceIndex;
use crate::display::world::*;

use nalgebra::Vector2;



// =================
// === SymbolRef ===
// =================

/// Reference to a specific symbol inside the `World` object.
#[derive(Clone,Debug)]
pub struct SymbolRef {
    world     : World,
    pub symbol_id : SymbolId,
}

impl SymbolRef {
    /// Constructor.
    pub fn new(world:World, symbol_id:SymbolId) -> Self {
        Self {world,symbol_id}
    }
}



// ==============
// === Screen ===
// ==============

/// A whole-screen covering geometry.
pub struct Screen {
    pub symbol_ref : SymbolRef,
    _uv        : Buffer<Vector2<f32>>,
}

impl Screen {
    /// Constructor.
    pub fn new(world:&World) -> Self {
        let world_data     = &mut world.borrow_mut();
        let workspace      = &mut world_data.workspace;
        let symbol_id      = workspace.new_symbol();
        let symbol         = &mut workspace[symbol_id];
        let mesh           = &mut symbol.surface;
        let uv             = mesh.scopes.point.add_buffer("uv");

        let geometry_material = Self::geometry_material();
        let surface_material  = Self::surface_material();

        symbol.shader.set_geometry_material (&geometry_material);
        symbol.shader.set_material          (&surface_material);

        let p1_index = mesh.scopes.point.add_instance();
        let p2_index = mesh.scopes.point.add_instance();
        let p3_index = mesh.scopes.point.add_instance();
        let p4_index = mesh.scopes.point.add_instance();

        uv.at(p1_index).set(Vector2::new(0.0, 0.0));
        uv.at(p2_index).set(Vector2::new(0.0, 1.0));
        uv.at(p3_index).set(Vector2::new(1.0, 0.0));
        uv.at(p4_index).set(Vector2::new(1.0, 1.0));

        world_data.stats.inc_sprite_system_count();

        let world      = world.clone_ref();
        let symbol_ref = SymbolRef::new(world,symbol_id);
        Self {symbol_ref,_uv:uv}
    }

    fn geometry_material() -> Material {
        let mut material = Material::new();
        material.add_input_def::<Vector2<f32>>("uv");
        material.set_main("gl_Position = vec4((input_uv-0.5)*2.0,0.0,1.0);");
        material
    }

    fn surface_material() -> Material {
        let mut material = Material::new();
        material.set_main("output_color = vec4(0.0,1.0,0.0,1.0);");
        material
    }
}



// === Setters ===

impl Screen {
    /// Sets the material for the geometry.
    pub fn set_material<M:Into<Material>>(&mut self, material:M) {
        let world_data = &mut self.symbol_ref.world.borrow_mut();
        let symbol     = &mut world_data.workspace[self.symbol_ref.symbol_id];
        symbol.shader.set_material(material);
    }
}
