use basegl_backend_webgl::{Context, compile_shader, link_program};
use basegl_core_msdf_sys as msdf_sys;
use basegl_core_fonts_base::FontsBase;

use web_sys::WebGlRenderingContext;
use basegl_system_web::Logger;

pub fn print_line(context : &Context, text : &str, logger : &Logger)
{
    let vert_shader = compile_shader(
        &context,
        WebGlRenderingContext::VERTEX_SHADER,
        include_str!("msdf_vert.glsl")
    ).unwrap();
//    logger.trace(|| format!("{:?}", vert_shader.as_ref().expect_err("No error?")));
    context.get_extension("OES_standard_derivatives").unwrap();
    let frag_shader = compile_shader(
        &context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        include_str!("msdf_frag.glsl")
    ).unwrap();
//    logger.trace(|| format!("{:?}", frag_shader.as_ref().expect_err("No error?")));
    let program = link_program(&context, &vert_shader, &frag_shader).unwrap();
    context.use_program(Some(&program));

    let msdf_location = context.get_uniform_location(&program, "msdf");
    let msdf_size_location = context.get_uniform_location(&program, "msdf_size");
    let bg_color_location = context.get_uniform_location(&program, "bgColor");
    let fg_color_location = context.get_uniform_location(&program, "fgColor");
    let px_range_location = context.get_uniform_location(&program, "pxRange");

    let font_base = FontsBase::new();
    let mfont = font_base.fonts_by_name.get("DejaVuSansMono-Bold").unwrap();
    let font = msdf_sys::Font::load_from_memory(mfont);
    let params = msdf_sys::MSDFParameters {
        width: 32,
        height: 32,
        edge_coloring_angle_threshold: 3.0,
        range: 2.0,
        edge_threshold: 1.001,
        overlap_support: true
    };
    // when
    let msdf = msdf_sys::MutlichannelSignedDistanceField::generate(
        &font,
        text.chars().next().unwrap() as u32,
        &params,
        msdf_sys::Vector2D { x: 1.0, y: 1.0 },
        msdf_sys::Vector2D { x: 4.0, y: 4.0 }
    );

    let u8msdf = msdf.data[0..32*32*3].iter().map(|f| nalgebra::clamp(f*255.0, 0.0, 255.0) as u8).collect::<Vec<u8>>();
//    logger.trace(|| format!("{:?}", (msdf.data[0..16*16*3].to_vec())));
//    logger.trace(|| format!("{:?}", u8msdf));

    let msdf_texture = context.create_texture();
//    context.active_texture(Context::TEXTURE0);
    context.bind_texture(Context::TEXTURE_2D, msdf_texture.as_ref());
    let res = context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        Context::TEXTURE_2D,
        0,
        Context::RGB as i32,
        32,
        32,
        0,
        Context::RGB,
        Context::UNSIGNED_BYTE,
        Some(&u8msdf)
    );
    context.tex_parameteri(Context::TEXTURE_2D, Context::TEXTURE_WRAP_S, Context::CLAMP_TO_EDGE as i32);
    context.tex_parameteri(Context::TEXTURE_2D, Context::TEXTURE_WRAP_T, Context::CLAMP_TO_EDGE as i32);
    context.tex_parameteri(Context::TEXTURE_2D, Context::TEXTURE_MIN_FILTER, Context::LINEAR as i32);
//    logger.trace(|| format!("{:?}", res.as_ref().expect_err("No error?")));
    context.uniform1i(msdf_location.as_ref(), 0);
    context.uniform1i(msdf_size_location.as_ref(), 16);
    context.uniform4f(bg_color_location.as_ref(), 0.0, 0.0, 0.0, 1.0);
    context.uniform4f(fg_color_location.as_ref(), 255.0, 255.0, 255.0, 1.0);
    context.uniform1f(px_range_location.as_ref(), 1.0);

    let vertices= [
        -1.0, -1.0, 0.0,
        -1.0,  1.0, 0.0,
         1.0,  1.0, 0.0,
         1.0,  1.0, 0.0,
         1.0, -1.0, 0.0,
        -1.0, -1.0, 0.0
    ];

    let buffer = context.create_buffer().unwrap();
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));

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