#![allow(missing_docs)]

#[warn(missing_docs)]
pub mod stats;

use crate::prelude::*;

pub use crate::data::container::*;
pub use crate::display::symbol::types::*;
pub use crate::display::scene::SymbolId;
pub use stats::*;

use crate::closure;
use crate::control::callback::CallbackHandle;
use crate::control::event_loop::EventLoop;
use crate::data::dirty::traits::*;
use crate::data::dirty;
use crate::debug::stats::Stats;
use crate::display::render::*;
use crate::display::scene::Scene;
use crate::display::symbol::Symbol;
use crate::display;
use crate::display::traits::*;
use crate::system::web;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::Closure;
use web_sys::KeyboardEvent;
use web_sys::Performance;
use crate::display::render::passes::SymbolsRenderPass;
use crate::display::shape::text::text_field;



// =============
// === World ===
// =============

// === Definition ===

/// World is the top-level application structure. It used to manage several instances of
/// `Scene`, and there is probability that we will get back to this design in the future.
/// It is responsible for updating the system on every animation frame.
#[derive(Clone,Debug)]
pub struct World {
    scene         : Scene,
    scene_dirty   : SceneDirty,
    logger        : Logger,
    event_loop    : EventLoop,
    performance   : Performance,
    start_time    : Immutable<f32>,
    time          : Uniform<f32>,
    display_mode  : Uniform<i32>,
    update_handle : Rc<RefCell<Option<CallbackHandle>>>,
    stats         : Stats,
    stats_monitor : StatsMonitor,
    focus_manager : text_field::FocusManager,
}

impl CloneRef for World {
    fn clone_ref(&self) -> Self {
        let scene         = self.scene.clone_ref();
        let scene_dirty   = self.scene_dirty.clone_ref();
        let logger        = self.logger.clone_ref();
        let event_loop    = self.event_loop.clone_ref();
        let performance   = self.performance.clone_ref();
        let start_time    = self.start_time.clone_ref();
        let time          = self.time.clone_ref();
        let display_mode  = self.display_mode.clone_ref();
        let update_handle = self.update_handle.clone_ref();
        let stats         = self.stats.clone_ref();
        let stats_monitor = self.stats_monitor.clone_ref();
        let focus_manager = self.focus_manager.clone_ref();
        Self {scene,scene_dirty,logger,event_loop,performance,start_time,time,display_mode
             ,update_handle,stats,stats_monitor,focus_manager}
    }
}


// === Types ===

pub type SceneID    = usize;
pub type SceneDirty = dirty::SharedBool;


// === Callbacks ===

closure! {
fn scene_on_change(dirty:SceneDirty) -> SceneOnChange {
    || dirty.set()
}}


// === Implementation ===

impl World {
    /// Create and initialize new world instance.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(dom:&web_sys::HtmlElement) -> World {
        let mut this = Self::new_uninitialized(dom);
        this.init_composer();
        let time        = this.time.clone_ref();
        let scene_dirty = this.scene_dirty.clone_ref();
        let scene       = this.scene.clone_ref();
        let performance = this.performance.clone_ref();
        let start_time  = this.start_time;
        let update      = move |_:&f64| {
            let relative_time = performance.now() as f32 - *start_time;
            time.set(relative_time);
                // group!(self.logger, "Updating.", {
                scene_dirty.unset_all();
                scene.update();
                scene.renderer.run();
                // });
        };
        let update_handle  = this.event_loop.add_callback(update);
        this.update_handle = Rc::new(RefCell::new(Some(update_handle)));

        // -----------------------------------------------------------------------------------------
        // FIXME[WD]: Hacky way of switching display_mode. To be fixed and refactored out.
        // FIXME[AO]: Commented out for Sylwia request. Should we keep this debug mode anyway?
//        let stats_monitor = this.stats_monitor.clone_ref();
//        let display_mode  = this.display_mode.clone_ref();
//        let c: Closure<dyn Fn(JsValue)> = Closure::wrap(Box::new(move |val| {
//            let val = val.unchecked_into::<KeyboardEvent>();
//            let key = val.key();
//            if      key == "`" { stats_monitor.toggle() }
//            else if key == "0" { display_mode.set(0) }
//            else if key == "1" { display_mode.set(1) }
//        }));
//        web::document().add_event_listener_with_callback
//        ("keydown",c.as_ref().unchecked_ref()).unwrap();
//        c.forget();
        // -----------------------------------------------------------------------------------------

        this
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn text_field_focus_manager(&self) -> &text_field::FocusManager {
        &self.focus_manager
    }

    pub fn add_child<T:display::Object>(&self, child:&T) {
        self.scene().display_object().add_child(child)
    }

    pub fn on_frame<F:FnMut(&f64)+'static>
    (&self, mut callback:F) -> CallbackHandle {
        let func = move |time_ms:&f64| callback(time_ms);
        self.event_loop.add_callback(func)
    }

//    pub fn display_object(&self) -> display::object::Node {
//        self.scene().display_object()
//    }

    /// Create new uninitialized world instance. You should rather not need to
    /// call this function directly.
    fn new_uninitialized(dom:&web_sys::HtmlElement) -> Self {
        let stats              = default();
        let logger             = Logger::new("world");
        let scene_logger       = logger.sub("scene");
        let scene_dirty_logger = logger.sub("scene_dirty");
        let scene_dirty        = SceneDirty::new(scene_dirty_logger,());
        let scene_dirty2       = scene_dirty.clone();
        let on_change          = move || {scene_dirty2.set()};
        let scene              = Scene::new(dom,scene_logger,&stats,on_change);
        let time               = scene.variables.add_or_panic("time",0.0);
        let display_mode       = scene.variables.add_or_panic("display_mode",0);
        let event_loop         = EventLoop::new();
        let update_handle      = default();
        let stats_monitor      = StatsMonitor::new(&stats);
        let performance        = web::performance();
        let start_time         = Immutable(performance.now() as f32);
        let focus_manager      = text_field::FocusManager::new_with_js_handlers();

        event_loop.set_on_loop_started  (enclose! ((stats_monitor) move || {
            stats_monitor.begin();
        }));
        event_loop.set_on_loop_finished (enclose! ((stats_monitor) move || {
            stats_monitor.end();
        }));
        Self {scene,scene_dirty,logger,event_loop,performance,start_time,time,display_mode
             ,update_handle,stats,stats_monitor,focus_manager}
    }

    fn init_composer(&self) {
        let mouse_hover_ids     = self.scene.mouse.hover_ids.clone_ref();
        let mut pixel_read_pass = PixelReadPass::<u32>::new(&self.scene.mouse.position);
        pixel_read_pass.set_callback(move |v| {
            mouse_hover_ids.set(Vector4::from_iterator(v))
        });
        // TODO: We may want to enable it on weak hardware.
        // pixel_read_pass.set_threshold(1);
        let pipeline = RenderPipeline::new()
            .add(SymbolsRenderPass::new(&self.scene.symbols(),&self.scene.views))
            .add(ScreenRenderPass::new(self))
            .add(pixel_read_pass)
            .add(SymbolsRenderPass2::new(&self.scene.symbols(),&self.scene.views)); // FIXME ugly way of rendering top layers

        self.scene.renderer.set_pipeline(pipeline);
    }

    /// Dispose the world object, cancel all handlers and events.
    pub fn dispose(&mut self) {
        *self.update_handle.borrow_mut() = None;
    }
}

impl<'t> From<&'t World> for &'t display::object::Node {
    fn from(world:&'t World) -> Self {
        world.scene.display_object()
    }
}
