//! In this module we handle the fonts information required for rendering glyphs.

use crate::prelude::*;

use crate::display::shape::text::glyph::msdf::MsdfTexture;
use crate::display::shape::text::glyph::msdf::convert_msdf_translation;
use crate::display::shape::text::glyph::msdf::x_distance_from_msdf_value;

use basegl_core_msdf_sys as msdf_sys;
use basegl_core_embedded_fonts::EmbeddedFonts;
use msdf_sys::MsdfParameters;
use msdf_sys::MultichannelSignedDistanceField;
use nalgebra::Vector2;
use std::collections::hash_map::Entry::Occupied;
use std::collections::hash_map::Entry::Vacant;



// ========================
// === Font render info ===
// ========================

/// Data used for rendering a single glyph
///
/// Each distance and transformation values are expressed in normalized coordinates, where
/// (0.0, 0.0) is initial pen position for an character, and `y` = 1.0 is _ascender_.
///
/// `offset` and `scale` transforms the _base square_ for a character, such the glyph will be
/// rendered correctly with assigned MSDF texture. The _base square_ corners are (0.0, 0.0),
/// (1.0, 1.0).
///
/// For explanation of various font-rendering terms, see
/// [freetype documentation](https://www.freetype.org/freetype2/docs/glyphs/glyphs-3.html#section-1)
#[derive(Copy,Clone,Debug)]
pub struct GlyphRenderInfo {
    /// An index of glyph in a msdf texture (counted from the top of column. For details, see
    /// MsdfTexture documentation.
    pub msdf_texture_glyph_id: usize,
    /// A required offset of the _base square_ see structure documentation for details.
    pub offset: Vector2<f32>,
    /// A required scale of the _base square_ see structure documentation for details.
    pub scale: Vector2<f32>,
    /// An advance. Advance is the distance between two successive pen positions for specific glyph.
    pub advance: f32
}

/// A single font data used for rendering
///
/// The data for individual characters and kerning are load on demand.
///
/// Each distance and transformation values are expressed in normalized coordinates, where `y` = 0.0
/// is _baseline_ and `y` = 1.0 is _ascender_. For explanation of various font-rendering terms, see
/// [freetype documentation](https://www.freetype.org/freetype2/docs/glyphs/glyphs-3.html#section-1)
#[derive(Debug)]
pub struct FontRenderInfo {
    /// Name of the font.
    pub name      : String,
    msdf_sys_font : msdf_sys::Font,
    msdf_texture  : RefCell<MsdfTexture>,
    glyphs        : RefCell<HashMap<char,GlyphRenderInfo>>,
    kerning       : RefCell<HashMap<(char,char),f32>>
}

impl FontRenderInfo {
    /// See `MSDF_PARAMS` docs.
    pub const MAX_MSDF_SHRINK_FACTOR : f64 = 4.; // Note [Picked MSDF parameters]
    /// See `MSDF_PARAMS` docs.
    pub const MAX_MSDF_GLYPH_SCALE   : f64 = 2.; // Note [Picked MSDF parameters]

    /// Parameters used for MSDF generation.
    ///
    /// The range was picked such way, that we avoid fitting range in one rendered pixel.
    /// Otherwise the antialiasing won't work. I assumed some maximum `shrink factor` (how many
    /// times rendered square will be smaller than MSDF size), and pick an arbitrary maximum glyph
    /// scale up.
    ///
    /// The rest of parameters are the defaults taken from msdfgen library
    pub const MSDF_PARAMS : MsdfParameters = MsdfParameters {
        width                         : MsdfTexture::WIDTH,
        height                        : MsdfTexture::ONE_GLYPH_HEIGHT,
        edge_coloring_angle_threshold : 3.0,
        range                         : Self::MAX_MSDF_SHRINK_FACTOR * Self::MAX_MSDF_GLYPH_SCALE,
        max_scale                     : Self::MAX_MSDF_GLYPH_SCALE,
        edge_threshold                : 1.001,
        overlap_support               : true
    };

    /// Create render info based on font data in memory
    pub fn new(name:String, font_data:&[u8]) -> FontRenderInfo {
        FontRenderInfo {name,
            msdf_sys_font : msdf_sys::Font::load_from_memory(font_data),
            msdf_texture  : RefCell::new(default()),
            glyphs        : RefCell::new(default()),
            kerning       : RefCell::new(default())
        }
    }

    /// Create render info for one of embedded fonts
    pub fn from_embedded(base:&EmbeddedFonts, name:&str)
    -> Option<FontRenderInfo> {
        let font_data_opt = base.font_data_by_name.get(name);
        font_data_opt.map(|data| FontRenderInfo::new(name.to_string(),data))
    }

    /// Load char render info
    pub fn load_char(&self, ch:char) {
        let handle         = &self.msdf_sys_font;
        let unicode        = ch as u32;
        let params         = Self::MSDF_PARAMS;
        let mut glyphs     = self.glyphs.borrow_mut();

        let msdf           = MultichannelSignedDistanceField::generate(handle,unicode,&params);
        let inversed_scale = Vector2::new(1.0/msdf.scale.x, 1.0/msdf.scale.y);
        let translation    = convert_msdf_translation(&msdf);
        let glyph_info = GlyphRenderInfo {
            msdf_texture_glyph_id : glyphs.len(),
            offset                : nalgebra::convert(-translation),
            scale                 : nalgebra::convert(inversed_scale),
            advance               : x_distance_from_msdf_value(msdf.advance),
        };
        self.msdf_texture.borrow_mut().extend(msdf.data.iter());
        glyphs.insert(ch,glyph_info);
    }

    /// Get render info for one character, generating one if not found
    pub fn get_glyph_info(&self, ch:char) -> GlyphRenderInfo {
        if !self.glyphs.borrow().contains_key(&ch) {
            self.load_char(ch);
        }
        *self.glyphs.borrow().get(&ch).unwrap()
    }

    /// Get kerning between two characters
    pub fn get_kerning(&self, left : char, right : char) -> f32 {
        match self.kerning.borrow_mut().entry((left,right)) {
            Occupied(entry) => *entry.get(),
            Vacant(entry)   => {
                let msdf_val   = self.msdf_sys_font.retrieve_kerning(left, right);
                let normalized = x_distance_from_msdf_value(msdf_val);
                *entry.insert(normalized)
            }
        }
    }

    /// A whole msdf texture bound for this font.
    pub fn msdf_texture(&self) -> Ref<MsdfTexture> {
        self.msdf_texture.borrow()
    }

    #[cfg(test)]
    pub fn mock_font(name : String) -> FontRenderInfo {
        FontRenderInfo { name,
            msdf_sys_font : msdf_sys::Font::mock_font(),
            msdf_texture  : RefCell::new(default()),
            glyphs        : RefCell::new(default()),
            kerning       : RefCell::new(default()),
        }
    }

    #[cfg(test)]
    pub fn mock_char_info
    (&self, ch:char, offset:Vector2<f32>, scale:Vector2<f32>, advance:f32) -> GlyphRenderInfo {
        let data_size             = MsdfTexture::ONE_GLYPH_SIZE;
        let msdf_data             = (0..data_size).map(|_| 0.12345);
        let mut glyphs            = self.glyphs.borrow_mut();
        let msdf_texture_glyph_id = glyphs.len();

        let char_info = GlyphRenderInfo {offset,scale,advance,msdf_texture_glyph_id};
        self.msdf_texture.borrow_mut().extend(msdf_data);
        glyphs.insert(ch, char_info);
        *glyphs.get(&ch).unwrap()
    }

    #[cfg(test)]
    pub fn mock_kerning_info(&self, l : char, r : char, value : f32) {
        self.kerning.borrow_mut().insert((l,r),value);
    }
}



// ===================
// === LoadedFonts ===
// ===================

/// A handle for fonts loaded into memory.
pub type FontHandle = Rc<FontRenderInfo>;

/// Structure keeping all fonts loaded from different sources.
#[derive(Debug)]
pub struct FontRegistry {
    embedded : EmbeddedFonts,
    fonts    : HashMap<String,FontHandle>,
}

impl FontRegistry {
    /// Create empty `Fonts` structure (however it contains raw data of embedded fonts).
    pub fn new() -> FontRegistry {
        FontRegistry {
            embedded : EmbeddedFonts::create_and_fill(),
            fonts    : HashMap::new(),
        }
    }

    /// Get render font info from loaded fonts, and if it does not exists, load data from one of
    /// embedded fonts. Returns None if the name is missing in both loaded and embedded font list.
    pub fn get_or_load_embedded_font(&mut self, name:&str) -> Option<FontHandle> {
        match self.fonts.entry(name.to_string()) {
            Occupied(entry) => Some(entry.get().clone()),
            Vacant(entry)   => match FontRenderInfo::from_embedded(&self.embedded,name) {
                Some(render_info) => {
                    let rc = Rc::new(render_info);
                    entry.insert(rc.clone());
                    Some(rc)
                },
                None => None
            }
        }
    }

    /// Get render info of one of loaded fonts.
    pub fn get_render_info(&mut self, name:&str) -> Option<FontHandle> {
        self.fonts.get_mut(name).cloned()
    }
}

impl Default for FontRegistry {
    fn default() -> Self {
        Self::new()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::shape::text::glyph::msdf::MsdfTexture;

    use basegl_core_embedded_fonts::EmbeddedFonts;
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;

    const TEST_FONT_NAME : &str = "DejaVuSansMono-Bold";

    fn create_test_font_render_info() -> FontRenderInfo {
        let mut embedded_fonts = EmbeddedFonts::create_and_fill();
        FontRenderInfo::from_embedded(
            &mut embedded_fonts,
            TEST_FONT_NAME
        ).unwrap()
    }

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test(async)]
    async fn empty_font_render_info() {
        basegl_core_msdf_sys::initialized().await;
        let font_render_info = create_test_font_render_info();

        assert_eq!(TEST_FONT_NAME, font_render_info.name);
        assert_eq!(0, font_render_info.msdf_texture.borrow().data.len());
        assert_eq!(0, font_render_info.glyphs.borrow().len());
    }

    #[wasm_bindgen_test(async)]
    async fn loading_chars() {
        basegl_core_msdf_sys::initialized().await;
        let font_render_info = create_test_font_render_info();

        font_render_info.load_char('A');
        font_render_info.load_char('B');

        let chars      = 2;
        let tex_width  = MsdfTexture::WIDTH;
        let tex_height = MsdfTexture::ONE_GLYPH_HEIGHT * chars;
        let channels   = MultichannelSignedDistanceField::CHANNELS_COUNT;
        let tex_size   = tex_width * tex_height * channels;

        assert_eq!(tex_height , font_render_info.msdf_texture.borrow().rows());
        assert_eq!(tex_size   , font_render_info.msdf_texture.borrow().data.len());
        assert_eq!(chars      , font_render_info.glyphs.borrow().len());

        let first_char  = *font_render_info.glyphs.borrow().get(&'A').unwrap();
        let second_char = *font_render_info.glyphs.borrow().get(&'B').unwrap();

        let first_index  = 0;
        let second_index = 1;

        assert_eq!(first_index  , first_char.msdf_texture_glyph_id);
        assert_eq!(second_index , second_char.msdf_texture_glyph_id);
    }

    #[wasm_bindgen_test(async)]
    async fn getting_or_creating_char() {
        basegl_core_msdf_sys::initialized().await;
        let font_render_info = create_test_font_render_info();

        {
            let char_info = font_render_info.get_glyph_info('A');
            assert_eq!(0, char_info.msdf_texture_glyph_id);
        }
        assert_eq!(1, font_render_info.glyphs.borrow().len());

        {
            let char_info = font_render_info.get_glyph_info('A');
            assert_eq!(0, char_info.msdf_texture_glyph_id);
        }
        assert_eq!(1, font_render_info.glyphs.borrow().len());
    }
}
