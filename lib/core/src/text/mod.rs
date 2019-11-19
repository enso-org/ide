
use basegl_backend_webgl::{Context, compile_shader, link_program};
use basegl_core_msdf_sys as msdf_sys;
use basegl_core_fonts_base::FontsBase;

use wasm_bindgen::prelude::*;
use web_sys::WebGlRenderingContext;

pub fn print_line(context : &Context, text : &str)
{
    let vert_shader = compile_shader(
        &context,
        WebGlRenderingContext::VERTEX_SHADER,
        r#"
        attribute vec4 position;
        void main() {
            gl_Position = position;
        }
    "#,
    ).unwrap();
    let frag_shader = compile_shader(
        &context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        include_str!("msdf_frag.glsl")
    ).unwrap();
    let program = link_program(&context, &vert_shader, &frag_shader).unwrap();
    context.use_program(Some(&program));

    let msdf_location = context.get_uniform(program, "msdf");
    let bg_color_location = context.get_uniform(program, "bgColor");
    let fg_color_location = context.get_uniform(program, "fgColor");
    let px_range_location = context.get_uniform(program, "pxRange");

    let vertices= [
        -0.5, -0.5, 0.0,
        -0.5,  0.5, 0.0,
         0.5,  0.5, 0.0,
         0.5, -0.5, 0.0
    ];

    let buffer = context.create_buffer().unwrap();
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));

    let font_base = FontsBase::new();
    let font = msdf_sys::Font::load_from_memory(
        font_base.fonts_by_name.get("DejaVuSansMono-Bold").unwrap()
    );
    let params = msdf_sys::MSDFParameters {
        width: 16,
        height: 16,
        edge_coloring_angle_threshold: 3.0,
        range: 2.0,
        edge_threshold: 1.001,
        overlap_support: true
    };
    // when
    let msdf = msdf_sys::MutlichannelSignedDistanceField::generate(
        &font,
        text[0],
        &params,
        Vector2D { x: 1.0, y: 1.0 },
        Vector2D { x: 0.0, y: 0.0 }
    );

    let msdf_texture = context.create_texture();
    context.bind_texture(context.TEXTURE_2D, msdf_texture.as_ref());
    context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
        Context::TEXTURE_2D,
        0,
        Context::RGB32F,
        16,
        16,
        0,
        Context::RGB32F,
        gl.FLOAT,
        js_sys::Float32Array::view(&vertices[0..16*16*3])
    );

    context.uniform1i(msdf_location, 0);
    context.uniform4f(bg_color_location, 0.0, 0.0, 0.0, 0.0);
    context.uniform4f(fg_color_location, 255.0, 255.0, 255.0, 0.0);
    context.uniform1f(px_range_location, 1.0);

    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        let vert_array = js_sys::Float32Array::view(&vertices);

        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }

    context.vertex_attrib_pointer_with_i32(0, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
    context.enable_vertex_attrib_array(0);

    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    context.draw_arrays(
        WebGlRenderingContext::TRIANGLES,
        0,
        (vertices.len() / 3) as i32,
    );
}