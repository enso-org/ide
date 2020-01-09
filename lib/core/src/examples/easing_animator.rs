//! EasingAnimator examples.

use wasm_bindgen::prelude::*;
use crate::animation::easing::*;
use crate::animation::animator::easing::EasingAnimator;
use crate::system::web::create_element;
use crate::system::web::NodeInserter;
use crate::system::web::AttributeSetter;
use crate::system::web::StyleSetter;
use crate::display::render::css3d::Object;

use nalgebra::{Vector2, Vector3};

use web_sys::{HtmlElement, HtmlCanvasElement, CanvasRenderingContext2d};
use wasm_bindgen::JsCast;
use crate::system::web::get_element_by_id;
use crate::animation::animator::continuous::ContinuousAnimator;
use basegl_system_web::animation_frame_loop::AnimationFrameLoop;
use crate::animation::position::HasPosition;
use crate::animation::animator::fixed_step::FixedStepAnimator;
use js_sys::Math;

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
/// A simplified Canvas object used in the EasingAnimator example.
pub struct Canvas {
    canvas  : HtmlCanvasElement,
    context : CanvasRenderingContext2d
}

impl Canvas {
    /// Creates a Canvas element inside the identified container.
    pub fn new(container_id:&str) -> Self {
        let canvas = create_element("canvas").unwrap();
        let canvas: HtmlCanvasElement = canvas.dyn_into().unwrap();
        canvas.set_property_or_panic("border", "1px solid black");

        canvas.set_width (256);
        canvas.set_height(256);

        let context = canvas.get_context("2d").unwrap().unwrap();
        let context : CanvasRenderingContext2d = context.dyn_into().unwrap();

        let app : HtmlElement = get_element_by_id(container_id).unwrap().dyn_into().unwrap();
        app.append_or_panic(&canvas);

        Self {canvas,context}
    }

    /// Clears the canvas.
    pub fn clear(&self) {
        self.context.clear_rect(0.0, 0.0, self.canvas.width() as f64, self.canvas.height() as f64)
    }

    /// Gets Canvas' width.
    pub fn width(&self) -> f64 { self.canvas.width() as f64 }

    /// Gets Canvas` height.
    pub fn height(&self) -> f64 { self.canvas.height() as f64 }

    /// Draw a point
    pub fn point(&self, point:Vector2<f64>, color:&str) {
        let size = 20.0 / self.height();
        self.context.save();
        self.context.set_fill_style(&color.into());
        self.context.scale(self.width() / 2.0, self.height() / 2.0).ok();
        self.context.set_line_width(2.0 / self.height());
        self.context.translate(1.0, 1.0).ok();
        self.context.fill_rect(point.x - size / 2.0, point.y - size / 2.0, size, size);
        self.context.restore();
    }

    /// Draw a 2D graph of the provided FnEasing function.
    pub fn graph<F:FnEasing>(&self, f:F, color:&str, time_ms:f64) {
        let width  = self.width() - 1.0;
        let height = self.height();

        self.context.set_stroke_style(&color.into());
        self.context.begin_path();
        self.context.save();
        self.context.scale(width, height / 2.0).ok();
        self.context.translate(0.0, 0.5).ok();
        self.context.set_line_width(1.0 / height);
        self.context.move_to(0.0, f(0.0));
        for x in 1..self.canvas.width() {
            let x = x as f64 / width;
            let y = f(x);
            self.context.line_to(x, y);
        }
        self.context.stroke();

        self.context.set_fill_style(&color.into());
        let width  = 8.0  / width;
        let height = 16.0 / height;
        let time_seconds = time_ms / 2000.0;
        let x      = time_seconds % 1.0;
        let y      = f(x);
        self.context.fill_rect(x - width / 2.0, y - height / 2.0, width, height);
        self.context.restore();
    }
}

/// Creates a Vector3<f32> with random components from -1 to 1.
fn vector3_random() -> Vector3<f32> {
    let x = ((Math::random() - 0.5) * 2.0) as f32;
    let y = ((Math::random() - 0.5) * 2.0) as f32;
    let z = ((Math::random() - 0.5) * 2.0) as f32;
    Vector3::new(x, y, z)
}

struct SharedData {
    graph_canvas     : Canvas,
    animation_canvas : Canvas,
    easing_animator  : EasingAnimator,
    object           : Object,
    easing_function  : &'static dyn FnEasing,
    event_loop       : AnimationFrameLoop
}

#[derive(Clone)]
struct SubExample {
    data : Rc<RefCell<SharedData>>
}

impl SubExample {
    fn new<F>
    ( mut event_loop   : &mut AnimationFrameLoop
    , graph_canvas     : Canvas
    , animation_canvas : Canvas
    , f                : &'static F
    , origin_position  : Vector3<f32>
    , target_position  : Vector3<f32>) -> Self
    where F:FnEasing {
        let object          = Object::new();
        let easing_animator = EasingAnimator::new(
            &mut event_loop,
            f,
            object.clone(),
            origin_position,
            target_position,
            2.0
        );
        let event_loop      = event_loop.clone();
        let easing_function = f;
        let data = SharedData {
            object,
            easing_function,
            graph_canvas,
            animation_canvas,
            easing_animator,
            event_loop
        };
        let data = Rc::new(RefCell::new(data));
        Self {data}
    }

    fn set_position(&mut self, target_position:Vector3<f32>) {
        let mut data         = self.data.borrow_mut();
        let origin_position  = data.object.position();
        let easing_function  = data.easing_function;
        let object           = data.object.clone();
        data.easing_animator = EasingAnimator::new(
            &mut data.event_loop,
            easing_function,
            object,
            origin_position,
            target_position,
            2.0
        );
    }

    fn render(&self, color:&str, time_ms:f64) {
        let data = self.data.borrow();
        let position = data.object.position();
        data.graph_canvas.graph(data.easing_function, color, time_ms);
        data.animation_canvas.point(Vector2::new(position.x as f64, position.y as f64), color);
    }
}

struct Example {
    _animator : ContinuousAnimator
}

impl Example {
    pub fn new<F1, F2, F3>
    ( mut event_loop : &mut AnimationFrameLoop
    , name           : &str
    , ease_in        : &'static F1
    , ease_out       : &'static F2
    , ease_in_out    : &'static F3) -> Self
    where F1:FnEasing, F2:FnEasing, F3:FnEasing {
        let example : HtmlElement = create_element("div").unwrap().dyn_into().unwrap();
        example.set_attribute_or_panic("id", name);
        example.set_property_or_panic("margin", "10px");
        let container : HtmlElement = get_element_by_id("examples").unwrap().dyn_into().unwrap();
        let header    : HtmlElement = create_element("center").unwrap().dyn_into().unwrap();
        header.set_property_or_panic("background-color", "black");
        header.set_property_or_panic("color", "white");
        header.set_inner_html(name);
        example.append_or_panic(&header);
        container.append_or_panic(&example);
        let graph_canvas     = Canvas::new(name);
        let animation_canvas = Canvas::new(name);

        let origin_position  = Vector3::new(0.0, 0.0, 0.0);
        let target_position  = vector3_random();

        let mut easing_in = SubExample::new(
            &mut event_loop,
            graph_canvas.clone(),
            animation_canvas.clone(),
            ease_in,
            origin_position.clone(),
            target_position.clone()
        );
        let easing_in_clone = easing_in.clone();

        let mut easing_out = SubExample::new(
            &mut event_loop,
            graph_canvas.clone(),
            animation_canvas.clone(),
            ease_out,
            origin_position.clone(),
            target_position.clone()
        );
        let easing_out_clone = easing_out.clone();

        let mut easing_in_out = SubExample::new(
            &mut event_loop,
            graph_canvas.clone(),
            animation_canvas.clone(),
            ease_in_out,
            origin_position.clone(),
            target_position.clone()
        );
        let easing_in_out_clone = easing_in_out.clone();

        let _fixed_step = FixedStepAnimator::new(&mut event_loop, 0.5, move |_| {
            let target_position = vector3_random();
            easing_in.set_position(target_position.clone());
            easing_out.set_position(target_position.clone());
            easing_in_out.set_position(target_position.clone());
        });

        let _animator = ContinuousAnimator::new(&mut event_loop, move |time_ms:f32| {
            let _keep_alive = &_fixed_step;
            graph_canvas.clear();
            animation_canvas.clear();
            easing_in_clone.render("red", time_ms as f64);
            easing_out_clone.render("green", time_ms as f64);
            easing_in_out_clone.render("blue", time_ms as f64);
        });
        Self { _animator }
    }
}

macro_rules! example {
    ($event_loop:ident, $name:ident) => {
        std::mem::forget(Example::new(
            &mut $event_loop,
            stringify!($name),
            &paste::expr!{[<$name _in>]},
            &paste::expr!{[<$name _out>]},
            &paste::expr!{[<$name _in_out>]},
        ))
    };
}

#[wasm_bindgen]
#[allow(dead_code)]
/// Runs EasingAnimator example.
pub fn run_example_easing_animator() {
    let mut event_loop = AnimationFrameLoop::new();
    let container : HtmlElement = create_element("div").unwrap().dyn_into().unwrap();
    container.set_attribute_or_panic("id", "examples");
    container.set_property_or_panic("display", "flex");
    container.set_property_or_panic("flex-wrap", "wrap");
    container.set_property_or_panic("position", "absolute");
    container.set_property_or_panic("top", "0px");
    get_element_by_id("app").unwrap().append_or_panic(&container);
    example!(event_loop, expo);
    example!(event_loop, bounce);
    example!(event_loop, circ);
    example!(event_loop, quad);
    example!(event_loop, cubic);
    example!(event_loop, quart);
    example!(event_loop, quint);
    example!(event_loop, sine);
    example!(event_loop, back);
    example!(event_loop, elastic);
    std::mem::forget(event_loop);
}
