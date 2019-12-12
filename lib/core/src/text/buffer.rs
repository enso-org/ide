pub mod fragment;
pub mod glyph_square;
pub mod line;

use crate::prelude::*;

use crate::text::buffer::glyph_square::
{BASE_LAYOUT_SIZE,GlyphAttributeBuilder,GlyphVertexPositionBuilder, GlyphTextureCoordsBuilder};
use crate::text::buffer::fragment::{BufferFragment,FragmentsDataBuilder};
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
    fragments           : Vec<BufferFragment>,
    assigned_lines      : RangeInclusive<usize>,
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
            self.reassign_after_y_scrolling(content.lines);
        }
        if self.scrolled_x {
            self.mark_dirty_after_x_scrolling(content.lines);
        }
        if self.scrolled_x || self.scrolled_y {
            self.refresh_dirty_fragments(gl_context,content);
        }
    }

    // === Initialize

    fn create_uninitialized(gl_context:&Context, display_size:Vector2<f64>, content:&mut ContentRef)
    -> TextComponentBuffers {
        let displayed_lines     = (display_size.y.ceil() as usize) + 2;
        let space_width         = content.font.get_glyph_info(' ').advance;
        let max_displayed_chars = (display_size.x.ceil() / space_width) as usize;
        TextComponentBuffers {display_size,max_displayed_chars,
            vertex_position : gl_context.create_buffer().unwrap(),
            texture_coords  : gl_context.create_buffer().unwrap(),
            fragments       : Self::create_fragments(displayed_lines),
            assigned_lines  : 1..=0,
            scroll_offset   : Vector2::new(0.0, 0.0),
            scrolled_x      : false,
            scrolled_y      : false,
        }
    }

    fn create_fragments(displayed_lines:usize) -> Vec<BufferFragment> {
        let indexes   = 0..displayed_lines;
        let fragments = indexes.map(|_| BufferFragment::unassigned());
        fragments.collect()
    }

    // === Reassigning after y scrolling

    fn reassign_after_y_scrolling(&mut self, lines:&[String]) {
        let displayed_lines    = self.displayed_lines(lines.len());
        let current_assignment = &self.assigned_lines;
        let new_on_top         = *displayed_lines.start()  .. *current_assignment.start();
        let new_on_bottom      = current_assignment.end()+1..=*displayed_lines.end();
        let mut new_lines      = new_on_top.chain(new_on_bottom);

        for fragment in &mut self.fragments {
            if fragment.can_be_reassigned(&displayed_lines) {
                fragment.assigned_line = new_lines.next();
                fragment.dirty         = true;
            }
        }
        self.assigned_lines = displayed_lines;
    }

    // === Marking dirty after x scrolling

    fn mark_dirty_after_x_scrolling(&mut self, lines:&[String]) {
        let displayed_x   = self.displayed_x_range();
        let not_yet_dirty = self.fragments.iter_mut().filter(|f| !f.dirty);
        for fragment in not_yet_dirty {
            fragment.dirty = fragment.should_be_dirty(&displayed_x,lines);
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

    fn reassign_fragments(&mut self, lines_count:usize) {
        let displayed_lines        = self.displayed_lines(lines_count);

        for (i, fragment) in self.fragments.iter_mut().enumerate() {
            let assigned_index     = displayed_lines.start() + i;
            let is_line_to_assign  = assigned_index <= *displayed_lines.end();
            let new_assignment     = is_line_to_assign.and_option(Some(assigned_index));
            fragment.dirty         = fragment.dirty || new_assignment != fragment.assigned_line;
            fragment.assigned_line = new_assignment;
        }
        self.assigned_lines = displayed_lines;
    }

    // === Creating and refreshing buffer data

    fn setup_buffers(&mut self, gl_context:&Context, content:ContentRef) {
        self.reassign_fragments(content.lines.len());
        let all_fragments        = 0..self.fragments.len();
        let builder              = self.build_buffer_data_for_fragments(all_fragments,content);
        let vertex_position_data = builder.vertex_position_data.as_ref();
        let texture_coords_data  = builder.texture_coords_data.as_ref();
        self.set_buffer_data(gl_context,&self.vertex_position, vertex_position_data);
        self.set_buffer_data(gl_context,&self.texture_coords , texture_coords_data);
    }

    fn refresh_dirty_fragments(&mut self, gl_context:&Context, content:ContentRef) {
        let fragments     = self.fragments.iter().enumerate();
        let dirty_indices = fragments.filter_map(|(i,f)| f.dirty.and_option_from(|| Some(i)));
        let first_dirty   = dirty_indices.clone().min();
        let last_dirty    = dirty_indices.clone().max();
        if let (Some(first),Some(last)) = (first_dirty,last_dirty) {
             self.refresh_fragments(gl_context,first..=last, content);
        }
        self.scrolled_x = false;
        self.scrolled_y = false;
    }

    fn refresh_fragments
    (&mut self, gl_context:&Context, indexes:RangeInclusive<usize>, content:ContentRef) {
        let ofsset  = *indexes.start();
        let builder = self.build_buffer_data_for_fragments(indexes,content);
        self.set_vertex_position_buffer_subdata(gl_context,ofsset,&builder);
        self.set_texture_coords_buffer_subdata (gl_context,ofsset,&builder);
    }

    fn build_buffer_data_for_fragments<'a, Indexes>
    (&mut self, fragments:Indexes, content:ContentRef<'_, 'a>)
    -> FragmentsDataBuilder<'a>
    where Indexes : Iterator<Item=usize> {
        let font  = content.font;
        let lines = content.lines;

        let mut builder = self.create_fragments_data_builder(font);
        for fragment_id in fragments {
            let fragment      = &mut self.fragments[fragment_id];
            let index         = fragment.assigned_line.unwrap_or(0);
            let line          = fragment.assigned_line.map_or("", |i| lines[i].as_str());
            fragment.rendered = builder.build_for_line(index, line);
            fragment.dirty    = false;
        }
        builder
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

    fn set_vertex_position_buffer_subdata
    (&self, gl_context:&Context, fragment_offset:usize, builder:&FragmentsDataBuilder) {
        let fragment_size = GlyphVertexPositionBuilder::OUTPUT_SIZE * self.max_displayed_chars * 4;
        let offset        = fragment_size * fragment_offset;
        let data          = builder.vertex_position_data.as_ref();
        self.set_buffer_subdata(gl_context,&self.vertex_position,offset,data);
    }

    fn set_texture_coords_buffer_subdata
    (&self, gl_context:&Context, fragment_offset:usize, builder:&FragmentsDataBuilder) {
        let fragment_size = GlyphTextureCoordsBuilder::OUTPUT_SIZE * self.max_displayed_chars * 4;
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
            let float_32_array = Float32Array::view(&data);
            gl_context.buffer_data_with_array_buffer_view(target,&float_32_array,usage);
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
            let float_32_array = Float32Array::view(&data);
            gl_context.buffer_sub_data_with_i32_and_array_buffer_view(target,offset,&float_32_array);
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
        BASE_LAYOUT_SIZE * self.fragments.len() * self.max_displayed_chars
    }
}
