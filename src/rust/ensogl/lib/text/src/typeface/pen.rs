//! The pen is a point on the text _baseline_ used to locate glyph. It moves along the _baseline_
//! with each glyph rendered. For details, see
//! [freetype documentation](https://www.freetype.org/freetype2/docs/glyphs/glyphs-3.html#section-1)

use crate::prelude::*;

use super::font::Font;



// ================
// === CharInfo ===
// ================

/// Information about the current char pointed by the pen.
#[derive(Clone,Copy,Debug)]
#[allow(missing_docs)]
pub struct CharInfo {
    pub char   : char,
    pub offset : f32,
}



// ================
// === Iterator ===
// ================

pub trait CharIterator = std::iter::Iterator<Item=char>;

/// Iterator over chars producing the pen position for a given char.
///
/// The pen is a font-specific term (see
/// [freetype documentation](https://www.freetype.org/freetype2/docs/glyphs/glyphs-3.html#section-1)
/// for details).
#[derive(Debug)]
pub struct Iterator<I> {
    offset       : f32,
    font_size    : f32,
    current_char : Option<char>,
    next_chars   : I,
    font         : Font,
}

impl<I:CharIterator> std::iter::Iterator for Iterator<I> {
    type Item = CharInfo;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_chars.next().map(|t| self.advance(t))
    }
}

impl<I:CharIterator> Iterator<I> {
    /// Create iterator wrapping `chars`, with pen starting from given position.
    pub fn new (font_size:f32, next_chars:I, font:Font) -> Self {
        let offset       = 0.0;
        let current_char = None;
        Self {offset,font_size,current_char,next_chars,font}
    }

    fn advance(&mut self, next_char:char) -> CharInfo {
        if let Some(current_char) = self.current_char {
            let kerning = self.font.get_kerning(current_char,next_char);
            let advance = self.font.get_glyph_info(current_char).advance + kerning;
            let offset  = advance * self.font_size;
            self.offset += offset;
        }
        self.current_char = Some(next_char);
        let offset        = self.offset;
        CharInfo {char:next_char,offset}
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    use crate::display::shape::text::glyph::font;
    use crate::display::shape::text::glyph::font::GlyphRenderInfo;

    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test(async)]
    async fn moving_pen(){
        ensogl_core_msdf_sys::initialized().await;
        let font = Font::new(font::RenderInfo::mock_font("Test font".to_string()));
        mock_a_glyph_info(font.clone_ref());
        mock_w_glyph_info(font.clone_ref());
        font.mock_kerning_info('A', 'W', -0.16);
        font.mock_kerning_info('W', 'A', 0.0);

        let chars    = "AWA".chars();
        let iter     = Iterator::new(1.0,chars,font);
        let result   = iter.collect_vec();
        let expected = vec!
            [ ('A', 0.0)
            , ('W', 0.4)
            , ('A', 1.1)
            ];
        assert_eq!(expected,result);
    }

    fn mock_a_glyph_info(font:Font) -> GlyphRenderInfo {
        let advance = 0.56;
        let scale   = Vector2::new(0.5, 0.8);
        let offset  = Vector2::new(0.1, 0.2);
        font.mock_char_info('A',scale,offset,advance)
    }

    fn mock_w_glyph_info(font:Font) -> GlyphRenderInfo {
        let advance = 0.7;
        let scale   = Vector2::new(0.6, 0.9);
        let offset  = Vector2::new(0.1, 0.2);
        font.mock_char_info('W',scale,offset,advance)
    }
}