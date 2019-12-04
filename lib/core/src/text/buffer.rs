pub mod glyph_square;
pub mod line;

use crate::prelude::*;

use crate::text::buffer::glyph_square::{GlyphVertexPositionBuilder, GlyphTextureCoordinatesBuilder, GlyphAttributeBuilder, BASE_LAYOUT_SIZE};
use crate::text::buffer::line::{LineAttributeBuilder, LineVerticesBuilder, LineTextureCoordinatesBuilder};
use crate::text::Line;
use crate::text::font::FontRenderInfo;

use basegl_backend_webgl::Context;
use js_sys::Float32Array;
use nalgebra::Transform2;
use web_sys::WebGlBuffer;

#[derive(Debug)]
pub struct TextComponentBuffers {
    pub vertex_position     : BufferData,
    pub texture_coordinates : BufferData,
    fragments               : Vec<BufferFragment>,
    pub displayed_lines     : usize,
    pub displayed_columns   : usize,
}

#[derive(Debug)]
pub struct BufferData {
    pub gl_handle     : WebGlBuffer,
    pub fragment_size : usize,
}

#[derive(Debug)]
struct BufferFragment {
//    vertex_position_offset     : usize,
//    texture_coordinates_offset : usize,
    assigned_line              : Option<usize>
}

impl TextComponentBuffers {
    pub fn new(gl_context:&Context, displayed_lines:usize, displayed_columns:usize)
    -> TextComponentBuffers {
        TextComponentBuffers {
            vertex_position     : BufferData {
                gl_handle       : gl_context.create_buffer().unwrap(),
                fragment_size   : GlyphVertexPositionBuilder::OUTPUT_SIZE * displayed_columns
            },
            texture_coordinates :  BufferData {
                gl_handle       : gl_context.create_buffer().unwrap(),
                fragment_size   : GlyphTextureCoordinatesBuilder::OUTPUT_SIZE * displayed_columns
            },
            fragments           : Self::build_fragments(displayed_lines,displayed_columns),
            displayed_lines,
            displayed_columns,
        }
    }

    fn build_fragments(displayed_lines:usize, displayed_columns:usize) -> Vec<BufferFragment> {
        let indexes   = 0..displayed_lines;
        let fragments = indexes.map(|_| BufferFragment { assigned_line : None });
        fragments.collect()
    }

    pub fn refresh_all<'a>(&mut self, gl_context:&Context, lines:&Vec<Line>, font:&'a mut FontRenderInfo, to_window:&Transform2<f64>) {
        self.assign_fragments(lines.len());

        let assigned_lines           = self.fragments.iter().map(|fragment| fragment.assigned_line);
        let content                  = assigned_lines.clone().map(|index| index.map_or("", |i| lines[i].content.as_str()));
        let line_indexes             = assigned_lines.map(|index| index.unwrap_or(0));
        let content_with_index       = content.zip(line_indexes);
        let mut vertex_position_data = Vec::new();
        let mut texture_coordinates_data = Vec::new();

        for (content, index) in content_with_index.clone() {
            let glyph_buider = GlyphVertexPositionBuilder::new(font,to_window.clone(),index);
            let builder      = LineVerticesBuilder::new(content,glyph_buider,self.displayed_columns);
            vertex_position_data.extend(builder.flatten().map(|f| f as f32));
        }

        self.set_buffer_data(gl_context, &self.vertex_position.gl_handle, vertex_position_data.as_ref());

        for (content, index) in content_with_index {
            let glyph_buider = GlyphTextureCoordinatesBuilder::new(font);
            let builder      = LineTextureCoordinatesBuilder::new(content,glyph_buider,self.displayed_columns);
            texture_coordinates_data.extend(builder.flatten().map(|f| f as f32));
        }

        self.set_buffer_data(gl_context, &self.texture_coordinates.gl_handle, texture_coordinates_data.as_ref());
    }

    fn vertex_position_builder_for_line<'a, 'b, 'c>(&self, line:&'a str, index:usize, font:&'b mut FontRenderInfo, to_window:&Transform2<f64>)
    -> LineVerticesBuilder<'a, 'b> {
        let glyph_buider = GlyphVertexPositionBuilder::new(font,to_window.clone(),index);
        LineVerticesBuilder::new(line,glyph_buider,self.displayed_columns)
    }

    fn assign_fragments(&mut self, lines_count:usize) {
        for (i, fragment) in self.fragments.iter_mut().enumerate() {
            fragment.assigned_line = (i < lines_count).and_option(Some(i))
        }
    }

    fn set_buffer_data(&self, gl_context:&Context, buffer:&WebGlBuffer, vertices:&[f32]) {
        let target     = Context::ARRAY_BUFFER;

        gl_context.bind_buffer(target,Some(&buffer));
        Self::set_bound_buffer_data(gl_context,target,vertices);
    }

    fn set_bound_buffer_data(gl_context:&Context, target:u32, data:&[f32]) {
        let usage      = Context::STATIC_DRAW;

        unsafe { // Note [unsafe buffer_data]
            let float_32_array = Float32Array::view(&data);
            gl_context.buffer_data_with_array_buffer_view(target,&float_32_array,usage);
        }
    }

    pub fn vertices_count(&self) -> usize {
        BASE_LAYOUT_SIZE * self.displayed_lines * self.displayed_columns
    }
}
