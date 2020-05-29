#![allow(missing_docs)]

#[warn(missing_docs)]
pub mod builder;

use crate::prelude::*;

use crate::data::dirty::traits::*;
use crate::data::dirty;
use crate::debug::stats::Stats;
use crate::display::symbol::material::Material;
use crate::display::symbol::material::VarDecl;
use crate::display::symbol::ScopeType;
use crate::display::symbol::shader;
use crate::system::gpu::shader::*;
use crate::system::gpu::shader::Context;
use crate::control::callback::CallbackFn;

use web_sys::WebGlProgram;

use shapely::shared;



// ==================
// === VarBinding ===
// ==================

#[derive(Clone,Debug)]
pub struct VarBinding {
    pub name  : String,
    pub decl  : VarDecl,
    pub scope : Option<ScopeType>,
}

impl VarBinding {
    pub fn new<Name:Str>(name:Name, decl:VarDecl, scope:Option<ScopeType>) -> Self {
        let name = name.into();
        Self {name,decl,scope}
    }
}



// ==============
// === Shader ===
// ==============

pub type Dirty = dirty::SharedBool<Box<dyn Fn()>>;

shared! { Shader
/// Shader keeps track of a shader and related WebGL Program.
#[derive(Debug)]
pub struct ShaderData {
    geometry_material : Material,
    surface_material  : Material,
    program           : Option<WebGlProgram>,
    dirty             : Dirty,
    logger            : Logger,
    context           : Context,
    stats             : Stats,
}

impl {

    pub fn program(&self) -> Option<WebGlProgram> {
        self.program.clone()
    }

    pub fn set_geometry_material<M:Into<Material>>(&mut self, material:M) {
        self.geometry_material = material.into();
        self.dirty.set();
    }

    pub fn set_material<M:Into<Material>>(&mut self, material:M) {
        self.surface_material = material.into();
        self.dirty.set();
    }

    /// Creates new shader with attached callback.
    pub fn new<OnMut:CallbackFn>(logger:Logger, stats:&Stats, context:&Context, on_mut:OnMut) -> Self {
        stats.inc_shader_count();
        let geometry_material = default();
        let surface_material  = default();
        let program           = default();
        let dirty_logger      = Logger::sub(&logger,"dirty");
        let dirty             = Dirty::new(dirty_logger,Box::new(on_mut));
        let context           = context.clone();
        let stats             = stats.clone_ref();
        dirty.set();
        Self {geometry_material,surface_material,program,dirty,logger,context,stats}
    }

    // TODO: this is very work-in-progress function. It should be refactored in the next PR.
    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self, bindings:&[VarBinding]) {
        group!(self.logger, "Updating.", {
            if self.dirty.check_all() {

                self.stats.inc_shader_compile_count();

                let mut shader_cfg     = shader::builder::ShaderConfig::new();
                let mut shader_builder = shader::builder::ShaderBuilder::new();

                for binding in bindings {
                    let name = &binding.name;
                    let tp   = &binding.decl.tp;
                    match binding.scope {
                        None => {
                            self.logger.warning("TODO: default shader values.");
                            shader_cfg.add_uniform(name,tp);
                        },
                        Some(scope_type) => match scope_type {
                            ScopeType::Symbol => shader_cfg.add_uniform   (name,tp),
                            ScopeType::Global => shader_cfg.add_uniform   (name,tp),
                            _                 => shader_cfg.add_attribute (name,tp),
                        }
                    }
                }

                self.geometry_material.outputs().iter().for_each(|(name,decl)|{
                    shader_cfg.add_shared_attribute(name,&decl.tp);
                });

                shader_cfg.add_output("color", glsl::PrimType::Vec4);
                self.surface_material.outputs().iter().for_each(|(name,decl)|{
                    shader_cfg.add_output(name,&decl.tp);
                });

                let vertex_code   = self.geometry_material.code().clone();
                let fragment_code = self.surface_material.code().clone();
                shader_builder.compute(&shader_cfg,vertex_code,fragment_code);
                let shader      = shader_builder.build();
                let vert_shader = compile_vertex_shader  (&self.context,&shader.vertex);
                let frag_shader = compile_fragment_shader(&self.context,&shader.fragment);
                if let Err(ref err) = frag_shader {
                    self.logger.error(|| format!("{}", err))
                }

                let vert_shader = vert_shader.unwrap();
                let frag_shader = frag_shader.unwrap();
                let program     = link_program(&self.context,&vert_shader,&frag_shader);

                let program     = program.unwrap();
                self.program    = Some(program);
                self.dirty.unset_all();
            }
        })
    }

    /// Traverses the shader definition and collects all attribute names.
    pub fn collect_variables(&self) -> BTreeMap<String,VarDecl> {
        let geometry_material_inputs = self.geometry_material.inputs().clone();
        let surface_material_inputs  = self.surface_material.inputs().clone();
        geometry_material_inputs.into_iter().chain(surface_material_inputs).collect()
    }
}}

impl Drop for ShaderData {
    fn drop(&mut self) {
        self.stats.dec_shader_count();
    }
}
