pub mod fragment;
pub mod glyph_square;
pub mod line;

use crate::prelude::*;

use crate::text::buffer::glyph_square::BASE_LAYOUT_SIZE;
use crate::text::buffer::glyph_square::GlyphAttributeBuilder;
use crate::text::buffer::glyph_square::GlyphVertexPositionBuilder;
use crate::text::buffer::glyph_square::GlyphTextureCoordsBuilder;
use crate::text::buffer::fragment::BufferFragments;
use crate::text::buffer::fragment::FragmentsDataBuilder;
use crate::text::font::FontRenderInfo;

use basegl_backend_webgl::Context;
use js_sys::Float32Array;
use nalgebra::Vector2;
use web_sys::WebGlBuffer;
use std::ops::RangeInclusive;


// =============================
// === TextComponentsBuffers ===
// =============================

/// A structure managing all WebGl buffers used by TextComponent
///
/// Each attribute buffer is split to equal-sized fragments, and each fragment may is assigned to
/// displayed line The fragment keeps the data for this line.
#[derive(Debug)]
pub struct TextComponentBuffers {
    pub vertex_position : WebGlBuffer,
    pub texture_coords  : WebGlBuffer,
    pub display_size    : Vector2<f64>,
    pub scroll_offset   : Vector2<f64>,
    pub fragments       : BufferFragments,
    max_displayed_chars : usize,
    scrolled_x          : bool,
    scrolled_y          : bool,
}

/// References to all needed stuff for generating buffer's data.
pub struct ContentRef<'a, 'b> {
    pub lines : &'a[String],
    pub font  : &'b mut FontRenderInfo,
}

impl TextComponentBuffers {
    /// Create and initialize buffers.
    pub fn new(gl_context:&Context, display_size:Vector2<f64>, content:ContentRef)
    -> TextComponentBuffers {
        let mut content_mut = content;
        let mut buffers     = Self::create_uninitialized(gl_context,display_size,&mut content_mut);
        buffers.setup_buffers(gl_context,content_mut);
        buffers
    }

    /// Change scrolling by given offset and marks appropriate dirties.
    ///
    /// The offset is in "text" space, where each line has height of 1.0
    pub fn scroll(&mut self, offset:Vector2<f64>) {
        self.scroll_offset += offset;
        self.scrolled_x    |= offset.x != 0.0;
        self.scrolled_y    |= offset.y != 0.0;
    }

    /// Refresh the whole buffers data.
    pub fn refresh(&mut self, gl_context:&Context, content:ContentRef) {
        if self.scrolled_y {
            let displayed_lines = self.displayed_lines(content.lines.len());
            self.fragments.reassign_fragments(displayed_lines);
        }
        if self.scrolled_x {
            let displayed_x   = self.displayed_x_range();
            self.fragments.mark_dirty_after_x_scrolling(displayed_x,content.lines);
        }
        if self.scrolled_x || self.scrolled_y {
            let opt_dirty_range = self.fragments.minimum_fragments_range_with_all_dirties();
            if let Some(dirty_range) = opt_dirty_range {
                self.refresh_fragments(gl_context,dirty_range,content); // Note[refreshing buffers]
            }
            self.scrolled_x = true;
            self.scrolled_y = true;
        }
    }

    /* Note[refreshing buffer]
     *
     * The data exchange with GPU have so big overhead, that we usually replace buffer data with
     * one operation. That's why we gathering range with all dirties possibly catching many
     * not-dirty fragments.
     */

    fn create_uninitialized(gl_context:&Context, display_size:Vector2<f64>, content:&mut ContentRef)
    -> TextComponentBuffers {
        // Display_size.y.floor() makes space for all lines that fit in space in their full height.
        // But we have 2 more lines: one clipped from top, and one from bottom.
        const ADDITIONAL_LINES : usize = 2;
        let displayed_lines            = (display_size.y.floor() as usize) + ADDITIONAL_LINES;
        let space_width                = content.font.get_glyph_info(' ').advance;
        let max_displayed_chars        = (display_size.x.ceil() / space_width) as usize;
        TextComponentBuffers {display_size,max_displayed_chars,
            vertex_position : gl_context.create_buffer().unwrap(),
            texture_coords  : gl_context.create_buffer().unwrap(),
            fragments       : BufferFragments::new(displayed_lines),
            scroll_offset   : Vector2::new(0.0, 0.0),
            scrolled_x      : false,
            scrolled_y      : false,
        }
    }

    fn displayed_x_range(&self) -> RangeInclusive<f64> {
        let begin = self.scroll_offset.x;
        let end   = begin + self.display_size.x;
        begin..=end
    }

    fn displayed_lines(&self, lines_count:usize) -> RangeInclusive<usize> {
        let top                      = self.scroll_offset.y;
        let bottom                   = self.scroll_offset.y - self.display_size.y;
        let top_line_clipped         = Self::line_at_y_position(top,lines_count);
        let bottom_line_clipped      = Self::line_at_y_position(bottom,lines_count);
        let first_line_index         = top_line_clipped.unwrap_or(0);
        let last_line_index          = bottom_line_clipped.unwrap_or(lines_count-1);
        first_line_index..=last_line_index
    }

    fn line_at_y_position(y:f64, lines_count:usize) -> Option<usize> {
        let index    = -y.ceil();
        let is_valid = index >= 0.0 && index < lines_count as f64;
        is_valid.and_option_from(|| Some(index as usize))
    }

    fn setup_buffers(&mut self, gl_context:&Context, content:ContentRef) {
        let displayed_lines      = self.displayed_lines(content.lines.len());
        let all_fragments        = 0..self.fragments.fragments.len();
        let mut builder          = self.create_fragments_data_builder(content.font);

        self.fragments.reassign_fragments(displayed_lines);
        self.fragments.build_buffer_data_for_fragments(all_fragments,&mut builder,content.lines);
        let vertex_position_data = builder.vertex_position_data.as_ref();
        let texture_coords_data  = builder.texture_coords_data.as_ref();
        self.set_buffer_data(gl_context,&self.vertex_position, vertex_position_data);
        self.set_buffer_data(gl_context,&self.texture_coords , texture_coords_data);
    }

    fn refresh_fragments
    (&mut self, gl_context:&Context, indexes:RangeInclusive<usize>, content:ContentRef) {
        let ofsset      = *indexes.start();
        let mut builder = self.create_fragments_data_builder(content.font);

        self.fragments.build_buffer_data_for_fragments(indexes,&mut builder,content.lines);
        self.set_vertex_position_buffer_subdata(gl_context,ofsset,&builder);
        self.set_texture_coords_buffer_subdata (gl_context,ofsset,&builder);
    }

    fn create_fragments_data_builder<'a>(&self, font:&'a mut FontRenderInfo)
    -> FragmentsDataBuilder<'a> {
        let line_clip_left  = self.scroll_offset.x;
        let line_clip_right = line_clip_left + self.display_size.x;
        FragmentsDataBuilder {
            vertex_position_data : Vec::new(),
            texture_coords_data  : Vec::new(),
            font               /*: font*/,
            line_clip            : line_clip_left..line_clip_right,
            max_displayed_chars  : self.max_displayed_chars
        }
    }

    const GL_FLOAT_SIZE : usize = 4;

    fn set_vertex_position_buffer_subdata
    (&self, gl_context:&Context, fragment_offset:usize, builder:&FragmentsDataBuilder) {
        let char_output_floats = GlyphVertexPositionBuilder::OUTPUT_SIZE;
        let line_output_floats = char_output_floats * self.max_displayed_chars;
        let fragment_size      = line_output_floats * Self::GL_FLOAT_SIZE;
        let offset             = fragment_size * fragment_offset;
        let data               = builder.vertex_position_data.as_ref();
        self.set_buffer_subdata(gl_context,&self.vertex_position,offset,data);
    }

    fn set_texture_coords_buffer_subdata
    (&self, gl_context:&Context, fragment_offset:usize, builder:&FragmentsDataBuilder) {
        let char_output_floats = GlyphTextureCoordsBuilder::OUTPUT_SIZE;
        let line_output_floats = char_output_floats * self.max_displayed_chars;
        let fragment_size      = line_output_floats * Self::GL_FLOAT_SIZE;
        let offset        = fragment_size * fragment_offset;
        let data          = builder.texture_coords_data.as_ref();
        self.set_buffer_subdata(gl_context,&self.texture_coords,offset,data);
    }

    fn set_buffer_data(&self, gl_context:&Context, buffer:&WebGlBuffer, data:&[f32]) {
        let target = Context::ARRAY_BUFFER;
        gl_context.bind_buffer(target,Some(&buffer));
        Self::set_bound_buffer_data(gl_context,target,data);
    }

    fn set_bound_buffer_data(gl_context:&Context, target:u32, data:&[f32]) {
        let usage      = Context::STATIC_DRAW;
        unsafe { // Note [unsafe buffer_data]
            let float_array = Float32Array::view(&data);
            gl_context.buffer_data_with_array_buffer_view(target,&float_array,usage);
        }
    }

    fn set_buffer_subdata
    (&self, gl_context:&Context, buffer:&WebGlBuffer, offset:usize, data:&[f32]) {
        let target = Context::ARRAY_BUFFER;
        gl_context.bind_buffer(target,Some(&buffer));
        Self::set_bound_buffer_subdata(gl_context,target,offset as i32,data);
    }

    fn set_bound_buffer_subdata(gl_context:&Context, target:u32, offset:i32, data:&[f32]) {
        unsafe { // Note [unsafe buffer_data]
            let float_array = Float32Array::view(&data);
            gl_context.buffer_sub_data_with_i32_and_array_buffer_view(target,offset,&float_array);
        }
    }

    /* Note [unsafe buffer_data]
     *
     * The Float32Array::view is safe as long there are no allocations done
     * until it is destroyed. This way of creating buffers were taken from
     * wasm-bindgen examples
     * (https://rustwasm.github.io/wasm-bindgen/examples/webgl.html)
     */

    pub fn vertices_count(&self) -> usize {
        BASE_LAYOUT_SIZE * self.fragments.fragments.len() * self.max_displayed_chars
    }
}
