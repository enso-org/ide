use crate::prelude::*;
use basegl_core_msdf_sys as msdf_sys;
use basegl_core_embedded_fonts::EmbeddedFonts;

// ====================
// === MSDF Texture ===
// ====================

/// Texture with msdf for all loaded glyphs of font
///
/// This structure keeps texture data in 8-bit-per-channel RGB format, which
/// is ready to be passed to webgl texImage2D. The texture contains MSDFs for
/// all loaded glyphs, organized in vertical column.
///
/// It implements Extend<f32> trait making possible to pass this structure
/// as an output argument for `basegl_core_msdf_sys::generate_msdf` function
pub struct MsdfTexture {
    pub data : Vec<u8>
}

impl MsdfTexture {
    pub const WIDTH : usize = 32;

    /// Number of rows in texture
    pub fn rows(&self) -> usize {
        self.data.len()/(msdf_sys::MultichannelSignedDistanceField::CHANNELS_COUNT*Self::WIDTH)
    }

    fn convert_cell_from_f32(value : f32) -> u8 {
        nalgebra::clamp(value*255.0, 0.0, 255.0) as u8
    }
}

impl Extend<f32> for MsdfTexture {
    /// Extends texture with new MSDF data in f32 format
    fn extend<T: IntoIterator<Item=f32>>(&mut self, iter: T) {
        self.data.extend(
            iter.into_iter().map(Self::convert_cell_from_f32)
        );
    }
}

// =================
// === Char info ===
// =================

/// A single character data used for rendering
pub struct CharRenderInfo {
    pub msdf_texture_rows     : std::ops::Range<usize>,
    pub points_transformation : nalgebra::Projective2<f32>,
    pub advance               : f32
}

/// A single font data used for rendering
///
/// The data for individual characters are load on demand
pub struct FontRenderInfo {
    pub name          : String,
    pub msdf_sys_font : msdf_sys::Font,
    pub msdf_texture  : MsdfTexture,
    chars             : HashMap<char, CharRenderInfo>,
    kerning           : HashMap<(char, char), f32>
}

impl FontRenderInfo {
    pub const MSDF_PARAMS : msdf_sys::MsdfParameters =
        msdf_sys::MsdfParameters {
        width                         : MsdfTexture::WIDTH,
        height                        : MsdfTexture::WIDTH,
        edge_coloring_angle_threshold : 3.0,
        range                         : 8.0,
        edge_threshold                : 1.001,
        overlap_support               : true
    };

    /// Create render info for font data in memory
    pub fn new(
        name      : String,
        font_data : &[u8],
    ) -> FontRenderInfo {
        FontRenderInfo {
            name,
            msdf_sys_font: msdf_sys::Font::load_from_memory(font_data),
            msdf_texture    : MsdfTexture { data : Vec::new() },
            chars           : HashMap::new(),
            kerning         : HashMap::new()
        }
    }

    /// Create render info for one of embedded fonts
    pub fn from_embedded(
        base : &EmbeddedFonts,
        name : &'static str
    ) -> FontRenderInfo {
        let font_data = base.font_data_by_name.get(name).unwrap();
        crate::text::font::FontRenderInfo::new(
            name.to_string(), font_data
        )
    }

    /// Load char render info
    pub fn load_char(&mut self, ch : char) {
        let msdf = msdf_sys::MultichannelSignedDistanceField::generate(
            &self.msdf_sys_font,
            ch as u32,
            &FontRenderInfo::MSDF_PARAMS,
        );

        let msdf_texture_rows_begin = self.msdf_texture.rows();
        self.msdf_texture.extend(msdf.data.iter());
        let msdf_texture_rows_end = self.msdf_texture.rows();

        let msdf_tranformation_matrix = nalgebra::Matrix3::new(
            msdf.scale[0] as f32, 0.0,                  msdf.translation[0] as f32/MsdfTexture::WIDTH as f32*msdf.scale[0] as f32,
            0.0,                  msdf.scale[1] as f32, msdf.translation[1] as f32/MsdfTexture::WIDTH as f32*msdf.scale[0] as f32,
            0.0,                  0.0,                  1.0
        );
        let msdf_transformation : nalgebra::Projective2<f32> = nalgebra::Projective2::from_matrix_unchecked(msdf_tranformation_matrix);

        let char_info = CharRenderInfo {
            msdf_texture_rows : msdf_texture_rows_begin..msdf_texture_rows_end,
            points_transformation: msdf_transformation.inverse(),
            advance: msdf.advance as f32 / MsdfTexture::WIDTH as f32
        };
        self.chars.insert(ch, char_info);
    }

    /// Get or create render info for one character
    pub fn get_or_create_char_info(&mut self, ch : char) -> &CharRenderInfo {
        if !self.chars.contains_key(&ch) {
            self.load_char(ch);
        }
        self.chars.get(&ch).unwrap()
    }

    pub fn get_or_retrieve_kerning(&mut self, left : char, right : char)
        -> f32 {
        if !self.kerning.contains_key(&(left, right)) {
            let kerning =
                self.msdf_sys_font.retrieve_kerning(left, right) as f32 / MsdfTexture::WIDTH as f32;
            self.kerning.insert((left, right), kerning);
            kerning
        } else {
            *self.kerning.get(&(left, right)).unwrap()
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::text::font::{MsdfTexture, FontRenderInfo};
    use basegl_core_msdf_sys as msdf_sys;
    use basegl_core_embedded_fonts::EmbeddedFonts;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use msdf_sys::test_utils::TestAfterInit;
    use std::future::Future;

    const TEST_FONT_NAME : &str = "DejaVuSansMono-Bold";

    fn create_test_font_render_info() -> FontRenderInfo {
        let mut embedded_fonts = EmbeddedFonts::create_and_fill();
        FontRenderInfo::from_embedded(
            &mut embedded_fonts,
            TEST_FONT_NAME
        )
    }

    #[test]
    fn extending_msdf_texture() {
        let mut texture = MsdfTexture {
            data : Vec::new()
        };
        let msdf_values: &[f32] = &[-0.5, 0.0, 0.25, 0.5, 0.75, 1.0, 1.25];
        texture.extend(msdf_values[..4].iter().cloned());
        texture.extend(msdf_values[4..].iter().cloned());

        assert_eq!([0, 0, 63, 127, 191, 255, 255], texture.data.as_slice());
    }

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test(async)]
    fn empty_font_render_info() -> impl Future<Output=()> {
        TestAfterInit::schedule(||{
            let font_render_info = create_test_font_render_info();

            assert_eq!(TEST_FONT_NAME, font_render_info.name);
            assert_eq!(0, font_render_info.msdf_texture.data.len());
            assert_eq!(0, font_render_info.chars.len());
        })
    }

    #[wasm_bindgen_test(async)]
    fn loading_chars() -> impl Future<Output=()> {
        TestAfterInit::schedule(|| {
            let mut font_render_info = create_test_font_render_info();

            font_render_info.load_char('A');
            font_render_info.load_char('B');

            let expected_texture_size = MsdfTexture::WIDTH * MsdfTexture::WIDTH
                * msdf_sys::MultichannelSignedDistanceField::CHANNELS_COUNT * 2;

            assert_eq!(MsdfTexture::WIDTH * 2,
                font_render_info.msdf_texture.rows());
            assert_eq!(expected_texture_size,
                font_render_info.msdf_texture.data.len());
            assert_eq!(2, font_render_info.chars.len());

            let first_char = font_render_info.chars.get(&'A').unwrap();
            let second_char = font_render_info.chars.get(&'B').unwrap();

            assert_eq!(0..MsdfTexture::WIDTH, first_char.msdf_texture_rows);
            assert_eq!(MsdfTexture::WIDTH..2 * MsdfTexture::WIDTH,
                second_char.msdf_texture_rows);
        })
    }

    #[wasm_bindgen_test(async)]
    fn getting_or_creating_char() -> impl Future<Output=()> {
        TestAfterInit::schedule(|| {
            let mut font_render_info = create_test_font_render_info();

            {
                let char_info = font_render_info.get_or_create_char_info('A');
                assert_eq!(0..MsdfTexture::WIDTH, char_info.msdf_texture_rows);
            }
            assert_eq!(1, font_render_info.chars.len());

            {
                let char_info = font_render_info.get_or_create_char_info('A');
                assert_eq!(0..MsdfTexture::WIDTH, char_info.msdf_texture_rows);
            }
            assert_eq!(1, font_render_info.chars.len());
        })
    }
}