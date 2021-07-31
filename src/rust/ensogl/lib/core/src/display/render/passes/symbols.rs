//! Pass for rendering all symbols.

use crate::prelude::*;

use crate::display::render::composer::PassInstance;
use crate::display::render::pipeline::*;
use crate::display::scene::layer;
use crate::display::scene;
use crate::display::symbol::registry::SymbolRegistry;
use crate::system::gpu::*;
use wasm_bindgen::JsCast;



// =========================
// === SymbolsRenderPass ===
// =========================

/// Pass for rendering all symbols. The results are stored in the 'color' and 'id' outputs.
#[derive(Clone,Debug)]
pub struct SymbolsRenderPass {
    logger          : Logger,
    symbol_registry : SymbolRegistry,
    layers          : scene::HardcodedLayers,
    color_fb        : Option<web_sys::WebGlFramebuffer>,
    mask_fb         : Option<web_sys::WebGlFramebuffer>,
    scissor_stack   : Vec<layer::ScissorBox>,
}

impl SymbolsRenderPass {
    /// Constructor.
    pub fn new
    ( logger          : impl AnyLogger
    , symbol_registry : &SymbolRegistry
    , layers          : &scene::HardcodedLayers
    ) -> Self {
        let logger          = Logger::new_sub(logger,"SymbolsRenderPass");
        let symbol_registry = symbol_registry.clone_ref();
        let layers          = layers.clone_ref();
        let color_fb        = default();
        let mask_fb         = default();
        let scissor_stack   = default();
        Self {logger,symbol_registry,layers,color_fb,mask_fb,scissor_stack}
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

        self.render_layer(instance,&self.layers.main.clone());

        if !self.scissor_stack.is_empty() {
            warning!(&self.logger,"The scissor stack was not cleaned properly. \
                This is an internal bug that may lead to visual artifacts. Please report it.");
            self.scissor_stack = default();
        }
        instance.context.bind_framebuffer(Context::FRAMEBUFFER,None);
    }
}

impl SymbolsRenderPass {
    fn render_layer(&mut self, instance:&PassInstance, layer:&layer::Layer) {
        let parent_scissor_box  = self.scissor_stack.first().copied();
        let layer_scissor_box   = layer.scissor_box();
        let scissor_box         = parent_scissor_box.concat(layer_scissor_box);
        let scissor_box_changed = layer_scissor_box.is_some();
        let scissor_root        = scissor_box_changed && parent_scissor_box.is_none();
        if let Some(scissor_box) = scissor_box {
            if scissor_box_changed {
                if scissor_root {
                    instance.context.enable(web_sys::WebGl2RenderingContext::SCISSOR_TEST);
                }
                self.scissor_stack.push(scissor_box);
                let position = scissor_box.position();
                let size     = scissor_box.size();
                instance.context.scissor(position.x,position.y,size.x,size.y);
            }
        }
        self.symbol_registry.set_camera(&layer.camera());
        self.symbol_registry.render_by_ids(&layer.symbols());

        for sublayer in layer.sublayers().iter() {
            self.render_layer(instance,&sublayer);
        }

        if scissor_box_changed {
            self.scissor_stack.pop();
            if scissor_root {
                instance.context.disable(web_sys::WebGl2RenderingContext::SCISSOR_TEST);
            }
        }
    }
}
