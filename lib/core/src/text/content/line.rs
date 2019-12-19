use crate::prelude::*;

use crate::text::font::FontRenderInfo;
use crate::text::buffer::glyph_square::Pen;

use nalgebra::Point2;
use std::ops::Range;


/// The rendered char position in line and on screen.
#[derive(Debug)]
#[derive(Clone)]
pub struct Line {
    chars            : Vec<char>,
    char_x_positions : Vec<f32>,
}

impl Line {

    pub fn new<S:Str>(string:S) -> Self {
        Self::new_raw(string.as_ref().chars().collect())
    }

    pub fn new_raw(chars:Vec<char>) -> Self {
        Line {chars,
            char_x_positions : Vec::new(),
        }
    }

    pub fn empty() -> Self {
        Line{
            chars            : Vec::new(),
            char_x_positions : Vec::new(),
        }
    }

    pub fn chars(&self) -> &[char] {
        &self.chars.as_ref()
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }

    pub fn modify(&mut self) -> &mut Vec<char> {
        self.char_x_positions.clear();
        &mut self.chars
    }

    pub fn get_char_x_position(&mut self, index:usize, font:&mut FontRenderInfo) -> f32 {
        self.fill_chars_x_position_up_to(index,font);
        self.char_x_positions[index]
    }

    pub fn get_char_x_range(&mut self, index:usize, font:&mut FontRenderInfo) -> Range<f32> {
        let start   = self.get_char_x_position(index,font);
        let advance = font.get_glyph_info(self.chars[index]).advance as f32;
        start..(start + advance)
    }

    pub fn find_char_at_x_position(&mut self, x_position:f32, font:&mut FontRenderInfo)
        -> Option<usize> {
        if self.chars.is_empty() {
            None
        } else {
            self.fill_chars_x_position_up_to_value(x_position,font);
            let last_index   = self.len() - 1;
            let found        = self.char_x_positions.binary_search_by(|f| f.partial_cmp(&x_position).unwrap());
            let mut in_range = || self.get_char_x_range(last_index, font).end >= x_position;
            match found {
                Ok(index)                        => Some(index),
                Err(0)                           => None,
                Err(index) if index > last_index => in_range().and_option_from(|| Some(last_index)),
                Err(index)                       => Some(index-1)
            }
        }
    }

    pub fn fill_chars_x_position_up_to(&mut self, index:usize, font:&mut FontRenderInfo) {
        let new_len = index + 1;
        let to_fill = new_len.saturating_sub(self.char_x_positions.len());
        let pen_opt = self.last_cached_char_pen(font);
        let mut pen = pen_opt.unwrap_or(Pen::new(Point2::new(0.0,0.0)));
        let chars   = &self.chars[self.char_x_positions.len()..];
        for ch in chars.iter().take(to_fill) {
            pen.next_char(*ch,font);
            let x_position = pen.position.x as f32;
            self.char_x_positions.push(x_position);
        }
    }

    pub fn fill_chars_x_position_up_to_value(&mut self, x_position:f32, font:&mut FontRenderInfo) {
        let last_cached    = self.char_x_positions.last();
        let already_filled = last_cached.map_or(false, |cached| *cached >= x_position);
        if !already_filled {
            for index in self.char_x_positions.len()..self.chars.len() {
                self.fill_chars_x_position_up_to(index,font);
                let current = self.char_x_positions[index];
                if current >= x_position {
                    break;
                }
            }
        }
    }

    fn last_cached_char_pen(&mut self, font:&mut FontRenderInfo) -> Option<Pen> {
        let y_position = 0.0;
        let x_position = self.char_x_positions.last().map(|f| *f as f64);
        let index      = self.char_x_positions.len().checked_sub(1);
        let char_opt   = index.map(|i| self.chars[i]);
        match (x_position,char_opt) {
            (Some(x),Some(ch)) => Some(Pen::new_with_char(Point2::new(x,y_position),ch,font)),
            _                  => None,
        }
    }
}

pub struct LineRef<'a> {
    pub line    : &'a mut Line,
    pub line_id : usize,
}

impl<'a> LineRef<'a> {
    pub fn start_point(&self) -> Point2<f64> {
        Point2::new(0.0, -(self.line_id as f64) - 1.0)
    }
}


#[cfg(test)]
mod test {
    use super::*;

    use basegl_core_msdf_sys::test_utils::TestAfterInit;
    use std::future::Future;
    use wasm_bindgen_test::wasm_bindgen_test;


    #[wasm_bindgen_test(async)]
    fn getting_chars_x_position() -> impl Future<Output=()> {
        TestAfterInit::schedule(|| {
            let mut font = prepare_font_with_ab();
            let mut line = Line::new("ABA");
            assert_eq!(0, line.char_x_positions.len());
            let first_pos = line.get_char_x_position(0,&mut font);
            assert_eq!(1, line.char_x_positions.len());
            let third_pos = line.get_char_x_position(2,&mut font);
            assert_eq!(3, line.char_x_positions.len());

            assert_eq!(0.0, first_pos);
            assert_eq!(2.5, third_pos);
        })
    }

    #[wasm_bindgen_test(async)]
    fn finding_char_by_x_position() -> impl Future<Output=()> {
        TestAfterInit::schedule(|| {
            let mut font           = prepare_font_with_ab();
            let mut line           = Line::new("ABBA");
            let before_first       = line.find_char_at_x_position(-0.1, &mut font);
            assert_eq!(1, line.char_x_positions.len());
            let first              = line.find_char_at_x_position(0.5, &mut font);
            assert_eq!(2, line.char_x_positions.len());
            let first_again        = line.find_char_at_x_position(0.5, &mut font);
            assert_eq!(2, line.char_x_positions.len());
            let third              = line.find_char_at_x_position(3.0, &mut font);
            assert_eq!(4, line.char_x_positions.len());
            let last               = line.find_char_at_x_position(4.5, &mut font);
            assert_eq!(4, line.char_x_positions.len());
            let after_last         = line.find_char_at_x_position(5.5, &mut font);
            let third_again        = line.find_char_at_x_position(3.0, &mut font);
            let before_first_again = line.find_char_at_x_position(-0.5, &mut font);

            assert_eq!(None, before_first);
            assert_eq!(Some(0), first);
            assert_eq!(Some(0), first_again);
            assert_eq!(Some(2), third);
            assert_eq!(Some(3), last);
            assert_eq!(None, after_last);
            assert_eq!(Some(2), third_again);
            assert_eq!(None, before_first_again);
        })
    }

    #[wasm_bindgen_test(async)]
    fn finding_char_by_x_position_in_empty_line() -> impl Future<Output=()> {
        TestAfterInit::schedule(|| {
            let mut font = prepare_font_with_ab();
            let mut line = Line::new("");
            let below_0  = line.find_char_at_x_position(-0.1,&mut font);
            let above_0  = line.find_char_at_x_position( 0.1,&mut font);
            assert_eq!(None,below_0);
            assert_eq!(None,above_0);
        })
    }

    #[wasm_bindgen_test(async)]
    fn modifying_line() -> impl Future<Output=()> {
        TestAfterInit::schedule(|| {
            let mut font = prepare_font_with_ab();
            let mut line = Line::new("AB");
            let before_edit = line.get_char_x_position(1,&mut font);
            assert_eq!(2, line.char_x_positions.len());
            line.modify().insert(0, 'B');
            assert_eq!(0, line.char_x_positions.len());
            let after_edit = line.get_char_x_position(1,&mut font);

            assert_eq!(1.0, before_edit);
            assert_eq!(1.5, after_edit);
        })
    }

    fn prepare_font_with_ab() -> FontRenderInfo {
        let mut font          = FontRenderInfo::mock_font("Test font".to_string());
        let mut a_info        = font.mock_char_info('A');
        a_info.advance        = 1.0;
        let mut b_info        = font.mock_char_info('B');
        b_info.advance        = 1.5;
        font.mock_kerning_info('A', 'B', 0.0);
        font.mock_kerning_info('B', 'A', 0.0);
        font.mock_kerning_info('A', 'A', 0.0);
        font.mock_kerning_info('B', 'B', 0.0);
        font
    }
}