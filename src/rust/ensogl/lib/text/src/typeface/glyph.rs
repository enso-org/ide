//! This module defines glyphs and glyphs systems. All glyphs in a glyph system share the same font,
//! but can differ in all other aspects.

use crate::prelude::*;

use super::font;
use font::GlyphRenderInfo;
use font::Font;

use ensogl::system::gpu;
use ensogl::system::gpu::texture;
use ensogl::system::gpu::types::*;
use ensogl::data::color::Rgba;
use ensogl::display;
use ensogl::display::world::*;
use ensogl::display::layout::Alignment;
use ensogl::display::symbol::material::Material;
use ensogl::display::symbol::shader::builder::CodeTemplate;
use ensogl::display::scene::Scene;



// =============
// === Glyph ===
// =============

/// Glyph texture. Contains all letters encoded in MSDF format.
pub type Texture = gpu::Texture<texture::GpuOnly,texture::Rgb,u8>;

/// A glyph rendered on screen. The displayed character will be stretched to fit the entire size of
/// underlying sprite.
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
pub struct Glyph {
    #[shrinkwrap(main_field)]
    sprite      : Sprite,
    context     : Context,
    font        : Font,
    color       : Attribute<Vector4<f32>>,
    atlas_index : Attribute<f32>,
    atlas       : Uniform<Texture>,
}

impl Glyph {
    /// Glyph color attribute accessor.
    pub fn color(&self) -> Rgba {
        self.color.get().into()
    }

    pub fn set_color(&self,color:impl Into<Rgba>) {
        self.color.set(color.into().into())
    }

    /// Change the displayed character.
    pub fn set_char(&self, ch:char) {
        let glyph_info = self.font.get_glyph_info(ch);
        self.atlas_index.set(glyph_info.msdf_texture_glyph_id as f32);
        self.update_msdf_texture();
    }

    // FIXME: How does it work? Replace with better checking.
    fn update_msdf_texture(&self) {
        let texture_changed = self.atlas.with_content(|texture| {
            texture.storage().height != self.font.msdf_texture_rows() as i32
        });
        if texture_changed {
            let width   = font::msdf::Texture::WIDTH as i32;
            let height  = self.font.msdf_texture_rows() as i32;
            let texture = Texture::new(&self.context,(width,height));
            self.font.with_borrowed_msdf_texture_data(|data| texture.reload_with_content(data));
            self.atlas.set(texture);
        }
    }
}

impl display::Object for Glyph {
    fn display_object(&self) -> &display::object::Instance {
        self.sprite.display_object()
    }
}



// ==============
// === System ===
// ==============

/// A system for displaying glyphs.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct System {
    logger        : Logger,
    context       : Context,
    sprite_system : SpriteSystem,
    pub font      : Font,
    color         : Buffer<Vector4<f32>>,
    atlas_index   : Buffer<f32>,
    atlas         : Uniform<Texture>,
}

impl System {
    /// Constructor.
    pub fn new(scene:impl AsRef<Scene>, font:Font) -> Self {
        let logger        = Logger::new("glyph_system");
        let size          = font::msdf::Texture::size();
        let scene         = scene.as_ref();
        let context       = scene.context.clone_ref();
        let sprite_system = SpriteSystem::new(scene);
        let symbol        = sprite_system.symbol();
        let texture       = Texture::new(&context,(0,0));
        let mesh          = symbol.surface();

        sprite_system.set_material(Self::material());
        sprite_system.set_alignment(Alignment::bottom_left());
        scene.variables.add("msdf_range",GlyphRenderInfo::MSDF_PARAMS.range as f32);
        scene.variables.add("msdf_size",size);
        Self {logger,context,sprite_system,font,
            atlas       : symbol.variables().add_or_panic("atlas",texture),
            color       : mesh.instance_scope().add_buffer("color"),
            atlas_index : mesh.instance_scope().add_buffer("atlas_index"),
        }
    }

    /// Create new glyph. In the returned glyph the further parameters (position,size,character)
    /// may be set.
    pub fn new_glyph(&self) -> Glyph {
        let context          = self.context.clone();
        let sprite           = self.sprite_system.new_instance();
        let instance_id      = sprite.instance_id;
        let color            = self.color.at(instance_id);
        let atlas_index = self.atlas_index.at(instance_id);
        let font             = self.font.clone_ref();
        let atlas     = self.atlas.clone();
        color.set(Vector4::new(0.0,0.0,0.0,0.0));
        atlas_index.set(0.0);
        Glyph {context,sprite,atlas_index,color,font,atlas}
    }

//    /// Create a new `Line` of text.
//    pub fn new_line(&self) -> Line {
//        Line::new(&self.logger,self)
//    }

    /// Get underlying sprite system.
    pub fn sprite_system(&self) -> &SpriteSystem {
        &self.sprite_system
    }
}

impl display::Object for System {
    fn display_object(&self) -> &display::object::Instance {
        self.sprite_system.display_object()
    }
}


// === Material ===

const FUNCTIONS : &str = include_str!("glsl/glyph.glsl");
const MAIN      : &str = "output_color = color_from_msdf(); output_id=vec4(0.0,0.0,0.0,0.0);";

impl System {
    /// Defines a default material of this system.
    fn material() -> Material {
        let mut material = Material::new();
        material.add_input_def::<texture::FloatSampler> ("atlas");
        material.add_input_def::<Vector2<f32>>          ("msdf_size");
        material.add_input_def::<f32>                   ("atlas_index");
        material.add_input("pixel_ratio", 1.0);
        material.add_input("z_zoom_1"   , 1.0);
        material.add_input("msdf_range" , GlyphRenderInfo::MSDF_PARAMS.range as f32);
        material.add_input("color"      , Vector4::new(0.0,0.0,0.0,1.0));
        // FIXME We need to use this output, as we need to declare the same amount of shader
        // FIXME outputs as the number of attachments to framebuffer. We should manage this more
        // FIXME intelligent. For example, we could allow defining output shader fragments,
        // FIXME which will be enabled only if pass of given attachment type was enabled.
        material.add_output("id", Vector4::<f32>::new(0.0,0.0,0.0,0.0));

        let code = CodeTemplate::new(FUNCTIONS,MAIN,"");
        material.set_code(code);
        material
    }
}


