//! Definition of the Connection component.


use crate::prelude::*;

//use crate::component::node::port::Registry;

use enso_frp;
use enso_frp as frp;
use ensogl::data::color;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::Sprite;
use ensogl::display::scene::Scene;
use ensogl::display::scene::ShapeRegistry;
use ensogl::display::shape::*;
use ensogl::display::traits::*;
use ensogl::display;
use ensogl::gui::component::animation;
use ensogl::gui::component;
use ensogl::display::shape::text::glyph::font::FontRegistry;
use ensogl::display::shape::text::glyph::system::GlyphSystem;
use ensogl::math::topology::unit::AngleOps;


// ==================
// === Connection ===
// ==================

/// Canvas node shape definition.
pub mod shape {
    use super::*;

    ensogl::define_shape_system! {
        (radius:f32, start_angle:f32, angle:f32) {
            let radius = 1.px() * radius;
            let width  = LINE_WIDTH.px();
            let width2 = width / 2.0;
            let ring   = Circle(&radius + &width2) - Circle(radius-width2);
            let right : Var<f32> = (std::f32::consts::PI/2.0).into();
            let rot    = right - &angle/2.0;
            let mask   = Plane().cut_angle_fast(angle).rotate(rot);
            let shape  = ring * mask;
            let shape  = shape.fill(color::Rgba::from(color::Lcha::new(0.6,0.5,0.76,1.0)));
            shape.into()
        }
    }
}


/// Canvas node shape definition.
pub mod line {
    use super::*;

    ensogl::define_shape_system! {
        () {
            let width  = LINE_WIDTH.px();
            let height : Var<Distance<Pixels>> = "input_size.y".into();
            let shape  = Rect((width,height));
            let shape  = shape.fill(color::Rgba::from(color::Lcha::new(0.6,0.5,0.76,1.0)));
            shape.into()
        }
    }
}


/// Canvas node shape definition.
pub mod helper {
    use super::*;

    ensogl::define_shape_system! {
        () {
            let shape = Circle(2.px());
            let shape = shape.fill(color::Rgba::new(1.0,0.0,0.0,1.0));
            shape.into()
        }
    }
}


const LINE_WIDTH : f32 = 4.0;
const PADDING    : f32 = 5.0;



// ==================
// === Connection ===
// ==================

/// Connection definition.
#[derive(AsRef,Clone,CloneRef,Debug,Deref)]
pub struct Connection {
    data : Rc<ConnectionData>,
}

impl AsRef<Connection> for Connection {
    fn as_ref(&self) -> &Self {
        self
    }
}


fn ease_out_quad(t:f32) -> f32 {
    let t = t.clamp(0.0,1.0);
    return 1.0 - (1.0 - t) * (1.0 - t);
}


#[derive(Clone,CloneRef,Debug)]
pub struct InputEvents {
    pub network         : frp::Network,
    pub target_position : frp::Source<frp::Position>,
}

impl InputEvents {
    pub fn new() -> Self {
        frp::new_network! { network
            def target_position = source();
        }
        Self {network,target_position}
    }
}


/// Internal data of `Connection`
#[derive(Debug)]
#[allow(missing_docs)]
pub struct ConnectionData {
    pub object    : display::object::Instance,
    pub logger    : Logger,
    pub events    : InputEvents,
    pub corner    : component::ShapeView<shape::Shape>,
    pub side_line : component::ShapeView<line::Shape>,
    pub main_line : component::ShapeView<line::Shape>,
}

impl Connection {
    /// Constructor.
    pub fn new(scene:&Scene) -> Self {
        let logger    = Logger::new("node");
        let object    = display::object::Instance::new(&logger);

        let corner    = component::ShapeView::<shape::Shape>::new(&logger,scene);
        let side_line = component::ShapeView::<line::Shape>::new(&logger,scene);
        let main_line = component::ShapeView::<line::Shape>::new(&logger,scene);
        object.add_child(&corner);
        object.add_child(&side_line);
        object.add_child(&main_line);
        side_line.mod_rotation(|r| r.z = std::f32::consts::PI/2.0);

        let input = InputEvents::new();
        let network = &input.network;

        frp::extend! { network
            // input.target_position
            def _tst = scene.mouse.frp.position.map(f!((object,side_line,main_line,corner)(target) {
                let target = Vector2::new(target.x - object.position().x, target.y - object.position().y);
                let radius = 14.0;
                let width  = 284.0 / 2.0;

                let side_circle_x = width - radius;
                let side          = target.x.signum();
                let target        = Vector2::new(target.x.abs(),target.y);

                let corner_grow   = ((target.x - width) * 0.6).max(0.0);
                let corner_radius = 20.0 + corner_grow;
                let corner_radius = corner_radius.min(target.y.abs());
                let corner_x      = target.x - corner_radius;


                let x = (corner_x - side_circle_x).clamp(-corner_radius,radius);
                let y = (radius*radius + corner_radius*corner_radius - x*x).sqrt();


                let angle1        = f32::atan2(y,x);
                let angle2        = f32::atan2(radius,corner_radius);
                let corner_angle  = std::f32::consts::PI - angle1 - angle2;
                let angle_overlap = if corner_x > width { 0.0 } else { 0.1 };

                corner.shape.angle.set((corner_angle + angle_overlap) * side);


                let corner_y    = - y;
                let corner_side = (corner_radius + PADDING) * 2.0;
                corner.shape.sprite.size().set(Vector2::new(corner_side,corner_side));
                corner.shape.radius.set(corner_radius);
                corner.mod_position(|t| t.x = corner_x * side);
                corner.mod_position(|t| t.y = corner_y);

                let line_overlap = 2.0;
                side_line.shape.sprite.size().set(Vector2::new(10.0,corner_x - width + line_overlap));
                side_line.mod_position(|p| p.x = side*(width + corner_x)/2.0);

                println!("{}",corner_y - target.y + line_overlap);
                main_line.shape.sprite.size().set(Vector2::new(10.0,corner_y - target.y + line_overlap));
                main_line.mod_position(|p| {
                    p.x = side * target.x;
                    p.y = (target.y + corner_y) / 2.0;
                });
            }));
        }

        let events = input;
        let data = Rc::new(ConnectionData {object,logger,events,corner,side_line,main_line});
        Self {data}
    }
}

impl display::Object for Connection {
    fn display_object(&self) -> &display::object::Instance {
        &self.object
    }
}



//fn inner_tangent_lines_intersection_point_for_two_circles
fn inner_tangent_lines_touch_points_for_two_circles
(center1:Vector2<f32>,radius1:f32,center2:Vector2<f32>,radius2:f32)
-> (Vector2<f32>,Vector2<f32>,Vector2<f32>,Vector2<f32>) {
    let radius_sum = radius1 + radius2;
    let cross_x    = (center2.x*radius1 + center1.x*radius2) / radius_sum;
    let cross_y    = (center2.y*radius1 + center1.y*radius2) / radius_sum;
    let cross      = Vector2::new(cross_x,cross_y);

    let go = |side:f32, center:Vector2<f32>, radius:f32| {
        let cross_center = cross - center;
        let cross_center_2 = cross_center.component_mul(&cross_center);
        let r1_2 = radius * radius;

        let q   = (cross_center_2.x + cross_center_2.y - r1_2).sqrt();
        let div = (cross_center_2.x + cross_center_2.y);

        let x = (r1_2 * cross_center.x + side * radius * cross_center.y * q) / div + center.x;
        let y = (r1_2 * cross_center.y - side * radius * cross_center.x * q) / div + center.y;
        Vector2::new(x,y)
    };

    let point1_1 = go( 1.0,center1,radius1);
    let point1_2 = go(-1.0,center1,radius1);
    let point2_1 = go( 1.0,center2,radius2);
    let point2_2 = go(-1.0,center2,radius2);

    (point1_1,point1_2,point2_1,point2_2)
}