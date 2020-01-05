use crate::prelude::*;

use crate::system::gpu::data::GpuData;
use crate::display::render::webgl::glsl;



#[derive(Clone,Debug)]
pub struct Binding {
    pub name         : String,
    pub glsl_type    : glsl::PrimType,
    pub glsl_default : String,
}

impl Binding {
    pub fn new(name:String, glsl_type:glsl::PrimType, glsl_default:String) -> Self {
        Self {name,glsl_type,glsl_default}
    }
}



// ================
// === Material ===
// ================

#[derive(Clone,Debug,Default)]
pub struct Material {
    pub inputs      : Vec<Binding>,
    pub outputs     : Vec<Binding>,
    pub before_main : String,
    pub main        : String,
}

impl Material {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }

    pub fn add_input<Name:Into<String>,T:GpuData>(&mut self, name:Name, t:T) {
        self.inputs.push(Self::make_binding(name,t));
    }

    pub fn add_output<Name:Into<String>,T:GpuData>(&mut self, name:Name, t:T) {
        self.outputs.push(Self::make_binding(name,t));
    }

    pub fn set_before_main<Code:Into<String>>(&mut self, code:Code) {
        self.before_main = code.into()
    }

    pub fn set_main<Code:Into<String>>(&mut self, code:Code) {
        self.main = code.into()
    }

    pub fn make_binding<Name:Into<String>,T:GpuData>(name:Name, t:T) -> Binding {
        Binding::new(name.into(), <T as GpuData>::glsl_type(), t.to_glsl())
    }
}
