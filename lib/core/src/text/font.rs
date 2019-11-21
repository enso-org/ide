use crate::prelude::*;
use basegl_core_msdf_sys as msdf_sys;
use basegl_core_fonts_base::FontsBase;

pub struct MsdfTexture {
    pub data : Vec<u8>
}

impl MsdfTexture {
    pub const WIDTH : usize = 32;

    pub fn rows(&self) -> usize {
        self.data.len()/(3*Self::WIDTH)
    }
}

impl Extend<f32> for MsdfTexture {
    fn extend<T: IntoIterator<Item=f32>>(&mut self, iter: T) {
        self.data.extend(
            iter.into_iter().map(|float| nalgebra::clamp(float*255.0, 0.0, 255.0) as u8)
        );
    }
}

pub struct FontRenderInfo {
    pub name         : String,
    pub msdf_sys_handle  : msdf_sys::Font,
    pub msdf_texture : MsdfTexture,
    chars            : HashMap<char, CharInfo>
}

pub struct CharInfo {
    pub msdf_texture_rows : std::ops::Range<usize>,
}

impl FontRenderInfo {
    pub const MSDF_PARAMS : msdf_sys::MSDFParameters = msdf_sys::MSDFParameters {
        width: MsdfTexture::WIDTH,
        height: MsdfTexture::WIDTH,
        edge_coloring_angle_threshold: 3.0,
        range: 8.0,
        edge_threshold: 1.001,
        overlap_support: true
    };

    pub fn new(
        name      : String,
        font_data : &[u8],
    ) -> FontRenderInfo {
        FontRenderInfo {
            name,
            msdf_sys_handle : msdf_sys::Font::load_from_memory(font_data),
            msdf_texture    : MsdfTexture { data : Vec::new() },
            chars           : HashMap::new()
        }
    }

    pub fn from_embedded(
        base : &FontsBase,
        name : String
    ) -> FontRenderInfo {
        let mfont = base.fonts_by_name.get(name.as_str()).unwrap();
        crate::text::font::FontRenderInfo::new(
            name, mfont
        )
    }

    pub fn get_char_info(&mut self, ch : char) -> &CharInfo {
        if !self.chars.contains_key(&ch) {
            self.chars.insert(ch, CharInfo::new(ch, &mut self.msdf_texture, &self.msdf_sys_handle));
        }
        self.chars.get(&ch).unwrap()
    }
}

impl CharInfo {
    fn new(ch : char, msdf_texture : &mut MsdfTexture, font_handle : &msdf_sys::Font) -> CharInfo {
        let msdf_texture_rows_begin = msdf_texture.rows();
        msdf_sys::generate_msdf(
            msdf_texture,
            &font_handle,
            ch as u32,
            &FontRenderInfo::MSDF_PARAMS,
            msdf_sys::Vector2D{ x : 1.0, y : 1.0 },
            msdf_sys::Vector2D{ x : 2.4, y : 2.4 }
        );
        let msdf_texture_rows_end = msdf_texture.rows();
        CharInfo {
            msdf_texture_rows : msdf_texture_rows_begin..msdf_texture_rows_end
        }
    }
}