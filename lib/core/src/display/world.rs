#![allow(missing_docs)]

#[warn(missing_docs)]
pub mod event_loop;
#[warn(missing_docs)]
pub mod scene;
#[warn(missing_docs)]
pub mod workspace;

pub use crate::display::symbol::types::*;
pub use crate::display::world::workspace::Workspace;
pub use crate::data::container::*;
pub use crate::display::world::workspace::SymbolId;

use crate::prelude::*;

use crate::closure;
use crate::control::callback::CallbackHandle;
use crate::data::dirty::traits::*;
use crate::data::dirty;
use crate::debug::monitor::Monitor;
use crate::debug::monitor::Panel;
use crate::debug::monitor;
use crate::debug::stats::Stats;
use crate::display::shape::text::font::Fonts;
use crate::display::object::*;
use crate::system::web;
use crate::display::symbol::Symbol;

use event_loop::EventLoop;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::Performance;
use web_sys::KeyboardEvent;

use crate::system::gpu::types::*;
use crate::system::gpu::data::texture;
use crate::system::gpu::data::texture::Texture;



static mut WORLD: Option<World> = None;


pub fn get_world() -> World {
    unsafe {
        WORLD.as_ref().unwrap().clone_ref()
    }
}




// ====================
// === StatsMonitor ===
// ====================

#[derive(Clone,Debug)]
pub struct StatsMonitor {
    rc: Rc<RefCell<StatsMonitorData>>
}

impl StatsMonitor {
    pub fn new(stats:&Stats) -> Self {
        let rc = Rc::new(RefCell::new(StatsMonitorData::new(stats)));
        Self {rc}
    }

    pub fn begin(&self) {
        self.rc.borrow_mut().begin()
    }

    pub fn end(&self) {
        self.rc.borrow_mut().end()
    }
}


#[derive(Debug)]
pub struct StatsMonitorData {
    stats   : Stats,
    monitor : Monitor,
    panels  : Vec<Panel>
}

impl StatsMonitorData {
    fn new(stats:&Stats) -> Self {
        let stats       = stats.clone_ref();
        let mut monitor = Monitor::new();
        let panels = vec![
            monitor.add( monitor::FrameTime          :: new()       ),
            monitor.add( monitor::Fps                :: new()       ),
            monitor.add( monitor::WasmMemory         :: new()       ),
            monitor.add( monitor::GpuMemoryUsage     :: new(&stats) ),
            monitor.add( monitor::DrawCallCount      :: new(&stats) ),
            monitor.add( monitor::DataUploadCount    :: new(&stats) ),
            monitor.add( monitor::DataUploadSize     :: new(&stats) ),
            monitor.add( monitor::BufferCount        :: new(&stats) ),
            monitor.add( monitor::SymbolCount        :: new(&stats) ),
            monitor.add( monitor::ShaderCount        :: new(&stats) ),
            monitor.add( monitor::ShaderCompileCount :: new(&stats) ),
            monitor.add( monitor::SpriteSystemCount  :: new(&stats) ),
            monitor.add( monitor::SpriteCount        :: new(&stats) ),
        ];
        Self {stats,monitor,panels}
    }

    fn begin(&mut self) {
        for panel in &self.panels {
            panel.begin();
        }
    }

    fn end(&mut self) {
        for panel in &self.panels {
            panel.end();
        }
        self.monitor.draw();
        self.stats.reset_per_frame_statistics();
    }
}




#[derive(Debug)]
pub struct RenderComposer {
    passes    : Vec<RenderPassRunner>,
    variables : UniformScope,
    context   : Context,
    width     : i32,
    height    : i32,
}

impl RenderComposer {
    pub fn new(context:&Context, variables:&UniformScope, width:i32, height:i32) -> Self {
        let passes    = default();
        let context   = context.clone();
        let variables = variables.clone_ref();
        Self {passes,variables,context,width,height}
    }

    pub fn add<Pass:RenderPass>(&mut self, pass:Pass) {
        let pass = RenderPassRunner::new(&self.context,&self.variables,pass,self.width,self.height);
        self.passes.push(pass);
    }

    pub fn run(&self) {
        for pass in &self.passes {
            pass.run(&self.context);
        }
    }
}






#[derive(Derivative)]
#[derivative(Debug)]
struct RenderPassRunner {
    #[derivative(Debug="ignore")]
    pass        : Box<dyn RenderPass>,
    outputs     : Vec<AnyTextureUniform>,
    framebuffer : Option<web_sys::WebGlFramebuffer>,
    variables   : UniformScope,
    context     : Context,
    width       : i32,
    height      : i32,
}

impl RenderPassRunner {
    pub fn new<Pass:RenderPass>(context:&Context, variables:&UniformScope, pass:Pass, width:i32, height:i32) -> Self {
        let pass        = Box::new(pass);
        let outputs     = default();
        let framebuffer = if pass.outputs().is_empty() {None} else {Some(context.create_framebuffer().unwrap())};
        let variables   = variables.clone_ref();
        let context     = context.clone();
        let mut this    = Self {pass,outputs,framebuffer,variables,context,width,height};
        this.initialize();
        this
    }

    pub fn run(&self, context:&Context) {
        self.context.bind_framebuffer(Context::FRAMEBUFFER,self.framebuffer.as_ref());
        self.pass.run(context);
    }

    fn initialize(&mut self) {
        for output in &self.pass.outputs() {
            let texture = Texture::<texture::GpuOnly,texture::Rgba,u8>::new(&self.context,(self.width,self.height));
            let uniform = self.variables.add_or_panic(&format!("pass_{}",output.name),texture);
            self.add_output(uniform.into());
        }
    }

    fn add_output(&mut self, texture:AnyTextureUniform) {
        let context          = &self.context;
        let target           = Context::FRAMEBUFFER;
        let texture_target   = Context::TEXTURE_2D;
        let index            = self.outputs.len() as u32;
        let attachment_point = Context::COLOR_ATTACHMENT0 + index;
        let gl_texture       = texture.gl_texture();
        let gl_texture       = Some(&gl_texture);
        let level            = 0;
        self.outputs.push(texture);
        context.bind_framebuffer(target,self.framebuffer.as_ref());
        context.framebuffer_texture_2d(target,attachment_point,texture_target,gl_texture,level);
    }
}







pub struct RenderPassOutput {
    name            : String,
    internal_format : texture::AnyInternalFormat,
    // type like u8
}

impl RenderPassOutput {
    pub fn new<Name:Str>(name:Name, internal_format:texture::AnyInternalFormat) -> Self {
        let name = name.into();
        Self {name,internal_format}
    }
}


pub trait RenderPass : 'static {
    fn run(&self, context:&Context);
    fn outputs(&self) -> Vec<RenderPassOutput> {
        default()
    }
}




#[derive(Debug)]
struct WorldRenderPass {
    target: DisplayObjectData
}

impl WorldRenderPass {
    pub fn new(target:&DisplayObjectData) -> Self {
        let target = target.clone_ref();
        Self {target}
    }
}

impl RenderPass for WorldRenderPass {
    fn run(&self, context:&Context) {
        context.clear_color(0.0, 0.0, 0.0, 1.0);
        context.clear(Context::COLOR_BUFFER_BIT);
        self.target.render();
    }

    fn outputs(&self) -> Vec<RenderPassOutput> {
        vec![RenderPassOutput::new("color",texture::AnyInternalFormat::Rgba)]
    }
}




#[derive(Debug)]
struct ScreenRenderPass {
    screen: Screen,
}

impl ScreenRenderPass {
    pub fn new() -> Self {
        let screen = Screen::new();
        Self {screen}
    }
}

impl RenderPass for ScreenRenderPass {
    fn run(&self, _:&Context) {
        self.screen.render();
    }
}



fn mk_render_composer(workspace:&Workspace, dp:&DisplayObjectData, width:i32, height:i32) -> RenderComposer {
    let context   = &workspace.context();
    let variables = &workspace.variables();
//    let width     = workspace.shape.canvas_shape().width  as i32;
//    let height    = workspace.shape.canvas_shape().height as i32;
    let mut composer  = RenderComposer::new(context,variables,width,height);
    composer.add(WorldRenderPass::new(dp));
    composer.add(ScreenRenderPass::new());
    composer
}



// =================
// === WorldData ===
// =================

// === Definition ===

/// World is the top-level application structure. It used to manage several instances of
/// `Workspace`, and there is probability that we will get back to this design in the future.
/// It is responsible for updating the system on every animation frame.
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct WorldData {
    display_object : DisplayObjectData,
    pub workspace       : Workspace,
    pub workspace_dirty : WorkspaceDirty,
    pub logger          : Logger,
    pub event_loop      : EventLoop,
    pub performance     : Performance,
    pub start_time      : f32,
    pub time            : Uniform<f32>,
    pub display_mode    : Uniform<i32>,
    pub fonts           : Fonts,
    pub update_handle   : Option<CallbackHandle>,
    pub stats           : Stats,
    pub stats_monitor   : StatsMonitor,

    pub tmp_composer: Option<RenderComposer>,


}


// === Types ===

pub type WorkspaceID    = usize;
pub type WorkspaceDirty = dirty::SharedBool;


// === Callbacks ===

closure! {
fn workspace_on_change(dirty:WorkspaceDirty) -> WorkspaceOnChange {
    || dirty.set()
}}


// === Implementation ===

impl WorldData {
    /// Create and initialize new world instance.
    #[allow(clippy::new_ret_no_self)]
    pub fn new<Dom:Str>(dom:Dom) -> World {
        println!("NOTICE! When profiling in Chrome check 'Disable JavaScript Samples' under the \
                  gear icon in the 'Performance' tab. It can drastically slow the rendering.");
        let world          = World::new(Self::new_uninitialized(dom));
        let world_ref      = world.clone_ref();
        let display_object = world.rc.borrow().display_object.clone();
        with(world.rc.borrow_mut(), |mut data| {
            let update = move || {
                world_ref.rc.borrow_mut().run();
            };
            let update_handle   = data.event_loop.add_callback(update);
            data.update_handle  = Some(update_handle);
        });

        // -----------------------------------------------------------------------------------------
        // FIXME[WD]: Hacky way of switching display_mode. To be fixed and refactored out.
        let world_copy = world.clone();
        let c: Closure<dyn Fn(JsValue)> = Closure::wrap(Box::new(move |val| {
            let val = val.unchecked_into::<KeyboardEvent>();
            let key = val.key();
            if      key == "0" { world_copy.rc.borrow_mut().display_mode.set(0) }
            else if key == "1" { world_copy.rc.borrow_mut().display_mode.set(1) }
        }));
        web::document().unwrap().add_event_listener_with_callback
            ("keydown",c.as_ref().unchecked_ref()).unwrap();
        c.forget();
        // -----------------------------------------------------------------------------------------

        world
    }

    /// Create new uninitialized world instance. You should rather not need to
    /// call this function directly.
    fn new_uninitialized<Dom:Str>(dom:Dom) -> Self {
        let stats                  = default();
        let logger                 = Logger::new("world");
        let display_object         = DisplayObjectData::new(logger.clone());
        let workspace_logger       = logger.sub("workspace");
        let workspace_dirty_logger = logger.sub("workspace_dirty");
        let workspace_dirty        = WorkspaceDirty::new(workspace_dirty_logger,());
        let workspace_dirty2       = workspace_dirty.clone();
        let on_change              = move || {workspace_dirty2.set()};
        let workspace              = Workspace::new(dom,workspace_logger,&stats,on_change);
        let variables              = &workspace.variables();
        let time                   = variables.add_or_panic("time",0.0);
        let display_mode           = variables.add_or_panic("display_mode",0);
        let fonts                  = Fonts::new();
        let event_loop             = EventLoop::new();
        let update_handle          = default();
        let stats_monitor          = StatsMonitor::new(&stats);
        let performance            = web::get_performance().unwrap();
        let start_time             = performance.now() as f32;
        let stats_monitor_cp_1     = stats_monitor.clone();
        let stats_monitor_cp_2     = stats_monitor.clone();

        let tmp_composer = None;

        event_loop.set_on_loop_started  (move || { stats_monitor_cp_1.begin(); });
        event_loop.set_on_loop_finished (move || { stats_monitor_cp_2.end();   });
        Self {display_object,workspace,workspace_dirty,logger,event_loop,performance,start_time,time,display_mode
             ,fonts,update_handle,stats,stats_monitor,tmp_composer}
    }


    pub fn run(&mut self) {
        let relative_time = self.performance.now() as f32 - self.start_time;
        self.time.set(relative_time);
        self.update();
        self.tmp_composer.as_ref().unwrap().run();
    }

    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self) {
        //TODO[WD]: Re-think when should we check the condition (uniform update):
        //          if self.workspace_dirty.check_all() {
        group!(self.logger, "Updating.", {
            self.workspace_dirty.unset_all();
            let fonts = &mut self.fonts;
            self.workspace.update(fonts);
        });
    }

    /// Dispose the world object, cancel all handlers and events.
    pub fn dispose(&mut self) {
        self.update_handle = None;
    }
}

impl Into<DisplayObjectData> for &WorldData {
    fn into(self) -> DisplayObjectData {
        self.display_object.clone()
    }
}

impl Drop for WorldData {
    fn drop(&mut self) {
        self.logger.info("Dropping.");
    }
}



// =============
// === World ===
// =============

// === Definition ===

/// Shared reference to the `World` object.
#[derive(Clone,Debug)]
pub struct World {
    pub rc: Rc<RefCell<WorldData>>,
}

impl World {
    /// Create new shared reference.
    pub fn new(world: WorldData) -> Self {
        let rc = Rc::new(RefCell::new(world));
        let out = Self {rc};
        unsafe {
            WORLD = Some(out.clone_ref());
        }
        out.test();

        out
    }

    /// Cheap clone of the world reference.
    pub fn clone_ref(&self) -> Self {
        self.clone()
    }

    /// Dispose the world object, cancel all handlers and events.
    pub fn dispose(&self) {
        self.rc.borrow_mut().dispose()
    }

    pub fn stats(&self) -> Stats {
        self.rc.borrow().stats.clone_ref()
    }

    pub fn new_symbol(&self) -> Symbol {
        self.rc.borrow().workspace.new_symbol2()
    }

    /// Run the provided callback on every frame. Returns a `CallbackHandle`,
    /// which when dropped will cancel the callback. If you want the function
    /// to run forever, you can use the `forget` method in the handle.
    pub fn on_frame<F:FnMut(&World)+'static>
    (&self, mut callback:F) -> CallbackHandle {
        let this = self.clone_ref();
        let func = move || callback(&this);
        self.rc.borrow_mut().event_loop.add_callback(func)
    }

    pub fn mod_stats<F:FnOnce(&Stats)>(&self, f:F) {
        f(&self.rc.borrow().stats);
    }

    pub fn render(&self) {
        self.rc.borrow_mut().run();
    }

    fn test(&self) {
        let width  = 961*2; // shape.width as i32;
        let height = 359*2; // shape.height as i32;
        let context = self.rc.borrow().workspace.context();

        let texture1 = Texture::<texture::GpuOnly,texture::Rgba,u8>::new(&context,(width,height));

        let screen = Screen::new();

        let uniform:Uniform<Texture<texture::GpuOnly,texture::Rgba,u8>> = {
            screen.variables().add_or_panic("pass_color",texture1)
        };


        let fb = context.create_framebuffer().unwrap();

        let composer = {
            let dp = &self.rc.borrow().display_object;
            mk_render_composer(&self.rc.borrow().workspace, dp, width, height)
        };
        let world = self.clone_ref();
        self.rc.borrow_mut().tmp_composer = Some(composer);
    }
}

impl<T> Add<T> for World where WorldData: Add<T> {
    type Result = AddResult<WorldData,T>;
    fn add(&mut self, t:T) -> Self::Result {
        self.rc.borrow_mut().add(t)
    }
}

impl Into<DisplayObjectData> for &World {
    fn into(self) -> DisplayObjectData {
        let data:&WorldData = &self.rc.borrow();
        data.into()
    }
}
