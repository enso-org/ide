//! This module defines render composer, a render pipeline bound to a specific context.

use crate::prelude::*;

use crate::system::gpu::data::texture::class::TextureOps;
use crate::display::render::pipeline::*;
use crate::system::gpu::*;
use js_sys::Array;



// ======================
// === RenderComposer ===
// ======================

shared! { RenderComposer
/// Render composer is a render pipeline bound to a specific context.
#[derive(Debug)]
pub struct RenderComposerData {
    passes    : Vec<ComposerPass>,
    variables : UniformScope,
    context   : Context,
    width     : i32,
    height    : i32,
}

impl {
    /// Constructor
    pub fn new
    ( pipeline  : &RenderPipeline
    , context   : &Context
    , variables : &UniformScope
    , width     : i32
    , height    : i32
    ) -> Self {
        let passes    = default();
        let context   = context.clone();
        let variables = variables.clone_ref();
        let mut this  = Self {passes,variables,context,width,height};
        for pass in pipeline.passes_clone() { this.add(pass); };
        this
    }

    fn add(&mut self, pass:Box<dyn RenderPass>) {
        let pass = ComposerPass::new(&self.context,&self.variables,pass,self.width,self.height);
        self.passes.push(pass);
    }

    /// Run all the registered passes in this composer.
    pub fn run(&mut self) {
        for pass in &mut self.passes {
            pass.run();
        }
    }
}}



// ====================
// === ComposerPass ===
// ====================

/// A `RenderPass` bound to a specific rendering context.
#[derive(Derivative)]
#[derivative(Debug)]
struct ComposerPass {
    #[derivative(Debug="ignore")]
    pass    : Box<dyn RenderPass>,
    instance : PassInstance,
}

impl Deref for ComposerPass {
    type Target = PassInstance;
    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl DerefMut for ComposerPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.instance
    }
}

impl ComposerPass {
    /// Constructor
    #[allow(clippy::borrowed_box)]
    pub fn new
    ( context   : &Context
    , variables : &UniformScope
    , mut pass  : Box<dyn RenderPass>
    , width     : i32
    , height    : i32
    ) -> Self {
        let instance = PassInstance::new(context,variables,width,height);
        pass.initialize(&instance);
        Self {pass,instance}
    }

    /// Run the pass.
    pub fn run(&mut self) {
        self.pass.run(&self.instance);
    }
}


#[derive(Debug)]
pub struct PassInstance {
    pub variables   : UniformScope,
    pub context     : Context,
    pub width       : i32,
    pub height      : i32,
}

impl PassInstance {
    /// Constructor
    #[allow(clippy::borrowed_box)]
    pub fn new
    ( context   : &Context
    , variables : &UniformScope
    , width     : i32
    , height    : i32
    ) -> Self {
        let variables    = variables.clone_ref();
        let context      = context.clone();
        Self {variables,context,width,height}
    }

    pub fn new_screen_texture(&self, output:&RenderPassOutput) -> AnyTextureUniform {
        let name    = format!("pass_{}",output.name());
        let args    = (self.width,self.height);
        uniform::get_or_add_gpu_texture_dyn
            (&self.context,&self.variables,&name,output.internal_format,output.item_type,args,
             Some(output.texture_parameters))
    }

    pub fn new_framebuffer(&self, textures:&[&AnyTextureUniform]) -> web_sys::WebGlFramebuffer {
        let context      = &self.context;
        let framebuffer  = self.context.create_framebuffer().unwrap();
        let target       = Context::FRAMEBUFFER;
        let draw_buffers = Array::new();
        context.bind_framebuffer(target,Some(&framebuffer));
        for (index,texture) in textures.into_iter().enumerate() {
            let texture_target   = Context::TEXTURE_2D;
            let attachment_point = Context::COLOR_ATTACHMENT0 + index as u32;
            let gl_texture       = texture.gl_texture();
            let gl_texture       = Some(&gl_texture);
            let level            = 0;
            draw_buffers.push(&attachment_point.into());
            context.framebuffer_texture_2d(target,attachment_point,texture_target,gl_texture,level);
        }
        context.draw_buffers(&draw_buffers);
        context.bind_framebuffer(target,None);
        framebuffer
    }
}
