use crate::prelude::*;

use basegl_backend_webgl::{Program, Context, compile_shader, link_program};
use nalgebra::{Point2, Vector2};
use web_sys::WebGlBuffer;
use web_sys::WebGl2RenderingContext;
use web_sys::WebGlUniformLocation;
use crate::Color;
use crate::text::font::FontRenderInfo;
use crate::text::msdf::MsdfTexture;


#[derive(Debug)]
pub struct BasicProgram {
    pub gl_program   : Program,
    pub to_scene_loc : WebGlUniformLocation
}

impl BasicProgram {
    pub fn new(gl_context:&Context, vertex_shader_body:&str, fragment_shader_body:&str) -> Self {
        let vert_type    = WebGl2RenderingContext::VERTEX_SHADER;
        let frag_type    = WebGl2RenderingContext::FRAGMENT_SHADER;
        let vert_shader  = compile_shader(gl_context,vert_type,vertex_shader_body).unwrap();
        let frag_shader  = compile_shader(gl_context,frag_type,fragment_shader_body).unwrap();
        let gl_program   = link_program(&gl_context, &vert_shader, &frag_shader).unwrap();
        let to_scene_loc = gl_context.get_uniform_location(&gl_program,"to_scene").unwrap();
        BasicProgram {gl_program,to_scene_loc}
    }

    pub fn set_constant_uniforms(&self, gl_context:&Context, position:&Point2<f64>, size:Vector2<f64>, color:&Color<f32>) {
        let left           = position.x            as f32;
        let right          = (position.x + size.x) as f32;
        let top            = (position.y + size.y) as f32;
        let bottom         = position.y            as f32;
        let clip_lower_loc = gl_context.get_uniform_location(&self.gl_program,"clip_lower");
        let clip_upper_loc = gl_context.get_uniform_location(&self.gl_program,"clip_upper");
        let color_loc      = gl_context.get_uniform_location(&self.gl_program,"color");

        gl_context.use_program(Some(&self.gl_program));
        gl_context.uniform2f(clip_lower_loc.as_ref(),left,bottom);
        gl_context.uniform2f(clip_upper_loc.as_ref(),right,top);
        gl_context.uniform4f(color_loc.as_ref(),color.r,color.g,color.b,color.a);
    }

    pub fn set_to_scene_transformation(&self, gl_context:&Context, matrix:&SmallVec<[f32;9]>) {
        let to_scene_ref  = matrix.as_ref();
        let to_scene_loc  = Some(&self.to_scene_loc);
        let transpose     = false;
        gl_context.use_program(Some(&self.gl_program));
        gl_context.uniform_matrix3fv_with_f32_array(to_scene_loc,transpose,to_scene_ref);
    }

    pub fn bind_buffer_to_attribute(&self, gl_context:&Context, attribute_name:&str, buffer:&WebGlBuffer) {
        let gl_program = &self.gl_program;
        let location   = gl_context.get_attrib_location(gl_program,attribute_name) as u32;
        let target     = WebGl2RenderingContext::ARRAY_BUFFER;
        let item_size  = 2;
        let item_type  = WebGl2RenderingContext::FLOAT;
        let normalized = false;
        let stride     = 0;
        let offset     = 0;

        gl_context.enable_vertex_attrib_array(location);
        gl_context.bind_buffer(target,Some(buffer));
        gl_context.vertex_attrib_pointer_with_i32
            (location,item_size,item_type,normalized,stride,offset);
    }
}

#[derive(Debug,Shrinkwrap)]
pub struct  MsdfProgram {
    #[shrinkwrap(main_field)]
    pub program       : BasicProgram,
    pub msdf_size_loc : WebGlUniformLocation,
}

impl MsdfProgram {

    pub fn new(gl_context:&Context, vertex_shader_body:&str, fragment_shader_body:&str) -> Self {
        let program       = BasicProgram::new(gl_context,vertex_shader_body,fragment_shader_body);
        let msdf_size_loc = gl_context.get_uniform_location(&program.gl_program,"msdf_size").unwrap();
        MsdfProgram{program,msdf_size_loc}
    }

    pub fn set_constant_uniforms(&self, gl_context:&Context, position:&Point2<f64>, size:Vector2<f64>, color:&Color<f32>) {
        let range     = FontRenderInfo::MSDF_PARAMS.range as f32;
        let msdf_loc  = gl_context.get_uniform_location(&self.gl_program,"msdf");
        let range_loc = gl_context.get_uniform_location(&self.gl_program,"range");

        gl_context.use_program(Some(&self.gl_program));
        self.program.set_constant_uniforms(gl_context,position,size,color);
        gl_context.uniform1f(range_loc.as_ref(),range);
        gl_context.uniform1i(msdf_loc.as_ref(),0);
    }

    pub fn set_msdf_size(&self, gl_context:&Context, font:&FontRenderInfo) {
        let msdf_width    = MsdfTexture::WIDTH as f32;
        let msdf_height   = font.msdf_texture.rows() as f32;

        gl_context.use_program(Some(&self.program.gl_program));
        gl_context.uniform2f(Some(&self.msdf_size_loc),msdf_width,msdf_height);
    }
}

pub fn create_content_program(gl_context:&Context) -> MsdfProgram {
    let vert_shader_body = include_str!("program/msdf_vert.glsl");
    let frag_shader_body = include_str!("program/msdf_frag.glsl");
    MsdfProgram::new(gl_context,vert_shader_body,frag_shader_body)
}

pub fn create_cursors_program(gl_context:&Context) -> BasicProgram {
    let vert_shader_body = include_str!("program/cursor_vert.glsl");
    let frag_shader_body = include_str!("program/cursor_frag.glsl");
    BasicProgram::new(gl_context,vert_shader_body,frag_shader_body)
}