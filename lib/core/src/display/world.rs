#![allow(missing_docs)]

use crate::prelude::*;

pub use crate::data::container::*;
pub use crate::display::symbol::types::*;
pub use crate::display::scene::SymbolId;

use crate::closure;
use crate::control::callback::CallbackHandle;
use crate::control::event_loop::EventLoop;
use crate::data::dirty::traits::*;
use crate::data::dirty;
use crate::debug::monitor::Monitor;
use crate::debug::monitor::Panel;
use crate::debug::monitor;
use crate::debug::stats::Stats;
use crate::display::object::*;
use crate::display::scene::Scene;
use crate::display::shape::text::font::Fonts;
use crate::display::symbol::Symbol;
use crate::system::gpu::data::texture::Texture;
use crate::system::gpu::data::texture;
use crate::system::gpu::types::*;
use crate::system::web;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::Performance;
use web_sys::KeyboardEvent;





static mut WORLD: Option<World> = None;


pub fn get_world() -> World {
    unsafe {
        WORLD.as_ref().unwrap_or_else(|| panic!("World not initialized.")).clone_ref()
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
    pub fn new(pipeline:&RenderPipeline, context:&Context, variables:&UniformScope, width:i32, height:i32) -> Self {
        let passes    = default();
        let context   = context.clone();
        let variables = variables.clone_ref();
        let mut this  = Self {passes,variables,context,width,height};
        for pass in &pipeline.passes { this.add(pass); };
        this
    }

    pub fn add(&mut self, pass:&Box<dyn RenderPass>) {
        let pass = RenderPassRunner::new(&self.context,&self.variables,pass,self.width,self.height);
        self.passes.push(pass);
    }

    pub fn run(&mut self) {
        for pass in &mut self.passes {
            pass.run(&self.context);
        }
    }
}




#[derive(Debug,Default)]
pub struct RenderPipeline {
    passes: Vec<Box<dyn RenderPass>>
}

impl RenderPipeline {
    pub fn new() -> Self {
        default()
    }

    pub fn add<Pass:RenderPass>(mut self, pass:Pass) -> Self {
        let pass = Box::new(pass);
        self.passes.push(pass);
        self
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
    pub fn new(context:&Context, variables:&UniformScope, pass:&Box<dyn RenderPass>, width:i32, height:i32) -> Self {
        let pass        = <Box<dyn RenderPass> as Clone>::clone(pass);
        let outputs     = default();
        let framebuffer = if pass.outputs().is_empty() {None} else {Some(context.create_framebuffer().unwrap())};
        let variables   = variables.clone_ref();
        let context     = context.clone();
        let mut this    = Self {pass,outputs,framebuffer,variables,context,width,height};
        this.initialize();
        this
    }

    pub fn run(&mut self, context:&Context) {
        self.context.bind_framebuffer(Context::FRAMEBUFFER,self.framebuffer.as_ref());
        self.pass.run(context,&self.variables);
    }

    fn initialize(&mut self) {
        for output in &self.pass.outputs() {
            let texture = Texture::<texture::GpuOnly,texture::Rgba,u8>::new(&self.context,(self.width,self.height));
            let uniform = self.variables.get_or_add(&format!("pass_{}",output.name),texture).unwrap();
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


pub trait RenderPass : CloneBoxedForRenderPass + Debug + 'static {
    fn run(&mut self, context:&Context, variables:&UniformScope);
    fn outputs(&self) -> Vec<RenderPassOutput> {
        default()
    }
}

clone_boxed!(RenderPass);




#[derive(Clone,Debug)]
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
    fn run(&mut self, context:&Context, _:&UniformScope) {
        context.clear_color(0.0, 0.0, 0.0, 1.0);
        context.clear(Context::COLOR_BUFFER_BIT);
        self.target.render();
    }

    fn outputs(&self) -> Vec<RenderPassOutput> {
        vec![RenderPassOutput::new("color",texture::AnyInternalFormat::Rgba)]
    }
}




#[derive(Clone,Debug)]
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
    fn run(&mut self, _:&Context, _:&UniformScope) {
        self.screen.render();
    }
}


#[derive(Clone,Debug)]
pub struct PixelReadPassData {
    uniform : Uniform<Vector4<i32>>,
    buffer  : WebGlBuffer,
}

impl PixelReadPassData {
    pub fn new(uniform:Uniform<Vector4<i32>>, buffer:WebGlBuffer) -> Self {
        Self {uniform,buffer}
    }
}


#[derive(Clone,Debug,Default)]
struct PixelReadPass {
    data: Option<PixelReadPassData>,
    sync: Option<WebGlSync>,
}

impl PixelReadPass {
    pub fn new() -> Self {
        default()
    }


    fn init_if_fresh(&mut self, context:&Context, variables:&UniformScope) {
        if self.data.is_none() {
            let buffer  = context.create_buffer().unwrap();
            let array   = ArrayBuffer::new(4);
            let target  = Context::PIXEL_PACK_BUFFER;
            let usage   = Context::DYNAMIC_READ;
            let uniform = variables.get_or_add("pass_pixel_color",Vector4::new(0,0,0,0)).unwrap();
            context.bind_buffer(target,Some(&buffer));
            context.buffer_data_with_opt_array_buffer(target,Some(&array),usage);
            self.data = Some(PixelReadPassData::new(uniform,buffer));
        }
    }

    fn run_not_synced(&mut self, context:&Context) {
        let data   = self.data.as_ref().unwrap();
        let mousex = 228*2;
        let mousey = 70*2;
        let width  = 1;
        let height = 1;
        let format = Context::RGBA;
        let typ    = Context::UNSIGNED_BYTE;
        let target = Context::PIXEL_PACK_BUFFER;
        let offset = 0;
        context.bind_buffer(target,Some(&data.buffer));
        context.read_pixels_with_i32(mousex,mousey,width,height,format,typ,offset).unwrap();
        let condition = Context::SYNC_GPU_COMMANDS_COMPLETE;
        let flags     = 0;
        let sync      = context.fence_sync(condition,flags).unwrap();
        self.sync     = Some(sync);
        context.flush();
    }

    fn check_and_handle_sync(&mut self, context:&Context, sync:&WebGlSync) {
        let data   = self.data.as_ref().unwrap();
        let status = context.get_sync_parameter(sync,Context::SYNC_STATUS);
        if status == Context::SIGNALED {
            context.delete_sync(Some(sync));
            self.sync      = None;
            let target     = Context::PIXEL_PACK_BUFFER;
            let offset     = 0;
            let mut result = vec![0,0,0,0];
            context.bind_buffer(target,Some(&data.buffer));
            context.get_buffer_sub_data_with_i32_and_u8_array(target,offset,&mut result);
            data.uniform.set(Vector4::from_iterator(result.iter().map(|t| *t as i32)));
            println!("GOT: {:?}", result);
        }
    }
}

impl RenderPass for PixelReadPass {
    fn run(&mut self, context:&Context, variables:&UniformScope) {
        self.init_if_fresh(context,variables);
        if let Some(sync) = self.sync.clone() {
            self.check_and_handle_sync(context,&sync);
        }
        if self.sync.is_none() {
            self.run_not_synced(context);
        }
    }
}

use js_sys::ArrayBuffer;
use web_sys::WebGlBuffer;
use web_sys::WebGlSync;

//pub fn buffer_data_with_opt_array_buffer(
//    &self,
//    target: u32,
//    src_data: Option<&ArrayBuffer>,
//    usage: u32
//)


fn default_render_pipeline(root:&DisplayObjectData) -> RenderPipeline {
    RenderPipeline::new()
        .add(WorldRenderPass::new(root))
        .add(ScreenRenderPass::new())
        .add(PixelReadPass::new())
}

fn mk_render_composer(scene:&Scene, dp:&DisplayObjectData, width:i32, height:i32) -> RenderComposer {
    let context   = &scene.context();
    let variables = &scene.variables();
//    let width     = scene.shape.canvas_shape().width  as i32;
//    let height    = scene.shape.canvas_shape().height as i32;

    let pipeline = default_render_pipeline(dp);

    RenderComposer::new(&pipeline,context,variables,width,height)
}








// =================
// === WorldData ===
// =================

// === Definition ===

/// World is the top-level application structure. It used to manage several instances of
/// `Scene`, and there is probability that we will get back to this design in the future.
/// It is responsible for updating the system on every animation frame.
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct WorldData {
    pub scene       : Scene,
    pub scene_dirty : SceneDirty,
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

impl WorldData {
    /// Create and initialize new world instance.
    #[allow(clippy::new_ret_no_self)]
    pub fn new<Dom:Str>(dom:Dom) -> World {
        println!("NOTICE! When profiling in Chrome check 'Disable JavaScript Samples' under the \
                  gear icon in the 'Performance' tab. It can drastically slow the rendering.");
        let world          = World::new(Self::new_uninitialized(dom));
        let world_ref      = world.clone_ref();
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
        let scene_logger       = logger.sub("scene");
        let scene_dirty_logger = logger.sub("scene_dirty");
        let scene_dirty        = SceneDirty::new(scene_dirty_logger,());
        let scene_dirty2       = scene_dirty.clone();
        let on_change              = move || {scene_dirty2.set()};
        let scene              = Scene::new(dom,scene_logger,&stats,on_change);
        let variables              = &scene.variables();
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

        event_loop.set_on_loop_started  (move || { stats_monitor_cp_1.begin(); });
        event_loop.set_on_loop_finished (move || { stats_monitor_cp_2.end();   });
        Self {scene,scene_dirty,logger,event_loop,performance,start_time,time,display_mode
             ,fonts,update_handle,stats,stats_monitor}
    }


    pub fn run(&mut self) {
        let relative_time = self.performance.now() as f32 - self.start_time;
        self.time.set(relative_time);
        self.update();
    }

    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self) {
        //TODO[WD]: Re-think when should we check the condition (uniform update):
        //          if self.scene_dirty.check_all() {
        group!(self.logger, "Updating.", {
            self.scene_dirty.unset_all();
            let fonts = &mut self.fonts;
            self.scene.update(fonts);
        });
    }

    /// Dispose the world object, cancel all handlers and events.
    pub fn dispose(&mut self) {
        self.update_handle = None;
    }
}

impl Into<DisplayObjectData> for &WorldData {
    fn into(self) -> DisplayObjectData {
        (&self.scene).into()
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
        out.init_composer();

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
        self.rc.borrow().scene.new_symbol2()
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

    fn init_composer(&self) {
        let dp = &self.display_object_description();
        let pipeline = default_render_pipeline(dp);
        self.rc.borrow_mut().scene.set_render_pipeline(pipeline);
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
