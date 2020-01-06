use crate::prelude::*;

use crate::system::gpu::data::GpuData;
use crate::display::render::webgl::glsl;
use crate::display::symbol::shader::builder::CodeTemplete;



// ===============
// === VarDecl ===
// ===============

#[derive(Clone,Debug)]
pub struct VarDecl {
    pub tp      : glsl::PrimType,
    pub default : String,
}

impl VarDecl {
    pub fn new(tp:glsl::PrimType, default:String) -> Self {
        Self {tp,default}
    }
}



// ================
// === Material ===
// ================

#[derive(Clone,Debug,Default,Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct Material {
    #[shrinkwrap(main_field)]
    pub code    : CodeTemplete,
    pub inputs  : BTreeMap<String,VarDecl>,
    pub outputs : BTreeMap<String,VarDecl>,
}

impl Material {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }

    pub fn add_input<Name:Str,T:GpuData>(&mut self, name:Name, t:T) {
        self.inputs.insert(name.into(),Self::make_var_decl(t));
    }

    pub fn add_output<Name:Str,T:GpuData>(&mut self, name:Name, t:T) {
        self.outputs.insert(name.into(),Self::make_var_decl(t));
    }

    pub fn make_var_decl<T:GpuData>(t:T) -> VarDecl {
        VarDecl::new(<T as GpuData>::glsl_type(), t.to_glsl())
    }
}

impl From<&Material> for Material {
    fn from(t:&Material) -> Self {
        t.clone()
    }
}

// === Setters ===

impl Material {
    pub fn set_code<T:Into<CodeTemplete>>(&mut self, code:T) {
        self.code = code.into();
    }
}
