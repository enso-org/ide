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
//            let radius = 1.px() * radius;
//            let width  = WIDTH.px();
//            let width2 = width / 2.0;
//            let ring   = Circle(&radius + &width2) - Circle(radius-width2);
//            let right : Var<f32> = (std::f32::consts::PI/2.0).into();
//            let mask   = Plane().cut_angle_fast(&angle).rotate(right - angle/2.0);
//            let shape  = ring * mask;
//            let shape  = shape.fill(color::Rgba::from(color::Lcha::new(0.6,0.5,0.76,1.0)));
//            shape.into()
            let radius = 1.px() * radius;
            let width  = WIDTH.px();
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
            let width  = WIDTH.px();
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


const WIDTH : f32 = 4.0;


const OVERLAP : f32 = 1.0;

// ============
// === Connection ===
// ============

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


/// Internal data of `Connection`
#[derive(Debug)]
#[allow(missing_docs)]
pub struct ConnectionData {
    pub object : display::object::Instance,
    pub logger : Logger,
    pub network    : frp::Network,
    pub src_view   : component::ShapeView<shape::Shape>,
    pub helper1    : component::ShapeView<helper::Shape>,
    pub helper2    : component::ShapeView<helper::Shape>,
    pub helper3    : component::ShapeView<helper::Shape>,
    pub helper4    : component::ShapeView<helper::Shape>,
    pub side_line       : component::ShapeView<line::Shape>,
    pub main_line       : component::ShapeView<line::Shape>,
}

impl Connection {
    /// Constructor.
    pub fn new(scene:&Scene) -> Self {
        let logger    = Logger::new("node");
        let object    = display::object::Instance::new(&logger);
        let src_view  = component::ShapeView::<shape::Shape>::new(&logger,scene);
        let helper1   = component::ShapeView::<helper::Shape>::new(&logger,scene);
        let helper2   = component::ShapeView::<helper::Shape>::new(&logger,scene);
        let helper3   = component::ShapeView::<helper::Shape>::new(&logger,scene);
        let helper4   = component::ShapeView::<helper::Shape>::new(&logger,scene);
        let side_line      = component::ShapeView::<line::Shape>::new(&logger,scene);
        let main_line      = component::ShapeView::<line::Shape>::new(&logger,scene);

        object.add_child(&src_view);
        object.add_child(&helper1);
        object.add_child(&helper2);
        object.add_child(&helper3);
        object.add_child(&helper4);
        object.add_child(&side_line);
        object.add_child(&main_line);

        let port_x = 85.0;
        let port_width = 38.5;
        let port_height = 20.0;
        let node_height = 28.0;

        let source = Vector2::new(port_x + port_width/2.0, 0.0);





        side_line.shape.sprite.size().set(Vector2::new(10.0,100.0));
        side_line.mod_rotation(|r| r.z = std::f32::consts::PI/2.0);

        main_line.shape.sprite.size().set(Vector2::new(10.0,100.0));



        helper1.shape.sprite.size().set(Vector2::new(100.0,100.0));
        helper2.shape.sprite.size().set(Vector2::new(100.0,100.0));
        helper3.shape.sprite.size().set(Vector2::new(100.0,100.0));
        helper4.shape.sprite.size().set(Vector2::new(100.0,100.0));



        frp::new_network! { network
            def _tst = scene.mouse.frp.position.map(f!((side_line,main_line,src_view,helper1,helper2,helper3,helper4)(pos) {
                let test_angle = pos.y / 30.0 - 1.0;
                let test_angle = test_angle * std::f32::consts::PI;

                let target = Vector2::new(pos.x-300.0, pos.y-260.0);
                helper1.set_position(Vector3::new(target.x,target.y,0.0));


                let radius = 14.0;
                let width  = 300.0 / 2.0;
                let side_circle_x = width - radius;

                let side          = target.x.signum();
                let target_x      = target.x.abs();
                let corner_radius = 40.0;
                let corner_x      = target_x - corner_radius;


                let full_corner_x = radius;
                let x = (corner_x - side_circle_x).clamp(-corner_radius,full_corner_x);
                let y = (radius*radius + corner_radius*corner_radius - x*x).sqrt();


                let a1 = f32::atan2(y,x);
                let a2 = f32::atan2(radius,corner_radius);
                let a  = std::f32::consts::PI - a1 - a2;


                let angle_overlap = if (a == std::f32::consts::PI / 2.0) { 0.0 } else { 0.1 };

                src_view.shape.angle.set((a + angle_overlap) * side);


                let source_circle_y   = - y;
                let source_circle = Vector2::new(corner_x,source_circle_y);
                src_view.shape.sprite.size().set(Vector2::new(400.0,400.0));
                src_view.shape.radius.set(corner_radius);
                src_view.mod_position(|t| t.x = corner_x * side);
                src_view.mod_position(|t| t.y = source_circle_y);

                let line_overlap = 2.0;
                side_line.shape.sprite.size().set(Vector2::new(10.0,corner_x - width + line_overlap));
                side_line.mod_position(|p| p.x = side*(width + corner_x)/2.0);

                main_line.shape.sprite.size().set(Vector2::new(10.0,source_circle_y - target.y + line_overlap));
                main_line.mod_position(|p| {
                    p.x = side * target_x;
                    p.y = (target.y + source_circle_y) / 2.0;
                });



            }));
        }

        let data = Rc::new(ConnectionData {object,logger,network,src_view,helper1,helper2,helper3,helper4,side_line,main_line});
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