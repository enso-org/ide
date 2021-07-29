//! Pass for rendering all symbols.

use crate::prelude::*;

use crate::display::render::pipeline::*;
use crate::system::gpu::*;
use crate::display::symbol::registry::SymbolRegistry;
use crate::display::scene;
use crate::display::render::composer::PassInstance;



// =========================
// === SymbolsRenderPass ===
// =========================

/// Pass for rendering all symbols. The results are stored in the 'color' and 'id' outputs.
#[derive(Clone,Debug)]
pub struct SymbolsRenderPass {
    target   : SymbolRegistry,
    layers   : scene::HardcodedLayers,
    color_fb : Option<web_sys::WebGlFramebuffer>,
    mask_fb  : Option<web_sys::WebGlFramebuffer>,
}

impl SymbolsRenderPass {
    /// Constructor.
    pub fn new(target:&SymbolRegistry, layers:&scene::HardcodedLayers) -> Self {
        let target   = target.clone_ref();
        let layers   = layers.clone_ref();
        let color_fb = default();
        let mask_fb  = default();
        Self {target,layers,color_fb,mask_fb}
    }
}

impl RenderPass for SymbolsRenderPass {
    fn initialize(&mut self, instance:&PassInstance) {
        let rgba         = texture::Rgba;
        let tex_type     = texture::item_type::u8;
        let color_params = texture::Parameters::default();
        let id_params    = texture::Parameters {
            min_filter : texture::MinFilter::Nearest,
            mag_filter : texture::MagFilter::Nearest,
            ..default()
        };
        let out_color = RenderPassOutput::new("color",rgba,tex_type,color_params);
        let out_mask  = RenderPassOutput::new("mask" ,rgba,tex_type,color_params);
        let out_id    = RenderPassOutput::new("id"   ,rgba,tex_type,id_params);
        let tex_color = instance.new_screen_texture(&out_color);
        let tex_mask  = instance.new_screen_texture(&out_mask);
        let tex_id    = instance.new_screen_texture(&out_id);
        self.color_fb = Some(instance.new_framebuffer(&[&tex_color,&tex_id]));
        self.mask_fb  = Some(instance.new_framebuffer(&[&tex_mask,&tex_id]));
    }

    fn run(&mut self, instance:&PassInstance) {
        instance.context.bind_framebuffer(Context::FRAMEBUFFER,self.color_fb.as_ref());

        let arr = vec![0.0,0.0,0.0,0.0];
        instance.context.clear_bufferfv_with_f32_array(Context::COLOR,0,&arr);
        instance.context.clear_bufferfv_with_f32_array(Context::COLOR,1,&arr);

        // FIXME: Please note that rendering of masks and nested layers is not implemented yet.
        for layer in self.layers.sublayers().iter() {
            self.target.set_camera(&layer.camera());
            let symbols = layer.symbols();
            self.target.render_by_ids(&symbols);
        }

        instance.context.bind_framebuffer(Context::FRAMEBUFFER,None);
    }
}
