use crate::prelude::*;

use crate::text::content::{CharPosition, RefreshInfo};

use basegl_backend_webgl::compile_shader;
use basegl_backend_webgl::Context;
use basegl_backend_webgl::link_program;
use basegl_backend_webgl::Program;
use basegl_backend_webgl::Shader;
use std::collections::HashSet;
use std::iter::once;
use std::ops::Range;
use web_sys::WebGlBuffer;

use crate::text::font::FontRenderInfo;
use crate::text::content::line::LineRef;
use crate::text::buffer::glyph_square::point_to_iterable;
use crate::text::buffer::set_buffer_data;

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
    pub static ref CURSOR_VERTICES_BASE_LAYOUT : [Point2<f64>;BASE_LAYOUT_SIZE] =
        [ Point2::new(0.9, 0.0)
        , Point2::new(0.9, 1.0)
        , Point2::new(1.1, 0.0)
        , Point2::new(1.1, 0.0)
        , Point2::new(0.9, 1.0)
        , Point2::new(1.1, 1.0)
        ];
}

pub struct Cursors {
    pub cursors       : Vec<Cursor>,
    pub dirty_cursors : HashSet<usize>,
    gl_program        : Program,
    buffer            : WebGlBuffer,
}

impl Cursors {

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
            let x_position = line.get_char_x_position(cursor.position.column, refresh.font);
            let y_position = line.start_point().y;
            CURSOR_VERTICES_BASE_LAYOUT.iter()
                .map(|p| Point2::new(p.x + x_position, p.y + y_position))
                .map(point_to_iterable)
                .flatten()
                .collect::<SmallVec<[f32;12]>>()
        }).flatten().collect_vec();

        set_buffer_data(gl_context,&self.buffer,data.as_slice());
    }
}
