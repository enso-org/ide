#![allow(missing_docs)]

#[warn(missing_docs)]
pub mod dom;

pub use crate::display::symbol::registry::SymbolId;

use crate::prelude::*;
use crate::display::traits::*;

use crate::closure;
use crate::control::callback::CallbackHandle;
use crate::control::callback::DynEvent;
use crate::control::io::mouse::MouseFrpCallbackHandles;
use crate::control::io::mouse::MouseManager;
use crate::control::io::mouse;
use crate::data::dirty::traits::*;
use crate::data::dirty;
use crate::debug::stats::Stats;
use crate::display::camera::Camera2d;
use crate::display::render::RenderComposer;
use crate::display::render::RenderPipeline;
use crate::display::symbol::registry::SymbolRegistry;
use crate::display::symbol::Symbol;
use crate::display;
use crate::system::gpu::data::uniform::Uniform;
use crate::system::gpu::data::uniform::UniformScope;
use crate::system::gpu::shader::Context;
use crate::system::gpu::types::*;
use crate::display::scene::dom::DomScene;
use crate::system::web::NodeInserter;
use crate::system::web::resize_observer::ResizeObserver;
use crate::system::web::StyleSetter;
use crate::system::web;
use crate::display::shape::primitive::system::ShapeSystem;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;

use enso_frp;
use enso_frp::core::node::class::EventEmitterPoly;


// =====================
// === ShapeRegistry ===
// =====================

use std::any::TypeId;

shared! { ShapeRegistry
#[derive(Debug,Default)]
pub struct ShapeRegistryData {
    shape_system_map : HashMap<TypeId,ShapeSystem>
}

impl {
    pub fn get(&self, id:&TypeId) -> Option<ShapeSystem> {
        self.shape_system_map.get(id).map(|s| s.clone())
    }

    pub fn insert(&mut self, id:TypeId, shape_system:ShapeSystem) {
        self.shape_system_map.insert(id,shape_system);
    }
}}



// ======================
// === Mouse Handling ===
// ======================

pub trait MouseEventFn      = Fn(JsValue) + 'static;
pub type  MouseEventClosure = Closure<dyn Fn(JsValue)>;

fn mouse_event_closure<F:MouseEventFn>(f:F) -> MouseEventClosure {
    Closure::wrap(Box::new(f))
}

#[derive(Clone,Debug)]
struct Mouse {
    mouse_manager   : MouseManager,
    position        : Uniform<Vector2<i32>>,
    hover_ids       : Uniform<Vector4<u32>>,
    button0_pressed : Uniform<bool>,
    button1_pressed : Uniform<bool>,
    button2_pressed : Uniform<bool>,
    button3_pressed : Uniform<bool>,
    button4_pressed : Uniform<bool>,
    last_hover_ids  : Rc<Cell<Vector4<u32>>>,
    handles         : Rc<Vec<CallbackHandle>>,
}

impl CloneRef for Mouse {
    fn clone_ref(&self) -> Self {
        let mouse_manager   = self.mouse_manager.clone_ref();
        let position        = self.position.clone_ref();
        let hover_ids       = self.hover_ids.clone_ref();
        let button0_pressed = self.button0_pressed.clone_ref();
        let button1_pressed = self.button1_pressed.clone_ref();
        let button2_pressed = self.button2_pressed.clone_ref();
        let button3_pressed = self.button3_pressed.clone_ref();
        let button4_pressed = self.button4_pressed.clone_ref();
        let last_hover_ids  = self.last_hover_ids.clone_ref();
        let handles         = self.handles.clone_ref();
        Self {mouse_manager,position,hover_ids,button0_pressed,button1_pressed,button2_pressed
             ,button3_pressed,button4_pressed,last_hover_ids,handles}
    }
}

impl Mouse {
    pub fn new(shape:&web::dom::Shape, variables:&UniformScope) -> Self {

        let empty_hover_ids = Vector4::<u32>::new(0,0,0,0);
        let position        = variables.add_or_panic("mouse_position",Vector2::new(0,0));
        let hover_ids       = variables.add_or_panic("mouse_hover_ids",empty_hover_ids);
        let button0_pressed = variables.add_or_panic("mouse_button0_pressed",false);
        let button1_pressed = variables.add_or_panic("mouse_button1_pressed",false);
        let button2_pressed = variables.add_or_panic("mouse_button2_pressed",false);
        let button3_pressed = variables.add_or_panic("mouse_button3_pressed",false);
        let button4_pressed = variables.add_or_panic("mouse_button4_pressed",false);
        let last_hover_ids  = Rc::new(Cell::new(empty_hover_ids));
        let document        = web::dom::WithKnownShape::new(&web::document().body().unwrap());
        let mouse_manager   = MouseManager::new(&document.into());

        let shape_ref       = shape.clone_ref();
        let position_ref    = position.clone_ref();
        let on_move_handle  = mouse_manager.on_move.add(move |event:&mouse::event::OnMove| {
            let pixel_ratio = shape_ref.pixel_ratio() as i32;
            let screen_x    = event.offset_x();
            let screen_y    = shape_ref.current().height() as i32 - event.offset_y();
            let canvas_x    = pixel_ratio * screen_x;
            let canvas_y    = pixel_ratio * screen_y;
            position_ref.set(Vector2::new(canvas_x,canvas_y))
        });

        let button0_pressed_ref = button0_pressed.clone_ref();
        let button1_pressed_ref = button1_pressed.clone_ref();
        let button2_pressed_ref = button2_pressed.clone_ref();
        let button3_pressed_ref = button3_pressed.clone_ref();
        let button4_pressed_ref = button4_pressed.clone_ref();
        let on_down_handle      = mouse_manager.on_down.add(move |event:&mouse::event::OnDown| {
            match event.button() {
                mouse::Button0 => button0_pressed_ref.set(true),
                mouse::Button1 => button1_pressed_ref.set(true),
                mouse::Button2 => button2_pressed_ref.set(true),
                mouse::Button3 => button3_pressed_ref.set(true),
                mouse::Button4 => button4_pressed_ref.set(true),
            }
        });

        let button0_pressed_ref = button0_pressed.clone_ref();
        let button1_pressed_ref = button1_pressed.clone_ref();
        let button2_pressed_ref = button2_pressed.clone_ref();
        let button3_pressed_ref = button3_pressed.clone_ref();
        let button4_pressed_ref = button4_pressed.clone_ref();
        let on_up_handle        = mouse_manager.on_up.add(move |event:&mouse::event::OnUp| {
            match event.button() {
                mouse::Button0 => button0_pressed_ref.set(false),
                mouse::Button1 => button1_pressed_ref.set(false),
                mouse::Button2 => button2_pressed_ref.set(false),
                mouse::Button3 => button3_pressed_ref.set(false),
                mouse::Button4 => button4_pressed_ref.set(false),
            }
        });

        let handles = Rc::new(vec![on_move_handle,on_down_handle,on_up_handle]);

        Self {mouse_manager,position,hover_ids,button0_pressed,button1_pressed,button2_pressed,button3_pressed
             ,button4_pressed,last_hover_ids,handles}
    }
}



// ===========
// === Dom ===
// ===========

/// DOM element manager
#[derive(Clone,Debug)]
pub struct Dom {
    /// Root DOM element of the scene.
    pub root : web::dom::WithKnownShape<web::HtmlDivElement>,
    /// Layers of the scene.
    pub layers : Layers,
}

impl CloneRef for Dom {}

impl Dom {
    /// Constructor.
    pub fn new(logger:&Logger) -> Self {
        let root   = web::create_div();
        let layers = Layers::new(&logger,&root);
        root.set_class_name("scene");
        root.set_style_or_panic("height"  , "100vh");
        root.set_style_or_panic("width"   , "100vw");
        root.set_style_or_panic("display" , "block");
        let root = web::dom::WithKnownShape::new(&root);
        Self {root,layers}
    }

    pub fn shape(&self) -> &web::dom::Shape {
        self.root.shape()
    }

    pub fn recompute_shape_with_reflow(&self) {
        self.shape().set_from_element_with_reflow(&self.root);
    }
}



// ==============
// === Layers ===
// ==============

/// DOM Layers of the scene. It contains a 2 CSS 3D layers and a canvas layer in the middle. The
/// CSS layers are used to manage DOM elements and to simulate depth-sorting of DOM and canvas
/// elements.
#[derive(Clone,Debug)]
pub struct Layers {
    /// Front DOM scene layer.
    pub dom_front : DomScene,
    /// The WebGL scene layer.
    pub canvas : web_sys::HtmlCanvasElement,
    /// Back DOM scene layer.
    pub dom_back : DomScene,
}

impl Layers {
    /// Constructor.
    pub fn new(logger:&Logger, dom:&web_sys::HtmlDivElement) -> Self {
        let canvas    = web::create_canvas();
        let dom_front = DomScene::new(&logger);
        let dom_back  = DomScene::new(&logger);
        canvas.set_style_or_panic("height"  , "100vh");
        canvas.set_style_or_panic("width"   , "100vw");
        canvas.set_style_or_panic("display" , "block");
        dom_front.dom.set_class_name("front");
        dom_back.dom.set_class_name("back");
        dom.append_or_panic(&dom_front.dom);
        dom.append_or_panic(&canvas);
        dom.append_or_panic(&dom_back.dom);
        dom_back.set_z_index(-1);
        Self {dom_front,canvas,dom_back}
    }
}



// ============
// === View ===
// ============

// === Definition ===

#[derive(Debug,Clone)]
pub struct View {
    data : Rc<ViewData>
}

#[derive(Debug,Clone)]
pub struct WeakView {
    data : Weak<ViewData>
}

#[derive(Debug,Clone)]
pub struct ViewData {
    logger  : Logger,
    camera  : Camera2d,
    symbols : RefCell<Vec<SymbolId>>,
}

impl CloneRef for View {}
impl CloneRef for WeakView {}


// === API ===

impl View {
    pub fn new(logger:&Logger, width:f32, height:f32) -> Self {
        let data = ViewData::new(logger,width,height);
        let data = Rc::new(data);
        Self {data}
    }

    pub fn downgrade(&self) -> WeakView {
        let data = Rc::downgrade(&self.data);
        WeakView {data}
    }
}

impl WeakView {
    pub fn upgrade(&self) -> Option<View> {
        self.data.upgrade().map(|data| View{data})
    }
}

impl ViewData {
    pub fn new(logger:&Logger, width:f32, height:f32) -> Self {
        let logger  = logger.sub("view");
        let camera  = Camera2d::new(logger.sub("camera"),width,height);
        let symbols = default();
        Self {logger,camera,symbols}
    }
}



// =============
// === Views ===
// =============

pub struct Views {
    logger : Logger,
    main   : View,
    other  : Vec<View>,
}

impl Views {
    pub fn new(logger:&Logger, width:f32, height:f32) -> Self {
        let logger = logger.sub("views");
        let main   = View::new(&logger,width,height);
        let other  = default();
        Self {logger,main,other}
    }
}



// =============
// === Scene ===
// =============

#[derive(Clone,Debug)]
pub struct Scene {
    display_object   : display::object::Node,
    dom              : Dom,
    context          : Context,
    symbols          : SymbolRegistry,
    symbols_dirty    : SymbolRegistryDirty,
    camera           : Camera2d,
    shape_dirty      : ShapeDirty,
    logger           : Logger,
    variables        : UniformScope,
    pipeline         : Rc<CloneCell<RenderPipeline>>,
    composer         : Rc<CloneCell<RenderComposer>>,
    stats            : Stats,
    pixel_ratio      : Uniform<f32>,
    zoom_uniform     : Uniform<f32>,
    zoom_callback    : CallbackHandle,
    mouse            : Mouse,
    on_resize        : CallbackHandle,
    shape_registry   : ShapeRegistry,
    frp_mouse        : Rc<enso_frp::Mouse>,
}

impl CloneRef for Scene {
    fn clone_ref(&self) -> Self {
        let display_object   = self.display_object.clone_ref();
        let dom              = self.dom.clone_ref();
        let context          = self.context.clone_ref();
        let symbols          = self.symbols.clone_ref();
        let symbols_dirty    = self.symbols_dirty.clone_ref();
        let camera           = self.camera.clone_ref();
        let shape_dirty      = self.shape_dirty.clone_ref();
        let logger           = self.logger.clone_ref();
        let variables        = self.variables.clone_ref();
        let pipeline         = self.pipeline.clone_ref();
        let composer         = self.composer.clone_ref();
        let stats            = self.stats.clone_ref();
        let pixel_ratio      = self.pixel_ratio.clone_ref();
        let zoom_uniform     = self.zoom_uniform.clone_ref();
        let zoom_callback    = self.zoom_callback.clone_ref();
        let mouse            = self.mouse.clone_ref();
        let on_resize        = self.on_resize.clone_ref();
        let shape_registry   = self.shape_registry.clone_ref();
        let frp_mouse        = self.frp_mouse.clone_ref();
        Self {display_object,dom,context,symbols,symbols_dirty,camera,shape_dirty,logger,variables
             ,pipeline,composer,stats,pixel_ratio,zoom_uniform,zoom_callback,mouse,on_resize
             ,shape_registry,frp_mouse}
    }
}

impl Scene {
    /// Create new instance with the provided on-dirty callback.
    pub fn new<OnMut:Fn()+Clone+'static>
    (parent_dom:&HtmlElement, logger:Logger, stats:&Stats, on_mut:OnMut) -> Self {
        logger.trace("Initializing.");

        let dom = Dom::new(&logger);
        parent_dom.append_child(&dom.root).unwrap();
        dom.recompute_shape_with_reflow();

        let display_object  = display::object::Node::new(&logger);
        let context         = web::get_webgl2_context(&dom.layers.canvas);
        let sub_logger      = logger.sub("shape_dirty");
        let shape_dirty     = ShapeDirty::new(sub_logger,Box::new(on_mut.clone()));
        let sub_logger      = logger.sub("symbols_dirty");
        let dirty_flag      = SymbolRegistryDirty::new(sub_logger,Box::new(on_mut));
        let on_change       = symbols_on_change(dirty_flag.clone_ref());
        let sub_logger      = logger.sub("symbols");
        let variables       = UniformScope::new(logger.sub("global_variables"),&context);
        let symbols         = SymbolRegistry::new(&variables,&stats,&context,sub_logger,on_change);
        let screen_shape    = dom.shape().current();
        let width           = screen_shape.width();
        let height          = screen_shape.height();
        let symbols_dirty   = dirty_flag;
        let camera          = Camera2d::new(logger.sub("camera"),width,height);
        let zoom_uniform    = variables.add_or_panic("zoom", 1.0);
        let stats           = stats.clone();
        let pixel_ratio     = variables.add_or_panic("pixel_ratio", dom.shape().pixel_ratio());
        let mouse           = Mouse::new(&dom.shape(),&variables);
        let shape_registry  = default();
        let zoom_callback   = camera.add_zoom_update_callback(
            enclose!((zoom_uniform) move |zoom:&f32| zoom_uniform.set(*zoom))
        );

        let on_resize = dom.root.on_resize(enclose!((shape_dirty) move |_:&web::dom::ShapeData| {
            shape_dirty.set();
        }));

        context.enable(Context::BLEND);
        // To learn more about the blending equations used here, please see the following articles:
        // - http://www.realtimerendering.com/blog/gpus-prefer-premultiplication
        // - https://www.khronos.org/opengl/wiki/Blending#Colors
        context.blend_equation_separate ( Context::FUNC_ADD, Context::FUNC_ADD );
        context.blend_func_separate     ( Context::ONE , Context::ONE_MINUS_SRC_ALPHA
                                        , Context::ONE , Context::ONE_MINUS_SRC_ALPHA );

        let pipeline = default();
        let width    = dom.shape().current().device_pixels().width()  as i32;
        let height   = dom.shape().current().device_pixels().height() as i32;
        let composer = RenderComposer::new(&pipeline,&context,&variables,width,height);
        let composer = Rc::new(CloneCell::new(composer));
        let pipeline = Rc::new(CloneCell::new(pipeline));


        let mouse_manager   = &mouse.mouse_manager;
        let frp_mouse       = enso_frp::Mouse::new();

        enso_frp::frp! {
            mouse_down_position    = frp_mouse.position.sample   (&frp_mouse.on_down);
            mouse_position_if_down = frp_mouse.position.gate     (&frp_mouse.is_down);
            final_position_ref     = recursive::<enso_frp::Position>       ();
            pos_diff_on_down       = mouse_down_position.map2    (&final_position_ref,|m,f|{m-f});
            final_position         = mouse_position_if_down.map2 (&pos_diff_on_down  ,|m,f|{m-f});
            debug                  = final_position.sample       (&frp_mouse.position);
        }
        final_position_ref.initialize(&final_position);

        // final_position.event.display_graphviz();

    //    trace("X" , &debug.event);

    //    final_position.map("foo",move|p| {callback(p.x as f32,-p.y as f32)});

        let target = frp_mouse.position.event.clone_ref();
        let handle = mouse_manager.on_move.add(move |event:&mouse::OnMove| {
            target.emit(enso_frp::Position::new(event.client_x(),event.client_y()));
        });
        handle.forget();

        let target = frp_mouse.on_down.event.clone_ref();
        let handle = mouse_manager.on_down.add(move |event:&mouse::OnDown| {
            target.emit(());
        });
        handle.forget();

        let target = frp_mouse.on_up.event.clone_ref();
        let handle = mouse_manager.on_up.add(move |event:&mouse::OnUp| {
            target.emit(());
        });
        handle.forget();

        let frp_mouse = Rc::new(frp_mouse);


        Self { pipeline,composer,display_object,dom,context,symbols,camera,symbols_dirty,shape_dirty
             , logger,variables,stats,pixel_ratio,mouse,zoom_uniform
             , zoom_callback,on_resize,shape_registry,frp_mouse }
    }

    pub fn mouse(&self) -> Rc<enso_frp::Mouse> {
        self.frp_mouse.clone()
    }

    pub fn display_object(&self) -> display::object::Node {
        self.display_object.clone()
    }

    pub fn dom(&self) -> Dom {
        self.dom.clone()
    }

    pub fn lookup_shape(&self, id:&TypeId) -> Option<ShapeSystem> {
        self.shape_registry.get(id)
    }

    pub fn register_shape(&self, id:TypeId, shape_system:ShapeSystem) {
        self.shape_registry.insert(id,shape_system)
    }

    pub fn symbol_registry(&self) -> SymbolRegistry {
        self.symbols.clone_ref()
    }

    pub fn dom_front_layer(&self) -> DomScene {
        self.dom.layers.dom_front.clone_ref()
    }

    pub fn dom_back_layer(&self) -> DomScene {
        self.dom.layers.dom_back.clone_ref()
    }

    pub fn canvas(&self) -> web_sys::HtmlCanvasElement {
        self.dom.layers.canvas.clone()
    }

    pub fn context(&self) -> Context {
        self.context.clone()
    }

    pub fn variables(&self) -> UniformScope {
        self.variables.clone_ref()
    }

    pub fn mouse_position_uniform(&self) -> Uniform<Vector2<i32>> {
        self.mouse.position.clone_ref()
    }

    pub fn mouse_hover_ids(&self) -> Uniform<Vector4<u32>> {
        self.mouse.hover_ids.clone_ref()
    }

    pub fn set_render_pipeline<P:Into<RenderPipeline>>(&self, pipeline:P) {
        self.pipeline.set(pipeline.into());
        self.init_composer();
    }

    pub fn init_composer(&self) {
        let width  = self.dom.shape().current().device_pixels().width()  as i32;
        let height = self.dom.shape().current().device_pixels().height() as i32;
        self.composer.set(RenderComposer::new(&self.pipeline.get(),&self.context,&self.variables,width,height));
    }

    /// Bind FRP graph to mouse js events.
    pub fn bind_frp_to_mouse_events(&self, frp:&enso_frp::Mouse) -> MouseFrpCallbackHandles {
        mouse::bind_frp_to_mouse(&self.dom.shape(),frp,&self.mouse.mouse_manager)
    }

    /// Check dirty flags and update the state accordingly.
    pub fn update_and_render(&self) {
        self.update();
        self.render();
    }

    pub fn camera(&self) -> Camera2d {
        self.camera.clone_ref()
    }

    pub fn stats(&self) -> Stats {
        self.stats.clone_ref()
    }

    pub fn index(&self, ix:usize) -> Symbol {
        self.symbols.index(ix)
    }

    /// Create a new `Symbol` instance.
    pub fn new_symbol(&self) -> Symbol {
        self.symbols.new_symbol()
    }
}


// === Render & Update ===

impl Scene {
    pub fn render(&self) {
        group!(self.logger, "Rendering.", {
            self.composer.get().run();
        })
    }

    fn handle_mouse_events(&self) {
        let mouse_hover_ids = self.mouse.hover_ids.get();
        if mouse_hover_ids != self.mouse.last_hover_ids.get() {
            self.mouse.last_hover_ids.set(mouse_hover_ids);
            let is_not_background = mouse_hover_ids.w != 0;
            if is_not_background {
                let symbol_id = mouse_hover_ids.x;
                let symbol    = self.symbols.index(symbol_id as usize);
                symbol.dispatch_event(&DynEvent::new(()));
                // println!("{:?}",self.mouse.hover_ids.get());
                // TODO: finish events sending, including OnOver and OnOut.
            }
        }
    }

    fn update_shape(&self) {
        if self.shape_dirty.check_all() {
            let screen = self.dom.shape().current();
            self.resize_canvas(&self.dom.shape());
            self.camera.set_screen(screen.width(), screen.height());
            self.init_composer();
            self.shape_dirty.unset_all();
        }
    }

    fn update_symbols(&self) {
        if self.symbols_dirty.check_all() {
            self.symbols.update();
            self.symbols_dirty.unset_all();
        }
    }

    fn update_camera(&self) {
        let camera_changed = self.camera.update();
        if camera_changed {
            self.symbols.update_view_projection(&self.camera);
            self.dom.layers.dom_front.update_view_projection(&self.camera);
            self.dom.layers.dom_back.update_view_projection(&self.camera);
        }
    }
}

impl Scene {
    pub fn update(&self) {
//        group!(self.logger, "Updating.", {
            self.display_object.update_with(self);
            self.update_shape();
            self.update_symbols();
            self.update_camera();
            self.handle_mouse_events();
//        })
    }
}


// === Types ===

pub type ShapeDirty          = dirty::SharedBool<Box<dyn Fn()>>;
pub type SymbolRegistryDirty = dirty::SharedBool<Box<dyn Fn()>>;


// === Callbacks ===

closure! {
fn symbols_on_change(dirty:SymbolRegistryDirty) -> OnSymbolRegistryChange {
    || dirty.set()
}}


impl Scene {
    /// Resize the underlying canvas. This function should rather not be called
    /// directly. If you want to change the canvas size, modify the `shape` and
    /// set the dirty flag.
    fn resize_canvas(&self, shape:&web::dom::Shape) {
        let screen = shape.current();
        let canvas = shape.current().device_pixels();
        self.logger.group(fmt!("Resized to {}px x {}px.", screen.width(), screen.height()), || {
            self.dom.layers.canvas.set_attribute("width",  &canvas.width().to_string()).unwrap();
            self.dom.layers.canvas.set_attribute("height", &canvas.height().to_string()).unwrap();
            self.context.viewport(0,0,canvas.width() as i32, canvas.height() as i32);
        });
    }
}
