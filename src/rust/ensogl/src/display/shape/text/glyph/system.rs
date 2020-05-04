//! A main glyph system implementation.

use crate::prelude::*;

use crate::display::layout::types::*;
use crate::display::shape::text::glyph::font::FontHandle;
use crate::display::shape::text::glyph::font::GlyphRenderInfo;
use crate::display::shape::text::glyph::pen::PenIterator;
use crate::display::shape::text::glyph::msdf::MsdfTexture;
use crate::display::symbol::material::Material;
use crate::display::symbol::shader::builder::CodeTemplate;
use crate::display::world::*;
use crate::display::scene::Scene;
use crate::system::gpu::texture::*;
use crate::system::gpu::types::*;
use crate::display::object::traits::*;

use nalgebra::Vector2;
use nalgebra::Vector4;
use crate::display;



// =============
// === Glyph ===
// =============

/// A glyph rendered on screen. The displayed character will be stretched to fit the entire bbox of
/// underlying sprite.
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
pub struct Glyph {
    #[shrinkwrap(main_field)]
    sprite          : Sprite,
    context         : Context,
    msdf_index_attr : Attribute<f32>,
    color_attr      : Attribute<Vector4<f32>>,
    font            : FontHandle,
    msdf_uniform    : Uniform<Texture<GpuOnly,Rgb,u8>>,
}

impl Glyph {
    /// Glyph color attribute accessor.
    pub fn color(&self) -> Attribute<Vector4<f32>> {
        self.color_attr.clone_ref()
    }

    /// Change the displayed character.
    pub fn set_glyph(&self, ch:char) {
        let glyph_info = self.font.get_glyph_info(ch);
        self.msdf_index_attr.set(glyph_info.msdf_texture_glyph_id as f32);
        self.update_msdf_texture();
    }

    fn update_msdf_texture(&self) {
        let texture_changed = self.msdf_uniform.with_content(|texture| {
            texture.storage().height != self.font.msdf_texture_rows() as i32
        });
        if texture_changed {
            let width   = MsdfTexture::WIDTH as i32;
            let height  = self.font.msdf_texture_rows() as i32;
            let texture = Texture::<GpuOnly,Rgb,u8>::new(&self.context,(width,height));
            self.font.with_borrowed_msdf_texture_data(|data| {
                texture.reload_with_content(data);
            });
            self.msdf_uniform.set(texture);
        }
    }
}

impl display::Object for Glyph {
    fn display_object(&self) -> &display::object::Instance {
        self.sprite.display_object()
    }
}



// ============
// === Line ===
// ============

/// A structure keeping line of glyphs with proper alignment.
///
/// Not all the glyphs in `glyphs` vector may be actually in use; this structure is meant to keep
/// changing text, and for best performance it re-uses the created Glyphs (what means the specific
/// buffer space). Therefore there is a cap for line length. See also `GlyphSystem::new_empty_line`.
#[derive(Debug)]
pub struct Line {
    display_object    : display::object::Instance,
    glyph_system      : GlyphSystem,
    content           : Rc<RefCell<String>>,
    pub glyphs        : Rc<RefCell<Vec<Glyph>>>,
    baseline_start    : Vector2<f32>,
    color             : Cell<Vector4<f32>>,
    height            : f32,
    font              : FontHandle,
    const_glyph_count : Cell<Option<usize>>,
}

impl Line {
    /// Replace currently visible text.
    pub fn set_text<S:Into<String>>(&self, content:S) {
        *self.content.borrow_mut() = content.into();
        self.redraw();
    }

    fn resize(&self) {
        let content_len        = self.content.borrow().len();
        let target_glyph_count = self.const_glyph_count.get().unwrap_or(content_len);
        let glyph_count        = self.glyphs.borrow().len();
        if target_glyph_count > glyph_count {
            let new_count  = target_glyph_count - glyph_count;
            let new_glyphs = (0..new_count).map(|_| {
                let glyph = self.glyph_system.new_instance();
                self.add_child(&glyph);
                glyph
            });
            self.glyphs.borrow_mut().extend(new_glyphs)
        }
        if glyph_count > target_glyph_count {
            self.glyphs.borrow_mut().truncate(target_glyph_count)
        }
    }

    fn redraw(&self) {
        self.resize();

        let content     = self.content.borrow();
        let font        = self.font.clone_ref();
        let chars       = content.chars();
        let pen         = PenIterator::new(self.baseline_start,self.height,chars,font);
        let content_len = content.len();

        for (glyph,(chr,position)) in self.glyphs.borrow().iter().zip(pen) {
            let glyph_info = self.font.get_glyph_info(chr);
            let size       = glyph_info.scale.scale(self.height);
            let offset     = glyph_info.offset.scale(self.height);
            let x = position.x + offset.x;
            let y = position.y + offset.y;
            glyph.set_position(Vector3::new(x,y,0.0));
            glyph.set_glyph(chr);
            glyph.color().set(self.color.get());
            glyph.size().set(size);
        }
        for glyph in self.glyphs.borrow().iter().skip(content_len) {
            glyph.size().set(Vector2::new(0.0,0.0));
        }
    }

    pub fn set_color(&self, color:Vector4<f32>) {
        self.color.set(color);
        for glyph in &*self.glyphs.borrow() {
            glyph.color().set(color);
        }
    }

    pub fn set_const_glyph_count_opt(&self, count:Option<usize>) {
        self.const_glyph_count.set(count);
        self.resize();
    }

    pub fn set_const_glyph_count(&self, count:usize) {
        self.set_const_glyph_count_opt(Some(count));
    }

    pub fn unset_const_glyph_count(&self) {
        self.set_const_glyph_count_opt(None);
    }

    /// Set the baseline start point for this line.
    pub fn set_baseline_start(&mut self, new_start:Vector2<f32>) {
        let offset = new_start - self.baseline_start;
        for glyph in &*self.glyphs.borrow() {
            glyph.mod_position(|pos| *pos += Vector3::new(offset.x,offset.y,0.0));
        }
        self.baseline_start = new_start;
    }
}


// === Getters ===

impl Line {
    /// The starting point of this line's baseline.
    pub fn baseline_start(&self) -> &Vector2<f32> {
        &self.baseline_start
    }

    /// Line's height in pixels.
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Number of glyphs, giving the maximum length of displayed line. // FIXME
    pub fn length(&self) -> usize {
        self.content.borrow().len()
    }

    /// Font used for rendering this line.
    pub fn font(&self) -> FontHandle {
        self.font.clone_ref()
    }
}

impl display::Object for Line {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}



// ===================
// === GlyphSystem ===
// ===================

/// A system for displaying glyphs.
#[derive(Clone,CloneRef,Debug)]
pub struct GlyphSystem {
    context          : Context,
    sprite_system    : SpriteSystem,
    font             : FontHandle,
    color            : Buffer<Vector4<f32>>,
    glyph_msdf_index : Buffer<f32>,
    msdf_uniform     : Uniform<Texture<GpuOnly,Rgb,u8>>,
}

impl GlyphSystem {
    /// Constructor.
    pub fn new<S>(scene:&S, font:FontHandle) -> Self
    where for<'t> &'t S : Into<&'t Scene> {
        let msdf_width    = MsdfTexture::WIDTH as f32;
        let msdf_height   = MsdfTexture::ONE_GLYPH_HEIGHT as f32;
        let scene         = scene.into();
        let context       = scene.context.clone_ref();
        let sprite_system = SpriteSystem::new(scene);
        let symbol        = sprite_system.symbol();
        let texture       = Texture::<GpuOnly,Rgb,u8>::new(&context,(0,0));
        let mesh          = symbol.surface();

        sprite_system.set_material(Self::material());
        sprite_system.set_alignment(HorizontalAlignment::Left,VerticalAlignment::Bottom);
        scene.variables.add("msdf_range",GlyphRenderInfo::MSDF_PARAMS.range as f32);
        scene.variables.add("msdf_size",Vector2::new(msdf_width,msdf_height));
        Self {context,sprite_system,font,
            msdf_uniform     : symbol.variables().add_or_panic("msdf_texture",texture),
            color            : mesh.instance_scope().add_buffer("color"),
            glyph_msdf_index : mesh.instance_scope().add_buffer("glyph_msdf_index"),
        }
    }

    /// Create new glyph. In the returned glyph the further parameters (position, bbox, character)
    /// may be set.
    pub fn new_instance(&self) -> Glyph {
        let context         = self.context.clone();
        let sprite          = self.sprite_system.new_instance();
        let instance_id     = sprite.instance_id;
        let color_attr      = self.color.at(instance_id);
        let msdf_index_attr = self.glyph_msdf_index.at(instance_id);
        let font            = self.font.clone_ref();
        let msdf_uniform    = self.msdf_uniform.clone();
        color_attr.set(Vector4::new(0.0,0.0,0.0,0.0));
        msdf_index_attr.set(0.0);

        Glyph {context,sprite,msdf_index_attr,color_attr,font,msdf_uniform}
    }

    /// Create an empty "line" structure with defined number of glyphs. In the returned `Line`
    /// structure you can set specific strings with no more than `length` characters.
    ///
    /// For details, see also `Line` structure documentation.
    pub fn new_empty_line
    (&self, baseline_start:Vector2<f32>, height:f32)
    -> Line {
        let logger = Logger::new("temp"); // FIXME
        let display_object = display::object::Instance::new(logger);
        let glyphs     = default();
        let font       = self.font.clone_ref();
        let color      = Cell::new(Vector4::new(0.0,0.0,0.0,1.0));
        let content    = default();
        let glyph_system = self.clone_ref();
        let const_glyph_count = default();
        Line {display_object,glyph_system,glyphs,baseline_start,height,color,font,content,const_glyph_count}
    }

    /// Create a line of glyphs with proper alignment.
    ///
    /// For details, see also `Line` structure documentation.
    pub fn new_line
    (&self, height:f32, text:&str)
    -> Line {
        let baseline_start = Vector2::new(0.0,0.0);
        let mut line = self.new_empty_line(baseline_start,height);
        line.set_text(text);
        line
    }

    /// Get underlying sprite system.
    pub fn sprite_system(&self) -> &SpriteSystem {
        &self.sprite_system
    }
}

impl display::Object for GlyphSystem {
    fn display_object(&self) -> &display::object::Instance {
        self.sprite_system.display_object()
    }
}


// === Private ===

impl GlyphSystem {
    /// Defines a default material of this system.
    fn material() -> Material {
        let mut material = Material::new();
        material.add_input_def::<FloatSampler>("msdf_texture");
        material.add_input_def::<Vector2<f32>>("msdf_size");
        material.add_input_def::<f32>         ("glyph_msdf_index");
        material.add_input("pixel_ratio", 1.0);
        material.add_input("zoom"       , 1.0);
        material.add_input("msdf_range" , GlyphRenderInfo::MSDF_PARAMS.range as f32);
        material.add_input("color"      , Vector4::new(0.0,0.0,0.0,1.0));
        // FIXME We need to use this output, as we need to declare the same amount of shader
        // FIXME outputs as the number of attachments to framebuffer. We should manage this more
        // FIXME intelligent. For example, we could allow defining output shader fragments,
        // FIXME which will be enabled only if pass of given attachment type was enabled.
        material.add_output("id", Vector4::<f32>::new(0.0,0.0,0.0,0.0));

        let code = CodeTemplate::new(BEFORE_MAIN.to_string(),MAIN.to_string(),"".to_string());
        material.set_code(code);
        material
    }
}

const BEFORE_MAIN : &str = include_str!("glyph.glsl");
const MAIN        : &str = "output_color = color_from_msdf(); output_id=vec4(0.0,0.0,0.0,0.0);";
