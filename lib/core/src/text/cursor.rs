use crate::text::content::CharPosition;

use basegl_backend_webgl::Context;
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

pub struct Cursors {
    pub cursors       : Vec<Cursor>,
    pub dirty_cursors : HashSet<usize>,
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

    pub fn set_buffer_data(&mut self, gl_context:Context) {

    }
}
