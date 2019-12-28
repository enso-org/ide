use crate::prelude::*;

use crate::display::object::*;
use crate::display::world::*;
use crate::display::symbol::geometry::primitive::mesh::InstanceId;

use basegl_system_web::Logger;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Matrix4;



// =================
// === SymbolRef ===
// =================

/// Reference to a specific symbol inside the `World` object.
#[derive(Clone,Debug)]
pub struct SymbolRef {
    world     : World,
    symbol_id : SymbolId,
}

impl SymbolRef {
    pub fn new(world: World, symbol_id:SymbolId) -> Self {
        Self {world,symbol_id}
    }
}



// =================
// === SpriteRef ===
// =================

/// Reference to a specific sprite object inside a `SpriteSystem`.
#[derive(Clone,Debug)]
pub struct SpriteRef {
    symbol_ref  : SymbolRef,
    instance_id : InstanceId,
}

impl SpriteRef {
    pub fn new(symbol_ref:SymbolRef, instance_id:InstanceId) -> Self {
        Self {symbol_ref,instance_id}
    }
}



// ==============
// === Sprite ===
// ==============

/// Sprite is a simple rectangle object. In most cases, sprites always face the camera and can be
/// freely rotated only by their local z-axis. This implementation, however, implements sprites as
/// full 3D objects. We may want to fork this implementation in the future to create a specialized
/// 2d representation as well.
pub struct Sprite {
    rc: Rc<RefCell<SpriteData>>
}

// === Public API ===

impl Sprite {
    /// Modifies the position of the sprite.
    pub fn mod_position<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.rc.borrow().display_object.mod_position(f);
    }

    /// Sets the position of the sprite.
    pub fn set_position(&self, value:Vector3<f32>) {
        self.rc.borrow().display_object.set_position(value)
    }

    /// Updates the sprite and all of its children.
    pub fn update(&self) {
        self.rc.borrow().update();
    }
}


// === Private API ===

impl Sprite {
    fn new(sprite_ref:SpriteRef, transform:Var<Matrix4<f32>>, bbox:Var<Vector2<f32>>) -> Self {
        let data = SpriteData::new(sprite_ref,transform,bbox);
        let rc   = Rc::new(RefCell::new(data));
        Self {rc}
    }
}

impl From<&Sprite> for DisplayObjectData {
    fn from(t:&Sprite) -> Self {
        t.rc.borrow().display_object.clone_ref()
    }
}



// ==================
// === SpriteData ===
// ==================

struct SpriteData {
    sprite_ref     : SpriteRef,
    display_object : DisplayObjectData,
    transform      : Var<Matrix4<f32>>,
    bbox           : Var<Vector2<f32>>,
}

impl SpriteData {
    pub fn new
    (sprite_ref:SpriteRef, transform:Var<Matrix4<f32>>, bbox:Var<Vector2<f32>>) -> Self {
        let logger         = Logger::new(format!("Sprite{}",sprite_ref.instance_id));
        let display_object = DisplayObjectData::new(logger);
        let transform_cp   = transform.clone();
        display_object.set_on_updated(move |t| {
            transform_cp.set(t.matrix().clone());
        });
        Self {sprite_ref,display_object,transform,bbox}
    }
}

impl From<&SpriteData> for DisplayObjectData {
    fn from(t:&SpriteData) -> Self {
        t.display_object.clone_ref()
    }
}

impl<'t> Modify<&'t DisplayObjectData> for &'t SpriteData {
    fn modify<F:FnOnce(&'t DisplayObjectData)>(self, f:F) {
        f(&self.display_object)
    }
}



// ====================
// === SpriteSystem ===
// ====================

/// Creates a set of sprites. All sprites in the sprite system share the same material. Sprite
/// system is a very efficient way to display geometry. Sprites are rendered as instances of the
/// same mesh. Each sprite can be controlled by the instance and global attributes.
pub struct SpriteSystem {
    display_object : DisplayObjectData,
    symbol_ref     : SymbolRef,
    transform      : Buffer<Matrix4<f32>>,
    uv             : Buffer<Vector2<f32>>,
    bbox           : Buffer<Vector2<f32>>,
}

impl SpriteSystem {
    pub fn new(world:&World) -> Self {
        let logger         = Logger::new("SpriteSystem");
        let display_object = DisplayObjectData::new(logger);
        let world_data     = &mut world.borrow_mut();
        let workspace      = &mut world_data.workspace;
        let symbol_id      = workspace.new_symbol();
        let symbol         = &mut workspace[symbol_id];
        let mesh           = &mut symbol.surface;
        let uv             = mesh.scopes.point.add_buffer("uv");
        let transform      = mesh.scopes.instance.add_buffer("transform");
        let bbox           = mesh.scopes.instance.add_buffer("bbox");

        let p1_index = mesh.scopes.point.add_instance();
        let p2_index = mesh.scopes.point.add_instance();
        let p3_index = mesh.scopes.point.add_instance();
        let p4_index = mesh.scopes.point.add_instance();

        uv.get(p1_index).set(Vector2::new(0.0, 0.0));
        uv.get(p2_index).set(Vector2::new(0.0, 1.0));
        uv.get(p3_index).set(Vector2::new(1.0, 0.0));
        uv.get(p4_index).set(Vector2::new(1.0, 1.0));

        let world      = world.clone_ref();
        let symbol_ref = SymbolRef::new(world,symbol_id);
        Self {display_object,symbol_ref,transform,uv,bbox}
    }

    pub fn new_instance(&self) -> Sprite {
        let world_data   = &mut self.symbol_ref.world.borrow_mut();
        let symbol       = &mut world_data.workspace[self.symbol_ref.symbol_id];
        let instance_id  = symbol.surface.instance.add_instance();
        let transform    = self.transform.get(instance_id);
        let bbox         = self.bbox.get(instance_id);
        let sprite_ref   = SpriteRef::new(self.symbol_ref.clone(),instance_id);
        bbox.set(Vector2::new(2.0,2.0));
        let sprite = Sprite::new(sprite_ref,transform,bbox);
        self.add_child(&sprite);
        sprite
    }
}

impl From<&SpriteSystem> for DisplayObjectData {
    fn from(t:&SpriteSystem) -> Self {
        t.display_object.clone_ref()
    }
}

impl<'t> Modify<&'t DisplayObjectData> for &'t SpriteSystem {
    fn modify<F:FnOnce(&'t DisplayObjectData)>(self, f:F) {
        f(&self.display_object)
    }
}
