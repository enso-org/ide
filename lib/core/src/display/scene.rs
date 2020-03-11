#![allow(missing_docs)]

use crate::prelude::*;

pub use crate::display::symbol::registry::SymbolId;

use crate::closure;
use crate::control::callback::CallbackRegistry1;
use crate::control::callback::CallbackMut1Fn;
use crate::control::callback::CallbackHandle;
use crate::control::callback::DynEvent;
use crate::control::io::mouse::MouseFrpCallbackHandles;
use crate::control::io::mouse::MouseManager;
use crate::control::io::mouse;
use crate::data::dirty::traits::*;
use crate::data::dirty;
use crate::debug::stats::Stats;
use crate::display;
use crate::display::camera::Camera2d;
use crate::display::object::DisplayObjectOps;
use crate::display::render::RenderComposer;
use crate::display::render::RenderPipeline;
use crate::display::symbol::registry::SymbolRegistry;
use crate::display::symbol::Symbol;
use crate::system::gpu::data::uniform::Uniform;
use crate::system::gpu::data::uniform::UniformScope;
use crate::system::gpu::shader::Context;
use crate::system::gpu::types::*;
use crate::system::web::dom::html::Css3dRenderer;
use crate::system::web::dyn_into;
use crate::system::web::resize_observer::ResizeObserver;
use crate::system::web::StyleSetter;
use crate::system::web;
use crate::system::web::NodeInserter;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;



// =============
// === Error ===
// =============

#[derive(Debug, Fail, From)]
pub enum Error {
    #[fail(display = "Web Platform error: {}", error)]
    WebError { error: web::Error },
}



// =============
// === Shape ===
// =============

// === Shape ===

use shapely::shared;

shared! { Shape
/// Contains information about DOM element shape and provides utils for querying the size both in
/// DOM pixel units as well as device pixel units.
#[derive(Debug)]
##[derive(Clone,Copy)]
pub struct ShapeData {
    width       : f32,
    height      : f32,
    pixel_ratio : f32
}

impl {
    /// Constructor.
    pub fn new() -> Self {
        let width       = 0.0;
        let height      = 0.0;
        let pixel_ratio = web::device_pixel_ratio().unwrap_or(1.0) as f32;
        Self {width,height,pixel_ratio}
    }

    /// Getter.
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Getter.
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Getter.
    pub fn pixel_ratio(&self) -> f32 {
        self.pixel_ratio
    }

    /// Dimension setter in DOM pixel units. Use `device_pixels` to switch to device units.
    pub fn set(&mut self, width:f32, height:f32) {
        self.width  = width;
        self.height = height;
    }

    /// Sets the size to the size of the provided element. This operation is slow as it causes
    /// reflow.
    pub fn set_from_element_with_reflow(&mut self, element:&HtmlElement) {
        let bbox   = element.get_bounding_client_rect();
        let width  = bbox.width()  as f32;
        let height = bbox.height() as f32;
        self.set(width,height);
    }
}}

impl ShapeData {
    /// Switched to device pixel units. On low-dpi screens device pixels map 1:1 with DOM pixels.
    /// On high-dpi screens, a single device pixel is often mapped to 2 or 3 DOM pixels.
    pub fn device_pixels(&self) -> Self {
        let width  = self.width  * self.pixel_ratio;
        let height = self.height * self.pixel_ratio;
        Self {width,height,..*self}
    }
}

impl Shape {
    pub fn from_element_with_reflow(element:&HtmlElement) -> Self {
        let mut this = Self::default();
        this.set_from_element_with_reflow(element);
        this
    }

    pub fn current(&self) -> ShapeData {
        *self.rc.borrow()
    }
}

impl Default for Shape {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ShapeData {
    fn default() -> Self {
        Self::new()
    }
}



// ====================
// === SizedElement ===
// ====================

/// A wrapper for `HtmlElement` or anything which derefs to it. It tracks the element size without
/// causing browser reflow.
#[derive(Debug,Shrinkwrap)]
pub struct SizedElement<T=web_sys::HtmlElement> {
    #[shrinkwrap(main_field)]
    dom       : T,
    shape     : Shape,
    observer  : ResizeObserver,
    on_resize : Rc<RefCell<CallbackRegistry1<ShapeData>>>,
}

impl<T> SizedElement<T> {
    /// Constructor.
    pub fn new(dom:&T) -> Self
    where T : Clone + AsRef<JsValue> + Into<web_sys::HtmlElement> {
        let dom          = dom.clone();
        let html_element = dom.clone().into();
        let shape        = Shape::from_element_with_reflow(&html_element);
        let on_resize    = Rc::new(RefCell::new(CallbackRegistry1::default()));
        let callback     = Closure::new(enclose!((shape,on_resize) move |width, height| {
            shape.set(width as f32,height as f32);
            on_resize.borrow_mut().run_all(&shape.current())
        }));
        let observer = ResizeObserver::new(dom.as_ref(),callback);
        Self {dom,shape,observer,on_resize}
    }

    /// Attach a new callback which will fire whenever the object will be resized.
    pub fn on_resize<F:CallbackMut1Fn<ShapeData>>(&self, callback:F) -> CallbackHandle {
        self.on_resize.borrow_mut().add(callback)
    }

    /// Get the current shape of the object.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }
}




// ======================
// === Mouse Handling ===
// ======================

pub trait MouseEventFn      = Fn(JsValue) + 'static;
pub type  MouseEventClosure = Closure<dyn Fn(JsValue)>;

fn mouse_event_closure<F:MouseEventFn>(f:F) -> MouseEventClosure {
    Closure::wrap(Box::new(f))
}

#[derive(Debug)]
struct Mouse {
    mouse_manager   : MouseManager,
    position        : Uniform<Vector2<i32>>,
    hover_ids       : Uniform<Vector4<u32>>,
    button0_pressed : Uniform<bool>,
    button1_pressed : Uniform<bool>,
    button2_pressed : Uniform<bool>,
    button3_pressed : Uniform<bool>,
    button4_pressed : Uniform<bool>,
    last_hover_ids  : Vector4<u32>,
    handles         : Vec<CallbackHandle>,
}

impl Mouse {
    pub fn new(shape:&Shape, variables:&UniformScope) -> Self {

        let empty_hover_ids = Vector4::<u32>::new(0,0,0,0);
        let position        = variables.add_or_panic("mouse_position",Vector2::new(0,0));
        let hover_ids       = variables.add_or_panic("mouse_hover_ids",empty_hover_ids);
        let button0_pressed = variables.add_or_panic("mouse_button0_pressed",false);
        let button1_pressed = variables.add_or_panic("mouse_button1_pressed",false);
        let button2_pressed = variables.add_or_panic("mouse_button2_pressed",false);
        let button3_pressed = variables.add_or_panic("mouse_button3_pressed",false);
        let button4_pressed = variables.add_or_panic("mouse_button4_pressed",false);
        let last_hover_ids  = empty_hover_ids;
        let document        = web::document().unwrap();
        let mouse_manager   = MouseManager::new(&document);

        let shape_ref       = shape.clone_ref();
        let position_ref    = position.clone_ref();
        let on_move_handle  = mouse_manager.on_move.add(move |event:&mouse::event::OnMove| {
            let pixel_ratio = shape_ref.pixel_ratio() as i32;
            let screen_x    = event.offset_x();
            let screen_y    = shape_ref.current().height as i32 - event.offset_y();
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

        let handles = vec![on_move_handle,on_down_handle,on_up_handle];

        Self {mouse_manager,position,hover_ids,button0_pressed,button1_pressed,button2_pressed,button3_pressed
             ,button4_pressed,last_hover_ids,handles}
    }
}



// ===========
// === Dom ===
// ===========

/// DOM element manager
#[derive(Debug)]
pub struct Dom {
    /// Root DOM element of the scene.
    pub root : SizedElement<web_sys::HtmlDivElement>,
    /// Layers of the scene.
    pub layers : Layers,
}

impl Dom {
    /// Constructor.
    pub fn new(logger:&Logger) -> Self {
        let root   = web::create_div();
        let layers = Layers::new(&logger,&root);
        root.set_style_or_panic("height"  , "100vh");
        root.set_style_or_panic("width"   , "100vw");
        root.set_style_or_panic("display" , "block");
        let root = SizedElement::new(&root);
        Self {root,layers}
    }

    pub fn shape(&self) -> &Shape {
        self.root.shape()
    }
}



// ==============
// === Layers ===
// ==============

/// DOM Layers of the scene. It contains a 2 CSS 3D layers and a canvas layer in the middle. The
/// CSS layers are used to manage DOM elements and to simulate depth-sorting of DOM and canvas
/// elements.
#[derive(Debug)]
pub struct Layers {
    /// Front DOM scene layer.
    pub dom_front : Css3dRenderer,
    /// The WebGL scene layer.
    pub canvas : web_sys::HtmlCanvasElement,
    /// Back DOM scene layer.
    pub dom_back : Css3dRenderer,
}

impl Layers {
    /// Constructor.
    pub fn new(logger:&Logger, dom:&web_sys::HtmlDivElement) -> Self {
        let canvas    = web::create_canvas();
        let dom_front = Css3dRenderer::new(&logger);
        let dom_back  = Css3dRenderer::new(&logger);
        canvas.set_style_or_panic("height"  , "100vh");
        canvas.set_style_or_panic("width"   , "100vw");
        canvas.set_style_or_panic("display" , "block");
        dom.append_or_panic(&dom_front.dom);
        dom.append_or_panic(&canvas);
        dom.append_or_panic(&dom_back.dom);
        dom_back.set_z_index(-1);
        Self {dom_front,canvas,dom_back}
    }
}



// =============
// === Scene ===
// =============

shared! { Scene
#[derive(Derivative)]
#[derivative(Debug)]
pub struct SceneData {
    display_object : display::object::Node,
    dom            : Dom,
    context        : Context,
    symbols        : SymbolRegistry,
    symbols_dirty  : SymbolRegistryDirty,
    camera         : Camera2d,
//    shape          : Shape,
    shape_dirty    : ShapeDirty,
    logger         : Logger,
//    listeners      : Listeners,
    variables      : UniformScope,
    pipeline       : RenderPipeline,
    composer       : RenderComposer,
    stats          : Stats,
    pixel_ratio    : Uniform<f32>,
    zoom_uniform   : Uniform<f32>,
    zoom_callback  : CallbackHandle,
    mouse          : Mouse,
    on_resize      : CallbackHandle,
//    #[derivative(Debug="ignore")]
//    on_resize2: Option<Box<dyn Fn(&Shape)>>,
}

impl {
    /// Create new instance with the provided on-dirty callback.
    pub fn new<OnMut:Fn()+Clone+'static>
    (parent_dom:&HtmlElement, logger:Logger, stats:&Stats, on_mut:OnMut) -> Self {
        logger.trace("Initializing.");

        let dom = Dom::new(&logger);
        parent_dom.append_child(&dom.root).unwrap();

        let display_object  = display::object::Node::new(&logger);
        let context         = web::get_webgl2_context(&dom.layers.canvas).unwrap();
        let sub_logger      = logger.sub("shape_dirty");
        let shape_dirty     = ShapeDirty::new(sub_logger,Box::new(on_mut.clone()));
        let sub_logger      = logger.sub("symbols_dirty");
        let dirty_flag      = SymbolRegistryDirty::new(sub_logger,Box::new(on_mut));
        let on_change       = symbols_on_change(dirty_flag.clone_ref());
        let sub_logger      = logger.sub("symbols");
        let variables       = UniformScope::new(logger.sub("global_variables"),&context);
        let symbols         = SymbolRegistry::new(&variables,&stats,&context,sub_logger,on_change);

        println!("--- 1");
        dom.shape().set_from_element_with_reflow(&dom.root);
        println!("--- 2");

        println!("--- 3 {:?}",dom.shape() );

//        let shape           = Shape::from_element_with_reflow(&dom.root);
        let screen_shape    = dom.shape().current();
        let width           = screen_shape.width;
        let height          = screen_shape.height;
//        let listeners       = Self::init_listeners(&logger,&dom.layers.canvas,&shape,&shape_dirty);
        let symbols_dirty   = dirty_flag;
        let camera          = Camera2d::new(logger.sub("camera"),width,height);
        let zoom_uniform    = variables.add_or_panic("zoom", 1.0);
//        let on_resize2       = default();
        let stats           = stats.clone();
        let pixel_ratio     = variables.add_or_panic("pixel_ratio", dom.shape().pixel_ratio());
        let mouse           = Mouse::new(&dom.shape(),&variables);
        let zoom_uniform_cp = zoom_uniform.clone();
        let zoom_callback   = camera.add_zoom_update_callback(
            move |zoom:&f32| zoom_uniform_cp.set(*zoom)
        );

        println!("x1");
        let on_resize = dom.root.on_resize(enclose!((shape_dirty) move |_:&ShapeData| {
            println!("resized callback");
            shape_dirty.set();
        }));
        println!("x2");

        context.enable(Context::BLEND);
        // To learn more about the blending equations used here, please see the following articles:
        // - http://www.realtimerendering.com/blog/gpus-prefer-premultiplication
        // - https://www.khronos.org/opengl/wiki/Blending#Colors
        context.blend_equation_separate ( Context::FUNC_ADD, Context::FUNC_ADD );
        context.blend_func_separate     ( Context::ONE , Context::ONE_MINUS_SRC_ALPHA
                                        , Context::ONE , Context::ONE_MINUS_SRC_ALPHA );

        let pipeline = default();
        let width    = dom.shape().current().device_pixels().width  as i32;
        let height   = dom.shape().current().device_pixels().height as i32;
        let composer = RenderComposer::new(&pipeline,&context,&variables,width,height);

        Self { pipeline,composer,display_object,dom,context,symbols,camera,symbols_dirty,shape_dirty
             , logger,variables,stats,pixel_ratio,mouse,zoom_uniform
             ,zoom_callback,on_resize }
    }

    pub fn symbol_registry(&self) -> SymbolRegistry {
        self.symbols.clone_ref()
    }

    pub fn css3d_renderer(&self) -> Css3dRenderer {
        self.dom.layers.dom_front.clone()
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

    pub fn set_render_pipeline<P:Into<RenderPipeline>>(&mut self, pipeline:P) {
        self.pipeline = pipeline.into();
        self.init_composer();
    }

    pub fn init_composer(&mut self) {
        let width    = self.dom.shape().current().device_pixels().width  as i32;
        let height   = self.dom.shape().current().device_pixels().height as i32;
        self.composer = RenderComposer::new(&self.pipeline,&self.context,&self.variables,width,height);
    }

    pub fn render2(&self) {
        self.symbols.render2()
    }


    pub fn render(&mut self) {
        let mouse_hover_ids = self.mouse.hover_ids.get();
        if mouse_hover_ids != self.mouse.last_hover_ids {
            self.mouse.last_hover_ids = mouse_hover_ids;
            let is_not_background = mouse_hover_ids.w != 0;
            if is_not_background {
                let symbol_id = mouse_hover_ids.x;
                let symbol = self.symbols.index(symbol_id as usize);
                symbol.dispatch(&DynEvent::new(()));
                // println!("{:?}",self.mouse.hover_ids.get());
                // TODO: finish events sending, including OnOver and OnOut.
            }
        }

        group!(self.logger, "Updating.", {
            if self.shape_dirty.check_all() {
                let screen = self.dom.shape().current();
                self.resize_canvas(&self.dom.shape());
                self.camera.set_screen(screen.width, screen.height);
                self.init_composer();
                self.shape_dirty.unset_all();
            }
            if self.symbols_dirty.check_all() {
                self.symbols.update();
                self.symbols_dirty.unset_all();
            }
            self.logger.info("Rendering meshes.");
            let camera_changed = self.camera.update();
            if camera_changed {
                self.symbols.render(&self.camera);
                self.dom.layers.dom_front.update(&self.camera);
                self.dom.layers.dom_back.update(&self.camera);
            }
            self.composer.run();
        })
    }

    /// Bind FRP graph to mouse js events.
    pub fn bind_frp_to_mouse_events(&self, frp:&enso_frp::Mouse) -> MouseFrpCallbackHandles {
        mouse::bind_frp_to_mouse(&self.dom.shape(),frp,&self.mouse.mouse_manager)
    }

    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self) {
        self.display_object.update();
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
}}

impl Into<display::object::Node> for &SceneData {
    fn into(self) -> display::object::Node {
        self.display_object.clone()
    }
}

impl Into<display::object::Node> for &Scene {
    fn into(self) -> display::object::Node {
        let data:&SceneData = &self.rc.borrow();
        data.into()
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


// === Implementation ===

#[derive(Debug)]
pub struct Listeners {
    resize: ResizeObserver,
}

impl Scene {
    pub fn tmp_borrow_mut(&self) -> RefMut<'_,SceneData> {
        self.rc.borrow_mut()
    }
}

impl SceneData {
    /// Initialize all listeners and attach them to DOM elements.
    fn init_listeners
    (logger:&Logger, canvas:&web_sys::HtmlCanvasElement, shape:&Shape, dirty:&ShapeDirty)
    -> Listeners {
        let logger = logger.clone();
        let shape  = shape.clone();
        let dirty  = dirty.clone();
        let on_resize = Closure::new(move |width, height| {
            group!(logger, "Resize observer event.", {
                shape.set(width as f32,height as f32);
                dirty.set();
            })
        });
        let resize = ResizeObserver::new(canvas,on_resize);
        Listeners {resize}
    }



    /// Resize the underlying canvas. This function should rather not be called
    /// directly. If you want to change the canvas size, modify the `shape` and
    /// set the dirty flag.
    fn resize_canvas(&self, shape:&Shape) {
        let screen = shape.current();
        let canvas = shape.current().device_pixels();
        self.logger.group(fmt!("Resized to {}px x {}px.", screen.width, screen.height), || {
            self.dom.layers.canvas.set_attribute("width",  &canvas.width.to_string()).unwrap();
            self.dom.layers.canvas.set_attribute("height", &canvas.height.to_string()).unwrap();
            self.context.viewport(0,0,canvas.width as i32, canvas.height as i32);
//            self.on_resize2.iter().for_each(|f| f(shape));
        });
    }
}
