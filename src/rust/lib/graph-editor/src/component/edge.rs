//! Definition of the Edge component.

#![allow(missing_docs)]
// WARNING! UNDER HEAVY DEVELOPMENT. EXPECT DRASTIC CHANGES.

use crate::prelude::*;

use enso_frp;
use enso_frp as frp;
use ensogl::data::color;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::Sprite;
use ensogl::display::scene::Scene;
use ensogl::display::shape::*;
use ensogl::display::traits::*;
use ensogl::display;
use ensogl::gui::component;

use super::node;



macro_rules! define_corner {($($color:tt)*) => {
    /// Shape definition.
    pub mod corner {
        use super::*;
        ensogl::define_shape_system! {
            (radius:f32, angle:f32, start_angle:f32, pos:Vector2<f32>, width:f32) {
                let radius = 1.px() * radius;
                let ww  = LINE_WIDTH.px();
                let width2 = ww / 2.0;
                let ring   = Circle(&radius + &width2) - Circle(radius-width2);
                let right : Var<f32> = (std::f32::consts::PI/2.0).into();
                let rot    = right - &angle/2.0 + start_angle;
                let mask   = Plane().cut_angle_fast(angle).rotate(rot);
                let shape  = ring * mask;

                let shadow_size = 10.px();
                let n_radius = &shadow_size + 14.px();
                let n_shape  = Rect((&shadow_size*2.0 + 2.px() * width,&n_radius*2.0)).corners_radius(n_radius);
                let n_shape  = n_shape.fill(color::Rgba::new(1.0,0.0,0.0,1.0));
                let tx       = - 1.px() * pos.x();
                let ty       = - 1.px() * pos.y();
                let n_shape  = n_shape.translate((tx,ty));

                let shape = shape - n_shape;
                let shape = shape.fill(color::Rgba::from($($color)*));

                shape.into()
            }
        }
    }
}}

macro_rules! define_corner2 {($($color:tt)*) => {
    /// Shape definition.
    pub mod corner {
        use super::*;
        ensogl::define_shape_system! {
            (radius:f32, angle:f32, start_angle:f32, pos:Vector2<f32>, dim:Vector2<f32>) {
                let radius = 1.px() * radius;
                let width  = LINE_WIDTH.px();
                let width2 = width / 2.0;
                let ring   = Circle(&radius + &width2) - Circle(radius-width2);
                let right : Var<f32> = (std::f32::consts::PI/2.0).into();
                let rot    = right - &angle/2.0 + start_angle;
                let mask   = Plane().cut_angle_fast(angle).rotate(rot);
                let shape  = ring * mask;

                let shadow_size = 10.px() + 1.px();
                let n_radius = &shadow_size + 1.px() * dim.y();
                let n_shape  = Rect((&shadow_size*2.0 + 2.px() * dim.x(),&n_radius*2.0)).corners_radius(n_radius);
                let n_shape  = n_shape.fill(color::Rgba::new(1.0,0.0,0.0,1.0));
                let tx       = - 1.px() * pos.x();
                let ty       = - 1.px() * pos.y();
                let n_shape  = n_shape.translate((tx,ty));

                let shape = shape * n_shape;
                let shape = shape.fill(color::Rgba::from($($color)*));

                shape.into()
            }
        }
    }
}}

macro_rules! define_line {($($color:tt)*) => {
    /// Shape definition.
    pub mod line {
        use super::*;
        ensogl::define_shape_system! {
            () {
                let width  = LINE_WIDTH.px();
                let height : Var<Distance<Pixels>> = "input_size.y".into();
                let shape  = Rect((width,height));
                let shape  = shape.fill(color::Rgba::from($($color)*));
                shape.into()
            }
        }
    }
}}

macro_rules! define_arrow {($($color:tt)*) => {
    /// Shape definition.
    pub mod arrow {
        use super::*;
        ensogl::define_shape_system! {
            () {
                let width  : Var<Distance<Pixels>> = "input_size.x".into();
                let height : Var<Distance<Pixels>> = "input_size.y".into();
                let width      = width  - (2.0 * PADDING).px();
                let height     = height - (2.0 * PADDING).px();
                let triangle   = Triangle(width,height);
                let offset     = (LINE_WIDTH/2.0).px();
                let triangle_l = triangle.translate_x(-&offset);
                let triangle_r = triangle.translate_x(&offset);
                let shape      = triangle_l + triangle_r;
                let shape      = shape.fill(color::Rgba::from($($color)*));
                shape.into()
            }
        }
    }
}}


// ============
// === Edge ===
// ============

pub mod front {
    use super::*;
    define_corner!(color::Lcha::new(0.6,0.5,0.76,1.0));
    define_line!(color::Lcha::new(0.6,0.5,0.76,1.0));
    define_arrow!(color::Lcha::new(0.6,0.5,0.76,1.0));
}

pub mod back {
    use super::*;
    define_corner2!(color::Lcha::new(0.6,0.5,0.76,1.0));
    define_line!(color::Lcha::new(0.6,0.5,0.76,1.0));
    define_arrow!(color::Lcha::new(0.6,0.5,0.76,1.0));
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
const PADDING    : f32 = 4.0;



// ============
// === Edge ===
// ============

/// Edge definition.
#[derive(AsRef,Clone,CloneRef,Debug,Deref)]
pub struct Edge {
    #[deref]
    model   : Rc<EdgeModel>,
    network : frp::Network,
}

impl AsRef<Edge> for Edge {
    fn as_ref(&self) -> &Self {
        self
    }
}


#[derive(Clone,CloneRef,Debug)]
pub struct Frp {
    pub source_width    : frp::Source<f32>,
    pub target_position : frp::Source<frp::Position>,
    pub target_attached : frp::Source<bool>,
}

impl Frp {
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! { network
            def source_width    = source();
            def target_position = source();
            def target_attached = source();
        }
        Self {source_width,target_position,target_attached}
    }
}


pub fn sort_hack_1(scene:&Scene) {
    let logger = Logger::new("hack");
    component::ShapeView::<back::corner::Shape>::new(&logger,scene);
    component::ShapeView::<back::line::Shape>::new(&logger,scene);
}

pub fn sort_hack_2(scene:&Scene) {
    let logger = Logger::new("hack");
    component::ShapeView::<front::corner::Shape>::new(&logger,scene);
    component::ShapeView::<front::line::Shape>::new(&logger,scene);
}


macro_rules! define_components {
    ($name:ident {
        $($field:ident : $field_type:ty),* $(,)?
    }) => {
        #[derive(Debug,Clone,CloneRef)]
        pub struct $name {
            pub logger         : Logger,
            pub display_object : display::object::Instance,
            $(pub $field : component::ShapeView<$field_type>),*
        }

        impl $name {
            pub fn new(logger:Logger, scene:&Scene) -> Self {
                let display_object = display::object::Instance::new(&logger);
                $(let $field = component::ShapeView::new(&logger.sub(stringify!($field)),scene);)*
                $(display_object.add_child(&$field);)*
                Self {logger,display_object,$($field),*}
            }
        }

        impl display::Object for $name {
            fn display_object(&self) -> &display::object::Instance {
                &self.display_object
            }
        }
    }
}

define_components!{
    Front {
        corner     : front::corner::Shape,
        corner2    : front::corner::Shape,
        corner3    : front::corner::Shape,
        side_line  : front::line::Shape,
        side_line2 : front::line::Shape,
        main_line  : front::line::Shape,
        port_line  : front::line::Shape,
        arrow      : front::arrow::Shape,
    }
}

define_components!{
    Back {
        corner     : back::corner::Shape,
        corner2    : back::corner::Shape,
        corner3    : back::corner::Shape,
        side_line  : back::line::Shape,
        side_line2 : back::line::Shape,
        main_line  : back::line::Shape,
        arrow      : front::arrow::Shape,
    }
}


/// Internal data of `Edge`
#[derive(Debug)]
#[allow(missing_docs)]
pub struct EdgeModelData {
    pub display_object  : display::object::Instance,
    pub logger          : Logger,
    pub frp             : Frp,
    pub front           : Front,
    pub back            : Back,
    pub source_width    : Rc<Cell<f32>>,
    pub target_position : Rc<Cell<frp::Position>>,
    pub target_attached : Rc<Cell<bool>>,
}

/// Edge definition.
#[derive(AsRef,Clone,CloneRef,Debug,Deref)]
pub struct EdgeModel {
    data : Rc<EdgeModelData>,
}

const INFINITE : f32 = 99999.0;



impl display::Object for EdgeModelData {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}

fn min(a:f32,b:f32) -> f32 {
    f32::min(a,b)
}


fn max(a:f32,b:f32) -> f32 {
    f32::max(a,b)
}

const LINE_SHAPE_WIDTH   : f32 = LINE_WIDTH + 2.0 * PADDING;
const LINE_SIDE_OVERLAP  : f32 = 1.0;
const LINE_SIDES_OVERLAP : f32 = 2.0 * LINE_SIDE_OVERLAP;

trait LayoutLine {
    fn layout(&self,start:Vector2<f32>,len:f32);
    fn layout_no_overlap(&self,start:Vector2<f32>,len:f32);
}

impl LayoutLine for component::ShapeView<front::line::Shape> {
    fn layout(&self, start: Vector2<f32>, len: f32) {
        let pos = Vector2::new(start.x, start.y + len / 2.0);
        let size = Vector2::new(LINE_SHAPE_WIDTH, len + LINE_SIDES_OVERLAP);
        self.shape.sprite.size().set(size);
        self.set_position_xy(pos);
    }
    fn layout_no_overlap(&self, start: Vector2<f32>, len: f32) {
        let pos = Vector2::new(start.x, start.y + len / 2.0);
        let size = Vector2::new(LINE_SHAPE_WIDTH, len);
        self.shape.sprite.size().set(size);
        self.set_position_xy(pos);
    }
}

impl LayoutLine for component::ShapeView<back::line::Shape> {
    fn layout(&self, start: Vector2<f32>, len: f32) {
        let pos = Vector2::new(start.x, start.y + len / 2.0);
        let size = Vector2::new(LINE_SHAPE_WIDTH, len + LINE_SIDES_OVERLAP);
        self.shape.sprite.size().set(size);
        self.set_position_xy(pos);
    }
    fn layout_no_overlap(&self, start: Vector2<f32>, len: f32) {
        let pos = Vector2::new(start.x, start.y + len / 2.0);
        let size = Vector2::new(LINE_SHAPE_WIDTH, len);
        self.shape.sprite.size().set(size);
        self.set_position_xy(pos);
    }
}

const NODE_PADDING     : f32 = node::SHADOW_SIZE;
const NODE_HEIGHT      : f32 = node::NODE_HEIGHT;
const NODE_HALF_HEIGHT : f32 = NODE_HEIGHT / 2.0;
const MOUSE_OFFSET     : f32 = 2.0;

impl EdgeModelData {

    pub fn redraw(&self) {
        let line_side_overlap  = 1.0;
        let line_sides_overlap = 2.0 * line_side_overlap;
        let line_shape_width   = LINE_WIDTH + 2.0 * PADDING;

        let fg              = &self.front;
        let bg              = &self.back;
        let target_attached = self.target_attached.get();

        let node_half_width  = self.source_width.get() / 2.0;
        let node_circle      = Vector2::new(node_half_width-NODE_HALF_HEIGHT,0.0);


        // === Target ===
        //
        // Target is the end position of the connection in local node space (the origin is placed in
        // the center of the node). We handle lines drawing in special way when target is below the
        // node (for example, not to draw the port line above source node).
        //
        // ╭──────────────╮
        // │      ┼ (0,0) │
        // ╰──────────────╯────╮
        //                     │
        //                     ▢ target

        let world_space_target     = self.target_position.get();
        let target_x               = world_space_target.x - self.position().x;
        let target_y               = world_space_target.y - self.position().y;
        let side                   = target_x.signum();
        let target_x               = target_x.abs();
        let target                 = Vector2::new(target_x,target_y);
        let target_is_below_node_x = target.x < node_half_width;
        let target_is_below_node_y = target.y < (-NODE_HALF_HEIGHT);
        let target_is_below_node   = target_is_below_node_x && target_is_below_node_y;


        let port_line_len_max     = NODE_HALF_HEIGHT + NODE_PADDING;



        // === Upward Discovery ===
        //
        // Discovers when the connection should go upwards. The `upward_corner_radius` defines the
        // preferred radius for every corner in this scenario.
        //
        // ╭─────╮    ╭─╮
        // ╰─────╯────╯ │
        //              ▢

        let upward_corner_radius        = 20.0;
        let min_len_for_non_curved_line = upward_corner_radius + port_line_len_max;




        // === Flat side ===
        //
        // Maximum side distance before connection is curved up.
        //
        // ╭─────╮◄──►    ╭─────╮◄──►╭─╮
        // ╰─────╯───╮    ╰─────╯────╯ │
        //           ▢                 ▢

        let flat_side_size = 40.0;
        let is_flat_side   = target.x < node_half_width + flat_side_size;
        let downward_flat  = if target_is_below_node_x {target_is_below_node_y} else {target.y<0.0};
        let downward_far   = -target.y > min_len_for_non_curved_line || target_is_below_node;
        let is_down        = if is_flat_side {downward_flat} else {downward_far};





        // === Port Line Length ===
        //
        // ╭──╮
        // ╰──╯───╮
        //        ╵
        //     ╭──┼──╮ ▲  Port line covers the area above target node and the area of target node
        //     │  ▢  │ ▼  shadow. It can be shorter if the target position is below the node or the
        //     ╰─────╯    connection is being dragged, in order not to overlap with the source node.

        let port_line_start    = Vector2::new(side * target.x, target.y + MOUSE_OFFSET);
        let space_attached     = -port_line_start.y - NODE_HALF_HEIGHT - LINE_SIDE_OVERLAP;
        let space              = space_attached - NODE_PADDING;
        let len_below_free     = max(0.0,min(port_line_len_max,space));
        let len_below_attached = max(0.0,min(port_line_len_max,space_attached));
        let len_below          = if target_attached {len_below_attached} else {len_below_free};
        let far_side_len       = if target_is_below_node {len_below} else {port_line_len_max};
        let flat_side_len      = min(far_side_len,-target.y);
        let port_line_len      = if is_flat_side && is_down {flat_side_len} else {far_side_len};
        let port_line_len_diff = port_line_len_max - port_line_len;




        let port_line_end   = Vector2::new(target.x,target.y + port_line_len);
//        let main_line_target   = Vector2::new(target.x,port_line_end.y); // + port_line_len_diff);






        let upward_distance             = target.y + min_len_for_non_curved_line;




        let mut corner1_target = port_line_end;

        if !is_down {
            corner1_target.x = if is_flat_side {
                node_half_width + upward_corner_radius + max(0.0,target.x - node_half_width + upward_corner_radius)
            } else {
                min(node_half_width + (target.x - node_half_width)/2.0,node_half_width + 2.0*upward_corner_radius)
            };
            corner1_target.y = min(upward_corner_radius,upward_distance/2.0);
        }


//                if !is_down && !is_flat_side {
//                    corner_radius = corner1_target.x - node_half_width;
//                    println!("? {}", corner_radius);
//                }


        // === Corner ===

        let corner_grow   = ((corner1_target.x - node_half_width) * 0.6).max(0.0);
        let corner_radius = 20.0 + corner_grow;
        let mut corner_radius = corner_radius.min(corner1_target.y.abs());
        let corner_x      = corner1_target.x - corner_radius;


        //      r1
        //    ◄───►                  (1) x^2 + y^2 = r1^2 + r2^2
        //    _____                  (1) => y = sqrt((r1^2 + r2^2)/x^2)
        //  .'     `.
        // /   _.-"""B-._     ▲
        // | .'0┼    |   `.   │      angle1 = A-XY-0
        // \/   │    /     \  │ r2   angle2 = 0-XY-B
        // |`._ │__.'       | │      alpha  = B-XY-X_AXIS
        // |   A└───┼─      | ▼
        // |      (x,y)     |        tg(angle1) = y  / x
        //  \              /         tg(angle2) = r1 / r2
        //   `._        _.'          alpha      = PI - angle1 - angle2
        //      `-....-'


        let x             = (corner_x - node_circle.x).clamp(-corner_radius,NODE_HALF_HEIGHT);
        let y             = (NODE_HALF_HEIGHT*NODE_HALF_HEIGHT + corner_radius*corner_radius - x*x).sqrt();
        let angle1        = f32::atan2(y,x);
        let angle2        = f32::atan2(NODE_HALF_HEIGHT,corner_radius);
        let angle_overlap = if corner_x > node_half_width { 0.0 } else { 0.1 };
        let corner_angle  = std::f32::consts::PI - angle1 - angle2;
        let corner_angle  = (corner_angle + angle_overlap) * side;
        let corner_angle  = if is_down {corner_angle} else {side * std::f32::consts::PI / 2.0};
        let corner_side   = (corner_radius + PADDING) * 2.0;
        let corner_size   = Vector2::new(corner_side,corner_side);
        let corner_y      = if is_down {-y} else {y};
        let start_angle   = if is_down {0.0} else {side * std::f32::consts::PI / 2.0};
        let corner        = Vector2::new(corner_x*side,corner_y);

        bg.corner.shape.sprite.size().set(corner_size);
        bg.corner.shape.start_angle.set(start_angle);
        bg.corner.shape.angle.set(corner_angle);
        bg.corner.shape.radius.set(corner_radius);
        bg.corner.shape.pos.set(corner);
        bg.corner.set_position_xy(corner);
        if !target_attached {
            bg.corner.shape.dim.set(Vector2::new(node_half_width,NODE_HALF_HEIGHT));
            fg.corner.shape.sprite.size().set(corner_size);
            fg.corner.shape.start_angle.set(start_angle);
            fg.corner.shape.angle.set(corner_angle);
            fg.corner.shape.radius.set(corner_radius);
            fg.corner.shape.pos.set(corner);
            fg.corner.shape.width.set(node_half_width);
            fg.corner.set_position_xy(corner);
        } else {
            fg.corner.shape.sprite.size().set(zero());
            bg.corner.shape.dim.set(Vector2::new(INFINITE,INFINITE));
        }


        // === Side Line ===

        let side_line_len = corner_x - node_half_width;

        if target_attached {
            let bg_line_len  = side_line_len;
            let bg_line_x    = side * (node_half_width + bg_line_len/2.0);
            let bg_line_size = Vector2::new(line_shape_width,bg_line_len + line_sides_overlap);

            fg.side_line.shape.sprite.size().set(zero());
            bg.side_line.shape.sprite.size().set(bg_line_size);
            bg.side_line.mod_position(|p| p.x = bg_line_x);

        } else {
            let bg_line_len  = min(side_line_len,NODE_PADDING);
            let fg_line_len  = side_line_len - bg_line_len;
            let bg_line_x    = side * (node_half_width + bg_line_len/2.0);
            let fg_line_x    = side * (node_half_width + fg_line_len/2.0 + bg_line_len + line_side_overlap);
            let bg_line_size = Vector2::new(line_shape_width,bg_line_len + line_sides_overlap * 2.0);
            let fg_line_size = Vector2::new(line_shape_width,fg_line_len);

            bg.side_line.shape.sprite.size().set(bg_line_size);
            bg.side_line.mod_position(|p| p.x = bg_line_x);

            fg.side_line.shape.sprite.size().set(fg_line_size);
            fg.side_line.mod_position(|p| p.x = fg_line_x);
        }



        // === Main Line (downwards) ===
        //
        // Main line is the long vertical line. In case it is placed below the node and the edge is
        // in drag mode, it is divided into two segments. The upper segment is drawn behind node
        // shadow, while the second is drawn on the top layer. In case of edge in drag mode drawn
        // next to node, only the top layer segment is used.
        //
        // Please note that only applies to edges going down. Refer to docs of main line of edges
        // going up to learn more.
        //
        // Double edge:   Single edge:
        // ╭─────╮        ╭─────╮
        // ╰──┬──╯        ╰─────╯────╮
        //    ╷                      │
        //    ▢                      ▢

        if is_down {
            let main_line_end_y = corner.y;
            let main_line_len   = main_line_end_y - port_line_start.y;
            if !target_attached && target_is_below_node {
                let back_line_start_y = max(-NODE_HALF_HEIGHT - NODE_PADDING, port_line_start.y);
                let back_line_start = Vector2::new(port_line_start.x, back_line_start_y);
                let back_line_len = main_line_end_y - back_line_start_y;
                let front_line_len = main_line_len - back_line_len;
                bg.main_line.layout(back_line_start, back_line_len);
                fg.main_line.layout(port_line_start, front_line_len);
            } else if target_attached {
                let main_line_start_y = port_line_start.y + port_line_len;
                let main_line_start = Vector2::new(port_line_start.x, main_line_start_y);
                fg.main_line.shape.sprite.size().set(zero());
                bg.main_line.layout(main_line_start, main_line_len - port_line_len);
            } else {
                bg.main_line.shape.sprite.size().set(zero());
                fg.main_line.layout(port_line_start, main_line_len);
            }
        }







        if !is_down {


            //
            // ╭─────╮  ╭──╮
            // ╰─────╯──╯  ▢

            // ╭─────╮
            // ╰─────╯────╮
            //            ▢

            // ╭─────╮
            // ╰──┬──╯
            //    ▢

            let corner2_radius = corner_radius;
            let corner3_radius = upward_corner_radius;

            let corner2_x      = corner1_target.x + corner_radius;
            let corner3_x      = port_line_end.x - corner3_radius;
            let corner2_bbox_x = corner2_x - corner2_radius;
            let corner3_bbox_x = corner3_x + corner3_radius;

            let corner_2_3_dist     = corner3_bbox_x - corner2_bbox_x;
            let corner_2_3_side     = corner_2_3_dist.signum();
            let corner_2_3_dist     = corner_2_3_dist.abs();
            let corner_2_3_width    = corner2_radius + corner3_radius;
            let corner_2_3_do_scale = corner_2_3_dist < corner_2_3_width;
            let corner_2_3_scale    = corner_2_3_dist / corner_2_3_width;
            let corner_2_3_scale    = if corner_2_3_do_scale {corner_2_3_scale} else {1.0};

            let side_combined       = side * corner_2_3_side;

            let corner2_radius = corner2_radius * corner_2_3_scale;
            let corner3_radius = corner3_radius * corner_2_3_scale;

//////////////////

            let corner3_side   = (corner3_radius + PADDING) * 2.0;
            let corner3_size   = Vector2::new(corner3_side,corner3_side);
            let corner3_x      = port_line_end.x - corner_2_3_side * corner3_radius;
            let corner3_y      = port_line_end.y;

            let corner2_y      = corner3_y + corner3_radius - corner2_radius;
            let corner2_y      = max(corner2_y, corner_y);

            let corner3_y      = max(corner3_y,corner2_y - corner3_radius + corner2_radius);

            let corner3        = Vector2::new(corner3_x*side,corner3_y);
            let corner3_angle  = if (side_combined == 1.0) {0.0} else {-std::f32::consts::PI / 2.0};

            fg.corner3.shape.sprite.size().set(corner3_size);
            fg.corner3.shape.start_angle.set(corner3_angle);
            fg.corner3.shape.angle.set(std::f32::consts::PI / 2.0);
            fg.corner3.shape.radius.set(corner3_radius);
            fg.corner3.shape.pos.set(corner3);
            fg.corner3.shape.width.set(0.0);
            fg.corner3.set_position_xy(corner3);

            // port_line_len = corner3_y - target.y;

            let xoff = 20.0;

            let corner2_x      = corner1_target.x + corner_2_3_side * corner2_radius;

            let corner2        = Vector2::new(corner2_x*side,corner2_y);
            let corner2_angle  = if (side_combined == 1.0) {-std::f32::consts::PI / 2.0} else {0.0};

            fg.corner2.shape.sprite.size().set(corner_size);
            fg.corner2.shape.start_angle.set(corner2_angle);
            fg.corner2.shape.angle.set(std::f32::consts::PI / 2.0);
            fg.corner2.shape.radius.set(corner2_radius);
            fg.corner2.shape.pos.set(corner2);
            fg.corner2.shape.width.set(0.0);
            fg.corner2.set_position_xy(corner2);






            let main_line_len    = corner2_y - corner_y;
            let main_line_size   = Vector2::new(line_shape_width,main_line_len + line_sides_overlap);
            let main_line_x      = side * corner1_target.x;
            let main_line_y      = main_line_len / 2.0 + corner_y;
            let main_line_pos    = Vector2::new(main_line_x,main_line_y);

            fg.main_line.shape.sprite.size().set(main_line_size);
            fg.main_line.set_position_xy(main_line_pos);

            if main_line_len > 0.0 {
                let arrow_pos = Vector2::new(main_line_pos.x, (corner_y - corner_radius + corner2_y + corner2_radius)/2.0);
                fg.arrow.shape.sprite.size().set(Vector2::new(20.0,20.0));
                fg.arrow.set_position_xy(arrow_pos);
            } else {
                fg.arrow.shape.sprite.size().set(zero());
            }

            let side_line2_len  = corner3_x - corner2_x;
            let side_line2_x    = side * (corner2_x + side_line2_len / 2.0);
            let side_line2_y    = corner2_y + corner2_radius;
            let side_line2_len  = side_line2_len.abs();
            let side_line2_pos  = Vector2::new(side_line2_x,side_line2_y);
            let side_line2_size = Vector2::new(line_shape_width,side_line2_len + line_sides_overlap);

            fg.side_line2.shape.sprite.size().set(side_line2_size);
            fg.side_line2.set_position_xy(side_line2_pos);

        } else {
            fg.corner3.shape.sprite.size().set(zero());
            fg.corner2.shape.sprite.size().set(zero());
            fg.side_line2.shape.sprite.size().set(zero());
        }

        // === Port Line ===

        fg.port_line.layout(port_line_start, port_line_len);


//        fg.port_line.shape.sprite.size().set(Vector2::new(line_shape_width,port_line_len-MOUSE_OFFSET));
//        fg.port_line.mod_position(|p| {
//            p.x = port_line_start.x;
//            p.y = port_line_end.y - port_line_len_max + port_line_len/2.0 + MOUSE_OFFSET;
//        });
    }
}

impl Edge {
    /// Constructor.
    pub fn new(scene:&Scene) -> Self {
        let logger    = Logger::new("edge");
        let display_object    = display::object::Instance::new(&logger);
        let fg     = Front::new(logger.sub("fg"),scene);
        let bg      = Back::new(logger.sub("bg"),scene);

        display_object.add_child(&fg);
        display_object.add_child(&bg);

        fg.side_line.mod_rotation(|r| r.z = std::f32::consts::PI/2.0);
        bg.side_line.mod_rotation(|r| r.z = std::f32::consts::PI/2.0);
        fg.side_line2.mod_rotation(|r| r.z = std::f32::consts::PI/2.0);
        bg.side_line2.mod_rotation(|r| r.z = std::f32::consts::PI/2.0);

        let network = frp::Network::new();
        let input = Frp::new(&network);

        let source_width : Rc<Cell<f32>> = default();
        let target_position = Rc::new(Cell::new(frp::Position::default()));
        source_width.set(100.0);

        let target_attached : Rc<Cell<bool>> = default();







        let frp = input;
        let front = fg;
        let back  = bg;
        let data = Rc::new(EdgeModelData {display_object,logger,frp,front,back
                                          ,source_width,target_position,target_attached});
        let model = Rc::new(EdgeModel {data});




        Self {model,network} . init()
    }

    fn init(self) -> Self {
        let network         = &self.network;
        let input           = &self.frp;
        let target_position = &self.target_position;
        let target_attached = &self.target_attached;
        let source_width    = &self.source_width;
        let model           = &self.model;
        frp::extend! { network
            eval input.target_position ((t) target_position.set(*t));
            eval input.target_attached ((t) target_attached.set(*t));
            eval input.source_width    ((t) source_width.set(*t));
            on_change <- any_ (input.source_width, input.target_position, input.target_attached);
            eval_ on_change (model.redraw());
        }
        self
    }
}

impl display::Object for Edge {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
