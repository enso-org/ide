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
            let width  = WIDTH.px();
            let width2 = width / 2.0;
            let ring   = Circle(&radius + &width2) - Circle(radius-width2);
            let rot    = -&angle/2.0 - start_angle;
            let mask   = Plane().cut_angle(angle).rotate(rot);
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





/// Internal data of `Connection`
#[derive(Debug)]
#[allow(missing_docs)]
pub struct ConnectionData {
    pub object : display::object::Instance,
    pub logger : Logger,
    pub network    : frp::Network,
    pub src_view   : component::ShapeView<shape::Shape>,
    pub tgt_view   : component::ShapeView<shape::Shape>,
    pub helper1    : component::ShapeView<helper::Shape>,
    pub helper2    : component::ShapeView<helper::Shape>,
    pub helper3    : component::ShapeView<helper::Shape>,
    pub helper4    : component::ShapeView<helper::Shape>,
    pub line       : component::ShapeView<line::Shape>,
}

impl Connection {
    /// Constructor.
    pub fn new(scene:&Scene) -> Self {
        let logger    = Logger::new("node");
        let object    = display::object::Instance::new(&logger);
        let src_view  = component::ShapeView::<shape::Shape>::new(&logger,scene);
        let tgt_view  = component::ShapeView::<shape::Shape>::new(&logger,scene);
        let helper1   = component::ShapeView::<helper::Shape>::new(&logger,scene);
        let helper2   = component::ShapeView::<helper::Shape>::new(&logger,scene);
        let helper3   = component::ShapeView::<helper::Shape>::new(&logger,scene);
        let helper4   = component::ShapeView::<helper::Shape>::new(&logger,scene);
        let line      = component::ShapeView::<line::Shape>::new(&logger,scene);

        object.add_child(&src_view);
        object.add_child(&tgt_view);
        object.add_child(&helper1);
        object.add_child(&helper2);
        object.add_child(&helper3);
        object.add_child(&helper4);
        object.add_child(&line);

        let port_x = 85.0;
        let port_width = 38.5;
        let port_height = 20.0;
        let node_height = 28.0;
        let src_r = 20.0;
        let src_x = port_x + port_width/2.0 + src_r;
        let src_y = (node_height - port_height)/2.0 + OVERLAP;
        let src_pos = Vector2::new(src_x,src_y);
        src_view.shape.sprite.size().set(Vector2::new(100.0,100.0));
        src_view.shape.radius.set(src_r);
        src_view.shape.start_angle.set(std::f32::consts::PI / 2.0);
        src_view.mod_position(|t| t.x = src_x);
        src_view.mod_position(|t| t.y = src_y);


        line.shape.sprite.size().set(Vector2::new(10.0,100.0));



        let tgt_r = 30.0;


        tgt_view.shape.sprite.size().set(Vector2::new(100.0,100.0));
        tgt_view.shape.angle.set(std::f32::consts::PI);
        tgt_view.shape.start_angle.set(std::f32::consts::PI * 1.5);



        helper1.shape.sprite.size().set(Vector2::new(100.0,100.0));
        helper2.shape.sprite.size().set(Vector2::new(100.0,100.0));
        helper3.shape.sprite.size().set(Vector2::new(100.0,100.0));
        helper4.shape.sprite.size().set(Vector2::new(100.0,100.0));



        frp::new_network! { network
            def _tst = scene.mouse.frp.position.map(f!((line,src_view,tgt_view,helper1,helper2,helper3,helper4)(pos) {
                let tgt_x = pos.x;
                let tgt_y = -180.0 + pos.y;
                let tgt_pos = Vector2::new(tgt_x,tgt_y);

                tgt_view.shape.radius.set(tgt_r);
                tgt_view.mod_position(|t| t.x = tgt_x);
                tgt_view.mod_position(|t| t.y = tgt_y);


                let ps = inner_tangent_lines_touch_points_for_two_circles(src_pos,src_r,tgt_pos,tgt_r);
                let p1 = ps.0;
                let p2 = ps.2;
//                helper1.mod_position(|t| { t.x = p2.x; t.y = p2.y; });
//                helper2.mod_position(|t| { t.x = ps.1.x; t.y = ps.1.y; });
//                helper3.mod_position(|t| { t.x = ps.2.x; t.y = ps.2.y; });
//                helper4.mod_position(|t| { t.x = ps.3.x; t.y = ps.3.y; });

                let dy    = p1.y - src_y;
                let dx    = p1.x - src_x;
                let angle = std::f32::consts::PI + f32::atan2(dy,dx);
                src_view.shape.angle.set(angle);

                let dy    = p2.y - tgt_y;
                let dx    = p2.x - tgt_x;
                let angle = f32::atan2(dy,dx);
                tgt_view.shape.angle.set(angle);

                let line_pos = (p1 + p2)/2.0;
                let line_pos = Vector3::new(line_pos.x, line_pos.y, 0.0);
                line.set_position(line_pos);
                let diff  = p2 - p1;
                let angle = f32::atan2(diff.y,diff.x) + std::f32::consts::PI / 2.0;
                let dist  = (diff.x*diff.x + diff.y*diff.y).sqrt();
                line.shape.sprite.size().set(Vector2::new(10.0,dist + 2.0));
                line.mod_rotation(|r| r.z = angle);

                println!("{}", angle);
            }));
        }

        let data = Rc::new(ConnectionData {object,logger,network,src_view,tgt_view,helper1,helper2,helper3,helper4,line});
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