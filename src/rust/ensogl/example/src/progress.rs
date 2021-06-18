//! Example scene showing simple usage of a shape system.

use ensogl_core::prelude::*;

use ensogl_core::display::navigation::navigator::Navigator;
use ensogl_core::system::web;
use wasm_bindgen::prelude::*;
use ensogl_core::display::object::ObjectOps;
use ensogl_core::display::world::*;
use ensogl_core::display::shape::*;
use ensogl_core::data::color;
use ensogl_core::display::style::theme;
use std::f32::consts::PI;


// ==============
// === Shapes ===
// ==============

const PADDING:f32 = 2.0;

struct ProgressMaskParams<'a> {
    width        : &'a Var<Pixels>,
    height       : &'a Var<Pixels>,
    perimeter    : Var<Pixels>,
}

impl<'a> ProgressMaskParams<'a> {
    fn new(width:&'a Var<Pixels>, height:&'a Var<Pixels>) -> Self {
        let perimeter = width * 2.0 + height * (PI - 2.0);
        Self{width,height,perimeter}
    }
}

fn progress_mask_shape(params:ProgressMaskParams, val_range:Range<&Var<f32>>) -> AnyShape {
    let mut generator = FragmentShapesGenerator::new(params,val_range);
    let first  = generator.generate_fragment::<LeftUpperCircle>()  .fill(color::Rgba::new(1.0,0.0,0.0,1.0));
    let second = generator.generate_fragment::<TopEdge>()          .fill(color::Rgba::new(0.0,1.0,0.0,1.0));
    let third  = generator.generate_fragment::<RightCircle>()      .fill(color::Rgba::new(0.0,0.0,1.0,1.0));
    let fourth = generator.generate_fragment::<BottomEdge>()       .fill(color::Rgba::new(1.0,1.0,0.0,1.0));
    let fifth  = generator.generate_fragment::<LeftBottomCircle>() .fill(color::Rgba::new(0.0,1.0,1.0,1.0));
    (first + second + third + fourth + fifth).into()
}

trait ProgressMaskFragment {
    fn shape_from_normalized(params:&ProgressMaskParams, range:Range<Var<f32>>) -> AnyShape;

    fn perimeter_length(params:&ProgressMaskParams) -> Var<Pixels>;
}

struct FragmentShapesGenerator<'a> {
    params                               : ProgressMaskParams<'a>,
    value_range                          : Range<&'a Var<f32>>,
    generated_fragments_perimeter_length : Var<Pixels>,
}

impl<'a> FragmentShapesGenerator<'a> {
    fn new(params:ProgressMaskParams<'a>, value_range:Range<&'a Var<f32>>) -> Self {
        let generated_fragments_perimeter_length = 0.0.px().into();
        Self {params,value_range,generated_fragments_perimeter_length}
    }

    fn generate_fragment<Fragment:ProgressMaskFragment>(&mut self) -> AnyShape {
        let length    = Fragment::perimeter_length(&self.params);
        let normalize = |val:&Var<f32>| {
            let perimeter_space          = val * &self.params.perimeter;
            let fragment_perimeter_space = (perimeter_space - &self.generated_fragments_perimeter_length);
            let fragment_normalized      = fragment_perimeter_space / &length;
            Max::max(Min::min(fragment_normalized,1.0.into()),0.0.into())
        };
        let range_normalized = normalize(self.value_range.start)..normalize(self.value_range.end);
        self.generated_fragments_perimeter_length = &self.generated_fragments_perimeter_length + &length;
        Fragment::shape_from_normalized(&self.params,range_normalized)
    }
}


struct LeftUpperCircle;

impl ProgressMaskFragment for LeftUpperCircle {
    fn shape_from_normalized(params: &ProgressMaskParams, range:Range<Var<f32>>) -> AnyShape {
        let circle      = Circle(params.height / 2.0);
        let angle       = (&range.end - &range.start) * PI / 2.0;
        let angle_plane = PlaneAngleFast(angle.clone());
        let start_angle = &range.start * PI / 2.0;
        let angle_plane = angle_plane.rotate(angle / 2.0 - PI / 2.0 + start_angle);
        let arc         = circle.intersection(angle_plane);
        arc.translate_x(-params.width / 2.0 + params.height / 2.0).into()
    }

    fn perimeter_length(params: &ProgressMaskParams) -> Var<Pixels> {
        params.height * PI / 4.0
    }
}

struct TopEdge;

impl ProgressMaskFragment for TopEdge {
    fn shape_from_normalized(params: &ProgressMaskParams, range: Range<Var<f32>>) -> AnyShape {
        let length      = &range.end - &range.start;
        let max_width   = params.width - params.height;
        let shape_width = &max_width * length;
        let rect        = Rect((&shape_width,params.height/2.0));
        let moved_rect  = rect.translate_x(-(&max_width - &shape_width) / 2.0 + max_width * &range.start);
        moved_rect.translate_y(params.height / 4.0).into()
    }

    fn perimeter_length(params: &ProgressMaskParams) -> Var<Pixels> {
        params.width - params.height
    }
}

struct RightCircle;

impl ProgressMaskFragment for RightCircle {
    fn shape_from_normalized(params: &ProgressMaskParams, range: Range<Var<f32>>) -> AnyShape {
        let circle      = Circle(params.height / 2.0);
        let angle       = (&range.end - &range.start) * PI;
        let angle_plane = PlaneAngleFast(angle.clone());
        let start_angle = &range.start * PI;
        let angle_plane = angle_plane.rotate(angle / 2.0 + start_angle);
        let arc         = circle.intersection(angle_plane);
        arc.translate_x(params.width / 2.0 - params.height / 2.0).into()
    }

    fn perimeter_length(params: &ProgressMaskParams) -> Var<Pixels> {
        params.height / 2.0 * PI
    }
}

struct BottomEdge;
impl ProgressMaskFragment for BottomEdge {
    fn shape_from_normalized(params: &ProgressMaskParams, range: Range<Var<f32>>) -> AnyShape {
        let length      = &range.end - &range.start;
        let max_width   = params.width - params.height;
        let shape_width = &max_width * length;
        let rect        = Rect((&shape_width,params.height/2.0));
        let moved_rect  = rect.translate_x((&max_width - &shape_width) / 2.0 - &max_width * &range.start);
        moved_rect.translate_y(-params.height / 4.0).into()
    }

    fn perimeter_length(params: &ProgressMaskParams) -> Var<Pixels> {
        params.width - params.height
    }
}

struct LeftBottomCircle;
impl ProgressMaskFragment for LeftBottomCircle {
    fn shape_from_normalized(params: &ProgressMaskParams, range: Range<Var<f32>>) -> AnyShape {
        let circle      = Circle(params.height / 2.0);
        let angle       = (&range.end - &range.start) * PI / 2.0;
        let angle_plane = PlaneAngleFast(angle.clone());
        let start_angle = &range.start * PI / 2.0;
        let angle_plane = angle_plane.rotate(angle / 2.0 + PI + start_angle);
        let arc         = circle.intersection(angle_plane);
        arc.translate_x(-params.width / 2.0 + params.height / 2.0).into()
    }

    fn perimeter_length(params: &ProgressMaskParams) -> Var<Pixels> {
        params.height * PI / 4.0
    }
}

mod shape {
    use super::*;

    ensogl_core::define_shape_system! {
        (style:Style, start:f32, end:f32) {
            let width     = Var::<Pixels>::from("input_size.x");
            let height    = Var::<Pixels>::from("input_size.y");
            let width     = width  - PADDING.px() * 2.0;
            let height    = height - PADDING.px() * 2.0;
            let thickness = 10.0.px();
            let radius    = &height / 2.0;

            let params = ProgressMaskParams::new(&width,&height);
            let mask = progress_mask_shape(params,(&start)..(&end));

            // let shape  = Rect((&width,&height)).corners_radius(&radius);
            // let inner_w = &width - &thickness * 2.0;
            // let inner_h = &height - &thickness * 2.0;
            // let inner_r = &radius - &thickness;
            // let inner  = Rect((&inner_w,&inner_h)).corners_radius(&inner_r);
            //
            // let shape = shape - inner;
            // let shape  = shape.fill(color::Rgba::new(1.0,0.0,0.0,1.0));

            mask.into()
        }
    }
}


// ===================
// === Entry Point ===
// ===================

/// The example entry point.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_progress() {
    web::forward_panic_hook_to_console();
    web::set_stack_trace_limit();

    let world     = World::new(&web::get_html_element_by_id("root").unwrap());
    let scene     = world.scene();
    let camera    = scene.camera().clone_ref();
    let navigator = Navigator::new(&scene,&camera);
    let logger    = Logger::new("entry_point_progress");

    let view0 = shape::View::new(&logger);
    view0.size.set(Vector2::new(500.0, 200.0));

    let view1 = shape::View::new(&logger);
    view1.size.set(Vector2::new(500.0, 300.0));
    view1.set_position_y(300.0);
    let view2 = shape::View::new(&logger);
    view2.size.set(Vector2::new(500.0, 150.0));
    view2.set_position_y(-300.0);
    view2.end.set(1.0);

    world.add_child(&view0);
    world.add_child(&view1);
    world.add_child(&view2);
    world.on_frame(move |time| {
        let progress = (time.local / 2000.0).sin() / 2.0 + 0.5;
        view0.end.set(progress);
        view0.start.set(progress - 0.1);
        view1.end.set(progress);
        view2.start.set(progress);
    }).forget();
    world.keep_alive_forever();
    std::mem::forget(navigator);
}
