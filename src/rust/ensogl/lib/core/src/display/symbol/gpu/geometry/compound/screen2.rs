//! This module defines a geometry which always covers the whole screen. An example use case is
//! render pass implementation - rendering to framebuffer and then using the result with some
//! post-processing effect by applying the previous output to a screen covering geometry.


use crate::prelude::*;

use crate::display::symbol::geometry::Sprite;
use crate::display::symbol::geometry::SpriteSystem;
use crate::display::symbol::material::Material;
use crate::display::scene::Scene;
use crate::system::gpu::data::texture;
use crate::system::gpu::data::types::*;



/// Defines a system containing shapes. It is a specialized `SpriteSystem` version.
#[derive(Clone,CloneRef,Debug)]
pub struct Screen2 {
    sprite        : Sprite,
    sprite_system : SpriteSystem,
}

impl Screen2 {
    /// Constructor.
    pub fn new(scene:&Scene, mask_input:impl AsRef<str>, color_input:impl AsRef<str>) -> Self {
        let sprite_system = SpriteSystem::new(scene);
        sprite_system.set_geometry_material(Self::geometry_material());
        sprite_system.set_material(Self::surface_material(mask_input,color_input));
        let sprite = sprite_system.new_instance();
        Self {sprite,sprite_system}
    }

    /// Hide the symbol. Hidden symbols will not be rendered.
    pub fn hide(&self) {
        self.sprite_system.hide();
    }

    /// Show the symbol. It will be rendered on next render call.
    pub fn show(&self) {
        self.sprite_system.show();
    }

    /// Local variables used by the screen object.
    pub fn variables(&self) -> UniformScope {
        self.sprite_system.symbol().variables().clone()
    }

    /// Render the shape.
    pub fn render(&self) {
        self.sprite_system.render()
    }
}


// === Materials ===

impl Screen2 {
    fn geometry_material() -> Material {
        let mut material = Material::new();
        material.add_input_def::<Vector2<f32>>("uv");
        material.set_main("gl_Position = vec4((input_uv-0.5)*2.0,0.0,1.0);");
        material
    }

    fn surface_material(mask_input:impl AsRef<str>,color_input:impl AsRef<str>) -> Material {
        let mask_input   = mask_input.as_ref();
        let color_input  = color_input.as_ref();
        let mut material = Material::new();
        let shader       = iformat!("
            vec4 sample_mask  = texture(input_{mask_input},input_uv);
            vec4 sample_color = texture(input_{color_input},input_uv);
            output_color = sample_color;
            output_color.a *= sample_mask.a;
            output_id=vec4(0.0,0.0,0.0,0.0);
            ");
        material.add_input_def::<texture::FloatSampler>(mask_input);
        material.add_input_def::<texture::FloatSampler>(color_input);
        material.add_output ("id", Vector4::<f32>::new(0.0,0.0,0.0,0.0));
        material.set_main(shader);
        material
    }
}
