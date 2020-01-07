use wasm_bindgen::prelude::*;
use crate::animation::easing::*;
use crate::system::web::console_log;
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
use crate::animation::HasPosition;
use crate::animation::animator::fixed_step::FixedStepAnimator;
use js_sys::Math;

pub struct Canvas {
    canvas  : HtmlCanvasElement,
    context : CanvasRenderingContext2d
}

impl Canvas {
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

    pub fn clear(&self) {
        self.context.clear_rect(0.0, 0.0, self.canvas.width() as f64, self.canvas.height() as f64)
    }

    pub fn width(&self) -> f64 { self.canvas.width() as f64 }

    pub fn height(&self) -> f64 { self.canvas.height() as f64 }

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

fn Vector3_random() -> Vector3<f32> {
    let x = ((Math::random() - 0.5) * 2.0) as f32;
    let y = ((Math::random() - 0.5) * 2.0) as f32;
    Vector3::new(x, y, 0.0)
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

        let mut object_in       = Object::new();
        let object_in_clone     = object_in.clone();
        let mut object_out      = Object::new();
        let object_out_clone    = object_out.clone();
        let mut object_in_out   = Object::new();
        let object_in_out_clone = object_in_out.clone();

        let origin_position  = Vector3::new(0.0, 0.0, 0.0);
        let target_position  = Vector3_random();

        let mut ease_in_animator = EasingAnimator::new(
            &mut event_loop,
            ease_in,
            object_in.clone(),
            origin_position.clone(),
            target_position.clone(),
            2.0
        );
        let mut ease_out_animator = EasingAnimator::new(
            &mut event_loop,
            ease_out,
            object_out.clone(),
            origin_position.clone(),
            target_position.clone(),
            2.0
        );
        let mut ease_in_out_animator = EasingAnimator::new(
            &mut event_loop,
            ease_in_out,
            object_in_out.clone(),
            origin_position,
            target_position,
            2.0
        );


        let mut event_loop_clone = event_loop.clone();
        let _fixed_step = FixedStepAnimator::new(&mut event_loop, 0.5, move |_| {
            let origin_position = object_in.position();
            let target_position = Vector3_random();
            ease_in_animator = EasingAnimator::new(
                &mut event_loop_clone,
                ease_in,
                object_in.clone(),
                origin_position.clone(),
                target_position.clone(),
                2.0
            );
            ease_out_animator = EasingAnimator::new(
                &mut event_loop_clone,
                ease_out,
                object_out.clone(),
                origin_position.clone(),
                target_position.clone(),
                2.0
            );
            ease_in_out_animator = EasingAnimator::new(
                &mut event_loop_clone,
                ease_in_out,
                object_in_out.clone(),
                origin_position,
                target_position,
                2.0
            );
        });
        std::mem::forget(_fixed_step);
        let _animator = ContinuousAnimator::new(&mut event_loop, move |time_ms:f32| {
            graph_canvas.clear();
            graph_canvas.graph(ease_in    , "red"  , time_ms as f64);
            graph_canvas.graph(ease_out   , "green", time_ms as f64);
            graph_canvas.graph(ease_in_out, "blue" , time_ms as f64);
            animation_canvas.clear();
            let position = object_in_clone.position();
            animation_canvas.point(Vector2::new(position.x as f64, position.y as f64), "red");
            let position = object_out_clone.position();
            animation_canvas.point(Vector2::new(position.x as f64, position.y as f64), "green");
            let position = object_in_out_clone.position();
            animation_canvas.point(Vector2::new(position.x as f64, position.y as f64), "blue");
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
pub fn run_example_animation_manager() {
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
    std::mem::drop(event_loop);
}
