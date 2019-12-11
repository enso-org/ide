pub mod fragment;
pub mod glyph_square;
pub mod line;

use crate::prelude::*;

use crate::text::buffer::glyph_square::{BASE_LAYOUT_SIZE, GlyphAttributeBuilder, GlyphVertexPositionBuilder, GlyphTextureCoordsBuilder};
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
    pub scroll_offset   : Vector2<f64>,
    pub display_size    : Vector2<f64>,
    fragments           : Vec<BufferFragment>,
    max_displayed_chars : usize,
    scrolled            : bool,
}

pub struct ContentRef<'a, 'b> {
    pub lines : &'a[String],
    pub font  : &'b mut FontRenderInfo,
}

impl TextComponentBuffers {
    /// Create and initialize buffers.
    pub fn new(gl_context:&Context, display_size:Vector2<f64>, content:ContentRef)
    -> TextComponentBuffers {
        let displayed_lines     = display_size.y.ceil() as usize;
        let space_width         = content.font.get_glyph_info(' ').advance;
        let max_displayed_chars = (display_size.x.ceil() / space_width) as usize;
        let mut buffers         = TextComponentBuffers {
            vertex_position     : gl_context.create_buffer().unwrap(),
            texture_coords      : gl_context.create_buffer().unwrap(),
            scroll_offset: Vector2::new(0.0, 0.0),
            display_size        ,
            fragments           : Self::build_fragments(displayed_lines),
            max_displayed_chars ,
            scrolled            : false
        };
        buffers.setup_buffers(gl_context,content);
        buffers
    }

    fn build_fragments(displayed_lines:usize) -> Vec<BufferFragment> {
        let indexes   = 0..displayed_lines;
        let fragments = indexes.map(|_| BufferFragment::unassigned());
        fragments.collect()
    }

    fn setup_buffers(&mut self, gl_context:&Context, content:ContentRef) {
        self.reassign_fragments(content.lines.len());
        let all_fragments        = 0..self.fragments.len();
        let builder              = self.build_buffer_data_for_fragments(all_fragments,content);
        let vertex_position_data = builder.vertex_position_data.as_ref();
        let texture_coords_data  = builder.texture_coords_data.as_ref();
        self.set_buffer_data(gl_context,&self.vertex_position, vertex_position_data);
        self.set_buffer_data(gl_context,&self.texture_coords , texture_coords_data);
    }

    /// Refresh the whole buffers making them display the given lines.
    pub fn refresh_all<'a>
    (&mut self, gl_context:&Context, content:ContentRef) {
        let fragments_to_refresh = self.fragments_to_refresh_after_scrolling(&content.lines);
        let first_fragment       = fragments_to_refresh.iter().min();
        let last_fragment        = fragments_to_refresh.iter().max();
        match (first_fragment, last_fragment) {
            (Some(first),Some(last)) => {
                let builder = self.build_buffer_data_for_fragments(*first..=*last, content);
                self.set_vertex_position_buffer_subdata(gl_context,*first,&builder);
                self.set_texture_coords_buffer_subdata (gl_context,*first,&builder);
            }
            _ => {}
        }
    }

    fn fragments_to_refresh_after_scrolling(&mut self, lines:&[String]) -> HashSet<usize> {
        let mut unassigned_fragments = HashSet::new();
        let mut assigned_lines       = HashSet::new();
        let mut buffers_to_refresh   = HashSet::new();

        let lines_rendered           = self.displayed_lines(lines.len());

        for (fragment_id, fragment) in self.fragments.iter_mut().enumerate() {
            match fragment.assigned_line {
                Some(line_index) => {
                    if lines_rendered.contains(&line_index) {
                        assigned_lines.insert(line_index);
                        match &fragment.rendered_fragment {
                            Some(rendered) => {
                                let line_front_rendered = rendered.first_char.byte_offset == 0;
                                let line_end_rendered   = rendered.last_char.byte_offset == lines[line_index].len()-1;
                                let line_begin = self.scroll_offset.x;
                                let line_end   = line_begin + self.display_size.x;
                                let something_from_left = !line_front_rendered && line_begin < rendered.first_char.pen.position.x;
                                let something_from_right = !line_end_rendered && line_end > rendered.last_char.pen.position.x + rendered.last_char.pen.next_advance;
                                if something_from_left || something_from_right {
                                    buffers_to_refresh.insert(fragment_id);
                                }
                            }
                            None => { buffers_to_refresh.insert(fragment_id); }
                        }
                    } else {
                        fragment.assigned_line = None;
                        buffers_to_refresh.insert(fragment_id);
                        unassigned_fragments.insert(fragment_id);
                    }
                },
                None            => { unassigned_fragments.insert(fragment_id);}
            }
        }

        for line in lines_rendered {
            if !assigned_lines.contains(&line) {
                let frag_id = unassigned_fragments.take(&unassigned_fragments.iter().next().unwrap().clone()).unwrap();
                self.fragments[frag_id].assigned_line = Some(line);
                buffers_to_refresh.insert(frag_id);
            }
        }

        buffers_to_refresh
    }

    fn displayed_lines(&self, lines_count:usize) -> RangeInclusive<usize> {
        let top                      = self.scroll_offset.y;
        let bottom                   = self.scroll_offset.y + self.display_size.y;
        let top_line_clipped         = Self::line_at_y_position(top,lines_count);
        let bottom_line_clipped      = Self::line_at_y_position(bottom,lines_count);
        let first_line_index         = top_line_clipped.unwrap_or(0);
        let last_line_index          = bottom_line_clipped.unwrap_or(lines_count-1);
        first_line_index..=last_line_index
    }

    fn line_at_y_position(y:f64, lines_count:usize) -> Option<usize> {
        let index    = -y.floor();
        let is_valid = index >= 0.0 && index < lines_count as f64;
        is_valid.and_option_from(|| Some(index as usize))
    }

    fn reassign_fragments(&mut self, lines_count:usize) {
        let displayed_lines        = self.displayed_lines(lines_count);

        for (i, fragment) in self.fragments.iter_mut().enumerate() {
            let assigned_index     = displayed_lines.start() + i;
            let is_line_to_assign  = assigned_index <= *displayed_lines.end();
            fragment.assigned_line = is_line_to_assign.and_option(Some(i))
        }
    }

    fn build_buffer_data_for_fragments<'a, Indexes>
    (&self, fragments:Indexes, content:ContentRef<'_, 'a>)
    -> FragmentsDataBuilder<'a>
    where Indexes : Iterator<Item=usize> {
        let font  = content.font;
        let lines = content.lines;
        let mut builder = self.create_fragments_data_builder(font);
        for fragment_id in fragments {
            let fragment = &self.fragments[fragment_id];
            let index    = fragment.assigned_line.unwrap_or(0);
            let line     = fragment.assigned_line.map_or("", |i| lines[i].as_str());
            builder.build_for_line(index, line);
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
            font,
            line_clip            : line_clip_left..line_clip_right,
            max_displayed_chars  : self.max_displayed_chars
        }
    }

    fn set_vertex_position_buffer_subdata
    (&self, gl_context:&Context, fragment_offset:usize, builder:&FragmentsDataBuilder) {
        let fragment_size = GlyphVertexPositionBuilder::OUTPUT_SIZE * self.max_displayed_chars;
        let offset        = fragment_size * fragment_offset;
        let data          = builder.vertex_position_data.as_ref();
        self.set_buffer_subdata(gl_context,&self.vertex_position,offset,data);
    }

    fn set_texture_coords_buffer_subdata
    (&self, gl_context:&Context, fragment_offset:usize, builder:&FragmentsDataBuilder) {
        let fragment_size = GlyphTextureCoordsBuilder::OUTPUT_SIZE * self.max_displayed_chars;
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
