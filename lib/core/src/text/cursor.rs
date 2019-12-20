use crate::prelude::*;

use crate::text::font::FontRenderInfo;
use crate::text::content::{CharPosition, TextComponentContent};
use crate::text::content::RefreshInfo;
use crate::text::content::line::LineRef;
use crate::text::buffer::glyph_square::point_to_iterable;
use crate::text::buffer::set_buffer_data;
use crate::text::TextComponentBuilder;

use basegl_backend_webgl::compile_shader;
use basegl_backend_webgl::Context;
use basegl_backend_webgl::link_program;
use basegl_backend_webgl::Program;
use basegl_backend_webgl::Shader;
use nalgebra::Point2;
use std::collections::HashSet;
use std::iter::once;
use std::ops::Range;
use web_sys::WebGlBuffer;


/// Cursor in TextComponent
#[derive(Debug)]
pub struct Cursor {
    pub position    : CharPosition,
    pub selected_to : CharPosition,
}

pub enum SelectionType {None,OnLeft,OnRight}


impl Cursor {
    pub fn new(position:CharPosition) -> Self {
        Cursor {
            position    : position.clone(),
            selected_to : position
        }
    }

    pub fn selection_type(&self) -> SelectionType {
        if self.position == self.selected_to {
            SelectionType::None
        } else if self.position > self.selected_to {
            SelectionType::OnLeft
        } else {
            SelectionType::OnRight
        }
    }

    pub fn selection_range(&self) -> Range<CharPosition> {
        match self.selection_type() {
            SelectionType::None    => self.position..self.position,
            SelectionType::OnLeft  => self.selected_to..self.position,
            SelectionType::OnRight => self.position..self.selected_to
        }
    }

    pub fn is_char_selected(&self, position:CharPosition) -> bool {
        self.selection_range().contains(&position)
    }
}

lazy_static! {
//    pub static ref CURSOR_VERTICES_BASE_LAYOUT : [Point2<f32>;6] =
//        [ Point2::new(-0.05, -0.2)
//        , Point2::new(-0.05,  0.8)
//        , Point2::new( 0.05, -0.2)
//        , Point2::new( 0.05, -0.2)
//        , Point2::new(-0.-5,  0.8)
//        , Point2::new( 0.05,  0.8)
//        ];
    pub static ref CURSOR_VERTICES_BASE_LAYOUT : [Point2<f32>;2] =
        [ Point2::new(0.0, -0.2)
        , Point2::new(0.0,  0.8)
        ];
}

#[derive(Debug)]
pub struct Cursors {
    pub cursors       : Vec<Cursor>,
    pub dirty_cursors : HashSet<usize>,
    pub buffer        : WebGlBuffer,
}

impl Cursors {

    pub fn new(gl_context:&Context) -> Self {
        Cursors {
            cursors       : Vec::new(),
            dirty_cursors : HashSet::new(),
            buffer        : gl_context.create_buffer().unwrap()
        }
    }

    pub fn set_cursor(&mut self, position:CharPosition) {
        self.cursors       = vec![Cursor::new(position)];
        self.dirty_cursors = once(0).collect();
    }

    pub fn add_cursor(&mut self, position:CharPosition) {
        let new_index = self.cursors.len();
        self.cursors.push(Cursor::new(position));
        self.dirty_cursors.insert(new_index);
    }


    pub fn set_buffer_data(&mut self, gl_context:&Context, refresh:&mut RefreshInfo) {
        let data = self.cursors.iter().map(|cursor| {
            let line       = LineRef{line:&mut refresh.lines[cursor.position.line], line_id:cursor.position.line};
            let x_position = if cursor.position.column > 0 {
                line.line.get_char_x_range(cursor.position.column-1, refresh.font).end
            } else {
                0.0
            };
            let y_position = line.start_point().y as f32;
            CURSOR_VERTICES_BASE_LAYOUT.iter()
                .map(|p| Point2::new(p.x + x_position, p.y + y_position))
                .map(point_to_iterable)
                .flatten()
                .collect::<SmallVec<[f32;12]>>()
        }).flatten().collect_vec();

        set_buffer_data(gl_context,&self.buffer,data.as_slice());
    }
}
