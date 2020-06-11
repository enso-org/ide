//! This module implements `World`, the main object responsible for handling what you see on the
//! screen.

pub mod stats;

pub use crate::display::symbol::types::*;

use crate::prelude::*;

use crate::animation;
use crate::control::callback;
use crate::data::dirty::traits::*;
use crate::data::dirty;
use crate::debug::stats::Stats;
use crate::display::render::*;
use crate::display::render::passes::SymbolsRenderPass;
use crate::display::scene::Scene;
use crate::display::shape::text::text_field;
use crate::display;
use crate::system::web;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::Closure;



// ===============
// === Handles ===
// ===============

/// Callback handles managed by world.
#[derive(Clone,CloneRef,Debug)]
pub struct CallbackHandles {
    on_before_frame : callback::Handle,
    on_frame        : callback::Handle,
    on_after_frame  : callback::Handle,
}



// ================
// === Uniforms ===
// ================

/// Uniforms managed by world.
#[derive(Clone,CloneRef,Debug)]
pub struct Uniforms {
    time         : Uniform<f32>,
    display_mode : Uniform<i32>,
}

impl Uniforms {
    /// Constructor.
    pub fn new(scope:&UniformScope) -> Self {
        let time         = scope.add_or_panic("time",0.0);
        let display_mode = scope.add_or_panic("display_mode",0);
        Self {time,display_mode}
    }
}



// =============
// === World ===
// =============

/// World is the top-level application structure. It used to manage several instances of
/// `Scene`, and there is probability that we will get back to this design in the future.
/// It is responsible for updating the system on every animation frame.
#[derive(Clone,CloneRef,Debug)]
pub struct World {
    logger           : Logger,
    scene            : Scene,
    scene_dirty      : dirty::SharedBool,
    main_loop        : animation::DynamicLoop,
    uniforms         : Uniforms,
    stats            : Stats,
    stats_monitor    : stats::Monitor,
    focus_manager    : text_field::FocusManager, // FIXME: Move it to `Application`.
    callback_handles : CallbackHandles,
}

impl World {
    /// Create and initialize new world instance.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(dom:&web_sys::HtmlElement) -> World {
        let logger          = Logger::new("world");
        let stats           = default();
        let scene_dirty     = dirty::SharedBool::new(Logger::sub(&logger,"scene_dirty"),());
        let on_change       = enclose!((scene_dirty) move || scene_dirty.set());
        let scene           = Scene::new(dom,&logger,&stats,on_change);
        let uniforms        = Uniforms::new(&scene.variables);
        let main_loop       = animation::DynamicLoop::new();
        let stats_monitor   = stats::Monitor::new(&stats);
        let focus_manager   = text_field::FocusManager::new_with_js_handlers();

        let on_before_frame = main_loop.on_before_frame (f_!(stats_monitor.begin()));
        let on_after_frame  = main_loop.on_after_frame  (f_!(stats_monitor.end()));
        let on_frame        = main_loop.on_frame(
            f!([uniforms,scene_dirty,scene] (t:animation::TimeInfo) {
                uniforms.time.set(t.local);
                scene_dirty.unset_all();
                scene.update(t);
                scene.renderer.run();
            })
        );

        let callback_handles = CallbackHandles {on_before_frame,on_frame,on_after_frame};
        Self {scene,scene_dirty,logger,main_loop,uniforms,callback_handles,stats,stats_monitor
             ,focus_manager} . init()
    }

    fn init(self) -> Self {
        self.init_composer();
        self.init_hotkeys();
        self
    }

    fn init_hotkeys(&self) {
        // -----------------------------------------------------------------------------------------
        // FIXME[WD]: Hacky way of switching display_mode. To be fixed and refactored out.
        let stats_monitor = self.stats_monitor.clone_ref();
        let display_mode  = self.uniforms.display_mode.clone_ref();
        let c: Closure<dyn Fn(JsValue)> = Closure::wrap(Box::new(move |val| {
            let event = val.unchecked_into::<web_sys::KeyboardEvent>();
            if event.alt_key() && event.ctrl_key() {
                let key   = event.code();
                if      key == "Backquote" { stats_monitor.toggle() }
                else if key == "Digit0"    { display_mode.set(0) }
                else if key == "Digit1"    { display_mode.set(1) }
                else if key == "Digit2"    { display_mode.set(2) }
            }
        }));
        web::document().add_event_listener_with_callback
        ("keydown",c.as_ref().unchecked_ref()).unwrap();
        c.forget();
        // -----------------------------------------------------------------------------------------
    }

    fn init_composer(&self) {
        let mouse_hover_ids     = self.scene.mouse.hover_ids.clone_ref();
        let mut pixel_read_pass = PixelReadPass::<u8>::new(&self.scene.mouse.position);
        pixel_read_pass.set_callback(move |v| {
            mouse_hover_ids.set(Vector4::from_iterator(v.iter().map(|value| *value as u32)))
        });
        // TODO: We may want to enable it on weak hardware.
        // pixel_read_pass.set_threshold(1);
        let pipeline = RenderPipeline::new()
            .add(SymbolsRenderPass::new(&self.scene.symbols(),&self.scene.views))
            .add(ScreenRenderPass::new(self))
            .add(pixel_read_pass);
            // FIXME ugly way of rendering top layers:
        self.scene.renderer.set_pipeline(pipeline);
    }

    /// Scene accessor.
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Text field focus manager accessor.
    pub fn text_field_focus_manager(&self) -> &text_field::FocusManager {
        &self.focus_manager
    }

    /// Register a callback which should be run on each animation frame.
    pub fn on_frame<F:FnMut(animation::TimeInfo)+'static>
    (&self, mut callback:F) -> callback::Handle {
        self.main_loop.on_frame(move |time| callback(time))
    }

    /// Keeps the world alive even when all references are dropped. Use only if you want to keep one
    /// instance of the world forever.
    pub fn  keep_alive_forever(&self) {
        mem::forget(self.clone_ref())
    }
}

impl display::Object for World {
    fn display_object(&self) -> &display::object::Instance {
        self.scene.display_object()
    }
}

impl<'t> From<&'t World> for &'t Scene {
    fn from(world:&'t World) -> Self {
        &world.scene
    }
}
