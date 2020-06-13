
use crate::prelude::*;

use ensogl::system::gpu;
use ensogl::system::gpu::texture;
use ensogl::system::gpu::types::*;
use crate::display;
use crate::display::shape::text::glyph::font;
use crate::display::shape::text::glyph::msdf;
use crate::display::world::*;
use crate::display::layout::Alignment;
use crate::display::shape::text::glyph::font::GlyphRenderInfo;
use crate::display::symbol::material::Material;
use crate::display::symbol::shader::builder::CodeTemplate;
use crate::display::scene::Scene;



// =============
// === Glyph ===
// =============


pub type Texture = gpu::Texture<texture::GpuOnly,texture::Rgb,u8>;

/// A glyph rendered on screen. The displayed character will be stretched to fit the entire size of
/// underlying sprite.
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
pub struct Glyph {
    #[shrinkwrap(main_field)]
    sprite       : Sprite,
    context      : Context,
    font         : font::Handle,
    color_attr   : Attribute<Vector4<f32>>,
    msdf_index   : Attribute<f32>,
    msdf_texture : Uniform<Texture>,
}

impl Glyph {
    /// Glyph color attribute accessor.
    pub fn color(&self) -> Attribute<Vector4<f32>> {
        self.color_attr.clone_ref()
    }

    /// Change the displayed character.
    pub fn set_glyph(&self, ch:char) {
        let glyph_info = self.font.get_glyph_info(ch);
        self.msdf_index.set(glyph_info.msdf_texture_glyph_id as f32);
        self.update_msdf_texture();
    }

    fn update_msdf_texture(&self) {
        /// FIXME: How does it work? Replace with better checking.
        let texture_changed = self.msdf_texture.with_content(|texture| {
            texture.storage().height != self.font.msdf_texture_rows() as i32
        });
        if texture_changed {
            let width   = msdf::Texture::WIDTH as i32;
            let height  = self.font.msdf_texture_rows() as i32;
            let texture = Texture::new(&self.context,(width,height));
            self.font.with_borrowed_msdf_texture_data(|data| texture.reload_with_content(data));
            self.msdf_texture.set(texture);
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
pub struct System {
    logger           : Logger,
    context          : Context,
    sprite_system    : SpriteSystem,
    font             : font::Handle,
    color            : Buffer<Vector4<f32>>,
    glyph_msdf_index : Buffer<f32>,
    msdf_texture     : Uniform<Texture>,
}

impl System {
    /// Constructor.
    pub fn new<S>(scene:&S, font:font::Handle) -> Self
    where for<'t> &'t S : Into<&'t Scene> {
        let logger        = Logger::new("glyph_system");
        let msdf_width    = msdf::Texture::WIDTH as f32;
        let msdf_height   = msdf::Texture::ONE_GLYPH_HEIGHT as f32;
        let scene         = scene.into();
        let context       = scene.context.clone_ref();
        let sprite_system = SpriteSystem::new(scene);
        let symbol        = sprite_system.symbol();
        let texture       = Texture::new(&context,(0,0));
        let mesh          = symbol.surface();

        sprite_system.set_material(Self::material());
        sprite_system.set_alignment(Alignment::bottom_left());
        scene.variables.add("msdf_range",GlyphRenderInfo::MSDF_PARAMS.range as f32);
        scene.variables.add("msdf_size",Vector2::new(msdf_width,msdf_height));
        Self {logger,context,sprite_system,font,
            msdf_texture     : symbol.variables().add_or_panic("msdf_texture",texture),
            color            : mesh.instance_scope().add_buffer("color"),
            glyph_msdf_index : mesh.instance_scope().add_buffer("glyph_msdf_index"),
        }
    }

    /// Create new glyph. In the returned glyph the further parameters (position, size, character)
    /// may be set.
    pub fn new_glyph(&self) -> Glyph {
        let context         = self.context.clone();
        let sprite          = self.sprite_system.new_instance();
        let instance_id     = sprite.instance_id;
        let color_attr      = self.color.at(instance_id);
        let msdf_index = self.glyph_msdf_index.at(instance_id);
        let font            = self.font.clone_ref();
        let msdf_texture    = self.msdf_texture.clone();
        color_attr.set(Vector4::new(0.0,0.0,0.0,0.0));
        msdf_index.set(0.0);
        Glyph {context,sprite,msdf_index,color_attr,font,msdf_texture}
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

const FUNCTIONS : &str = include_str!("../glsl/glyph.glsl");
const MAIN      : &str = "output_color = color_from_msdf(); output_id=vec4(0.0,0.0,0.0,0.0);";

impl System {
    /// Defines a default material of this system.
    fn material() -> Material {
        let mut material = Material::new();
        material.add_input_def::<texture::FloatSampler> ("msdf_texture");
        material.add_input_def::<Vector2<f32>>          ("msdf_size");
        material.add_input_def::<f32>                   ("glyph_msdf_index");
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


