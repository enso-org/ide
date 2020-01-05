#![allow(missing_docs)]

#[warn(missing_docs)]
pub mod builder;

use crate::prelude::*;

use crate::data::dirty::traits::*;
use crate::data::dirty;
use crate::data::function::callback::*;
use crate::debug::stats::Stats;
use crate::display::render::webgl::Context;
use crate::display::render::webgl::glsl;
use crate::display::render::webgl;
use crate::display::shape::primitive::shader::builder::Builder;
use crate::display::symbol::material::Material;
use crate::display::symbol::shader;
use crate::system::web::group;
use crate::system::web::Logger;

use web_sys::WebGlProgram;



// ==============
// === Shader ===
// ==============

// === Definition ===

/// Shader keeps track of a shader and related WebGL Program.
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Shader<OnMut> {
    geometry   : Material,
    material   : Material,
    program    : Option<WebGlProgram>,
    pub dirty  : Dirty <OnMut>,
    pub logger : Logger,
    context    : Context,
    stats      : Stats,
}

// === Types ===

pub type Dirty <F> = dirty::SharedBool<F>;

#[macro_export]
/// Promote relevant types to parent scope. See `promote!` macro for more information.
macro_rules! promote_shader_types { ($($args:tt)*) => {
    promote! {$($args)* [Shader]}
};}

// === Implementation ===

impl<OnMut:Callback0> Shader<OnMut> {

    /// Creates new shader with attached callback.
    pub fn new(logger:Logger, stats:&Stats, context:&Context, on_mut:OnMut) -> Self {
        stats.inc_shader_count();
        let geometry     = default();
        let material     = default();
        let program      = default();
        let dirty_logger = logger.sub("dirty");
        let dirty        = Dirty::new(dirty_logger,on_mut);
        let context      = context.clone();
        let stats        = stats.clone_ref();
        dirty.set();
        Self {geometry,material,program,dirty,logger,context,stats}
    }

    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self) {
        group!(self.logger, "Updating.", {
            if self.dirty.check_all() {

                        println!("UPDATE SHADER");

                self.stats.inc_shader_compile_count();


                let mut shader_cfg     = shader::builder::ShaderConfig::new();
                let mut shader_builder = shader::builder::ShaderBuilder::new();

//                for t in &self.geometry.attributes {
//                    shader_cfg.add_attribute(&t.name,&t.glsl_type);
//                }
//
//                for t in &self.geometry.uniforms {
//                    shader_cfg.add_uniform(&t.name,&t.glsl_type);
//                }
//
//                for t in &self.geometry.outputs {
//                    shader_cfg.add_shared_attribute(&t.name,&t.glsl_type);
//                }


                shader_cfg.add_attribute        ("bbox"            , glsl::PrimType::Vec2);
                shader_cfg.add_attribute        ("uv"              , glsl::PrimType::Vec2);
                shader_cfg.add_attribute        ("transform"       , glsl::PrimType::Mat4);
                shader_cfg.add_shared_attribute ("local"           , glsl::PrimType::Vec3);
                shader_cfg.add_uniform          ("view_projection" , glsl::PrimType::Mat4);
                shader_cfg.add_output           ("color"           , glsl::PrimType::Vec4);

                let vtx_template = shader::builder::CodeTemplete::from_main("
                mat4 model_view_projection = view_projection * transform;
                local                      = vec3((uv - 0.5) * bbox, 0.0);
                gl_Position                = model_view_projection * vec4(local,1.0);
                ");
                let frag_template = shader::builder::CodeTemplete::from_main("
                output_color = vec4(1.0,1.0,1.0,1.0);
                ");
                shader_builder.compute(&shader_cfg,vtx_template,frag_template);
                let shader      = shader_builder.build();
                let vert_shader = webgl::compile_vertex_shader  (&self.context,&shader.vertex);
                let frag_shader = webgl::compile_fragment_shader(&self.context,&shader.fragment);
                let vert_shader = vert_shader.unwrap();
                let frag_shader = frag_shader.unwrap();
                let program     = webgl::link_program(&self.context,&vert_shader,&frag_shader);
                let program     = program.unwrap();
                self.program    = Some(program);
                self.dirty.unset_all();
            }
        })
    }

    /// Traverses the shader definition and collects all attribute names.
    pub fn collect_variables(&self) -> Vec<String> {
        // FIXME: Hardcoded.
        vec!["bbox".into(),"uv".into(),"transform".into(),"view_projection".into()]
    }
}

impl<OnMut> Drop for Shader<OnMut> {
    fn drop(&mut self) {
        self.stats.dec_shader_count();
    }
}


// === Getters ===

impl<OnMut> Shader<OnMut> {
    pub fn program(&self) -> &Option<WebGlProgram> {
        &self.program
    }
}


// === Setters ===

impl<OnMut:Callback0> Shader<OnMut> {
    pub fn set_geometry_material(&mut self, material:Material) {
        println!("SET GEO MAT");
        self.geometry = material;
        self.dirty.set();
    }

    pub fn set_material(&mut self, material:Material) {
        self.material = material;
        self.dirty.set();
    }
}